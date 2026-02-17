//! BrowserManager — CDP browser lifecycle with profile support.
//!
//! Central browser lifecycle manager. Profile-aware: launches Chrome with
//! `--user-data-dir` pointing to the saved profile so cookies/sessions persist.

use crate::profile::ProfileManager;
use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::Page;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for the BrowserManager.
#[derive(Debug, Clone)]
pub struct BrowserManagerConfig {
    /// Custom Chrome/Edge binary path.
    pub browser_path: Option<String>,
    /// Connect to an already-running browser via CDP URL.
    pub cdp_url: Option<String>,
    /// Run headless (default: true).
    pub headless: bool,
    /// Browser window size.
    pub window_size: (u32, u32),
    /// Named profile to use for session persistence.
    pub profile: Option<String>,
}

impl Default for BrowserManagerConfig {
    fn default() -> Self {
        Self {
            browser_path: None,
            cdp_url: None,
            headless: true,
            window_size: (1280, 720),
            profile: None,
        }
    }
}

/// Central browser lifecycle manager.
///
/// Holds a single browser instance and page. Profile-aware: when a profile is
/// specified, Chrome launches with `--user-data-dir` so cookies, localStorage,
/// and saved passwords persist across sessions.
pub struct BrowserManager {
    browser: RwLock<Option<Browser>>,
    page: RwLock<Option<Page>>,
    config: BrowserManagerConfig,
    profile_manager: Arc<ProfileManager>,
}

impl BrowserManager {
    pub fn new(config: BrowserManagerConfig, profile_manager: Arc<ProfileManager>) -> Self {
        Self {
            browser: RwLock::new(None),
            page: RwLock::new(None),
            config,
            profile_manager,
        }
    }

    /// Ensure a browser is running, launching one if needed.
    pub async fn ensure_browser(&self) -> Result<()> {
        // Fast path: browser already running
        {
            let guard = self.browser.read().await;
            if guard.is_some() {
                return Ok(());
            }
        }

        let mut browser_guard = self.browser.write().await;
        // Double-check after acquiring write lock
        if browser_guard.is_some() {
            return Ok(());
        }

        let browser = if let Some(ref cdp_url) = self.config.cdp_url {
            let (browser, mut handler) =
                Browser::connect(cdp_url)
                    .await
                    .with_context(|| format!("Failed to connect to browser at {}", cdp_url))?;

            tokio::spawn(async move {
                while let Some(h) = handler.next().await {
                    if h.is_err() {
                        break;
                    }
                }
            });

            browser
        } else {
            let mut builder = BrowserConfig::builder();

            if let Some(ref path) = self.config.browser_path {
                builder = builder.chrome_executable(path);
            }

            if !self.config.headless {
                builder = builder.with_head();
            }

            builder = builder.window_size(self.config.window_size.0, self.config.window_size.1);

            // Profile support: set user-data-dir for session persistence
            if let Some(ref profile_name) = self.config.profile {
                let user_data_dir = self.profile_manager.user_data_dir(profile_name)?;
                builder = builder.user_data_dir(user_data_dir);
                let _ = self.profile_manager.touch_profile(profile_name);
            }

            // Chrome args for stability and compatibility
            builder = builder
                .arg("--disable-dev-shm-usage")
                .arg("--remote-allow-origins=*");

            let config = builder.build().map_err(|e| anyhow::anyhow!("{}", e))?;

            let (browser, mut handler) = Browser::launch(config)
                .await
                .context("Failed to launch browser")?;

            tokio::spawn(async move {
                while let Some(h) = handler.next().await {
                    if h.is_err() {
                        break;
                    }
                }
            });

            browser
        };

        *browser_guard = Some(browser);
        Ok(())
    }

    /// Get or create the active page.
    pub async fn page(&self) -> Result<Page> {
        self.ensure_browser().await?;

        // Fast path: page already exists
        {
            let guard = self.page.read().await;
            if let Some(ref page) = *guard {
                return Ok(page.clone());
            }
        }

        let mut page_guard = self.page.write().await;
        // Double-check
        if let Some(ref page) = *page_guard {
            return Ok(page.clone());
        }

        let browser_guard = self.browser.read().await;
        let browser = browser_guard
            .as_ref()
            .context("Browser not initialized")?;

        let page = browser
            .new_page("about:blank")
            .await
            .context("Failed to create new page")?;

        *page_guard = Some(page.clone());
        Ok(page)
    }

    /// Launch a non-headless browser for manual login (used by setup-login).
    pub async fn launch_for_login(
        profile_manager: Arc<ProfileManager>,
        profile_name: &str,
        url: &str,
        browser_path: Option<String>,
    ) -> Result<Browser> {
        let user_data_dir = profile_manager.user_data_dir(profile_name)?;

        let mut builder = BrowserConfig::builder()
            .with_head()
            .window_size(1280, 900)
            .user_data_dir(user_data_dir)
            .arg("--disable-dev-shm-usage")
            .arg("--remote-allow-origins=*");

        if let Some(ref path) = browser_path {
            builder = builder.chrome_executable(path);
        }

        let config = builder.build().map_err(|e| anyhow::anyhow!("{}", e))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .context("Failed to launch browser for login")?;

        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        // Navigate to the login URL
        let page = browser
            .new_page(url)
            .await
            .context("Failed to open login page")?;

        tracing::info!("Browser opened at {}", url);

        // Keep page alive (it's attached to the browser)
        drop(page);

        Ok(browser)
    }
}
