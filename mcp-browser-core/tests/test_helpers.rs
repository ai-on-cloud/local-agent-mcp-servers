//! Shared test utilities for browser integration tests.
//!
//! Provides browser path resolution, manager creation, preflight checking,
//! and script execution helpers used by both local and Amplify test suites.

use mcp_browser_core::browser::{BrowserManager, BrowserManagerConfig};
use mcp_browser_core::code_mode;
use mcp_browser_core::profile::ProfileManager;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Monotonic counter so each test_manager() gets a unique profile name,
/// avoiding Chrome SingletonLock conflicts between sequential tests.
static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Cached result of the browser preflight check.
/// `Ok(())` means the browser can launch; `Err(msg)` means it can't.
static PREFLIGHT: OnceCell<Result<(), String>> = OnceCell::const_new();

/// Resolve the browser binary path from the BROWSER env var.
pub fn resolve_browser_path() -> Option<String> {
    match std::env::var("BROWSER").ok().as_deref() {
        None | Some("") | Some("chrome") => None, // auto-detect (picks Chrome)
        Some("edge") => {
            if cfg!(target_os = "macos") {
                Some("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge".to_string())
            } else if cfg!(target_os = "windows") {
                Some(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".to_string())
            } else {
                Some("microsoft-edge-stable".to_string())
            }
        }
        Some(path) if path.starts_with('/') || path.starts_with('\\') || path.contains(':') => {
            Some(path.to_string())
        }
        Some(other) => {
            panic!(
                "Unknown BROWSER value: '{}'. Use 'chrome', 'edge', or an absolute path.",
                other
            );
        }
    }
}

/// Create a headless BrowserManager for testing with the selected browser.
/// Each call gets a unique profile (user-data-dir) to avoid Chrome
/// SingletonLock conflicts from stale or concurrent sessions.
pub fn test_manager() -> Arc<BrowserManager> {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let profile_name = format!("test-{}-{}", std::process::id(), n);
    let profile_manager = Arc::new(ProfileManager::new().expect("ProfileManager init"));
    // Ensure the profile exists so user_data_dir() works
    let _ = profile_manager.get_or_create_profile(
        &profile_name,
        mcp_browser_core::profile::CreateOpts::default(),
    );
    Arc::new(BrowserManager::new(
        BrowserManagerConfig {
            headless: true,
            browser_path: resolve_browser_path(),
            profile: Some(profile_name),
            ..Default::default()
        },
        profile_manager,
    ))
}

/// Try to launch the selected browser once.  If it crashes or can't start,
/// every subsequent call returns the cached error so tests skip immediately
/// instead of repeating the same failure many times.
pub async fn preflight_check() {
    let result = PREFLIGHT
        .get_or_init(|| async {
            let mgr = test_manager();
            match mgr.ensure_browser().await {
                Ok(()) => {
                    mgr.shutdown().await;
                    Ok(())
                }
                Err(e) => Err(format!(
                    "Browser failed to launch (BROWSER={:?}): {:#}",
                    std::env::var("BROWSER").unwrap_or_default(),
                    e
                )),
            }
        })
        .await;

    if let Err(msg) = result {
        panic!(
            "SKIPPING — {}\n\
             Hint: the selected browser may be too old or incompatible with this OS.\n\
             Try updating it, or use a different browser: BROWSER=chrome",
            msg
        );
    }
}

/// Helper: validate + execute a script, returning the result JSON.
pub async fn run_script(
    manager: Arc<BrowserManager>,
    code: &str,
) -> Result<serde_json::Value, String> {
    let validation = code_mode::validate_script(code)?;
    assert!(validation.is_valid);
    code_mode::execute_script(manager, code, &validation.approval_token, None).await
}
