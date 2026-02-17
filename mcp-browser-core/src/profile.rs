//! Profile management for browser sessions.
//!
//! Enables users to log in manually (SSO/MFA/password), save the browser profile,
//! and reuse it later. Profiles persist Chrome's user-data-dir so cookies, localStorage,
//! and saved passwords carry across sessions.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Metadata for a single browser profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub description: String,
    /// Absolute path to Chrome user-data-dir for this profile.
    pub user_data_dir: PathBuf,
    /// Browser channel: "chrome", "msedge", "chromium".
    pub browser_channel: String,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub usage_count: u64,
    /// If true, user must manually log in before automation works.
    pub requires_human_login: bool,
    /// Notes for the user about what login is needed.
    pub login_notes: String,
    /// Hours before session is considered expired.
    pub session_timeout_hours: u64,
}

/// Top-level profiles.json structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfilesFile {
    profiles: std::collections::HashMap<String, ProfileMetadata>,
    version: String,
}

impl Default for ProfilesFile {
    fn default() -> Self {
        Self {
            profiles: std::collections::HashMap::new(),
            version: "1.0".to_string(),
        }
    }
}

/// Options for creating a new profile.
pub struct CreateOpts {
    pub description: String,
    pub browser_channel: String,
    pub requires_human_login: bool,
    pub login_notes: String,
    pub session_timeout_hours: u64,
}

impl Default for CreateOpts {
    fn default() -> Self {
        Self {
            description: String::new(),
            browser_channel: "chrome".to_string(),
            requires_human_login: false,
            login_notes: String::new(),
            session_timeout_hours: 24,
        }
    }
}

/// Result of profile validation.
#[derive(Debug)]
pub struct ProfileValidation {
    pub exists: bool,
    pub has_cookies: bool,
    pub session_valid: bool,
}

/// Manages browser profiles on disk.
///
/// Profile storage:
/// - macOS: `~/Library/Application Support/mcp-browser-server/profiles/`
/// - Linux: `~/.local/share/mcp-browser-server/profiles/`
/// - Override: `BROWSER_PROFILES_DIR` env var
pub struct ProfileManager {
    profiles_dir: PathBuf,
}

impl ProfileManager {
    /// Create a new ProfileManager, auto-detecting the OS-appropriate path.
    pub fn new() -> Result<Self> {
        let profiles_dir = resolve_profiles_dir()?;
        std::fs::create_dir_all(&profiles_dir)
            .with_context(|| format!("Failed to create profiles dir: {}", profiles_dir.display()))?;
        Ok(Self { profiles_dir })
    }

    /// Create a ProfileManager with a specific directory (useful for testing).
    pub fn with_dir(profiles_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&profiles_dir)
            .with_context(|| format!("Failed to create profiles dir: {}", profiles_dir.display()))?;
        Ok(Self { profiles_dir })
    }

    /// List all profiles.
    pub fn list_profiles(&self) -> Result<Vec<ProfileMetadata>> {
        let file = self.load_profiles_file()?;
        Ok(file.profiles.into_values().collect())
    }

    /// Get a single profile by name.
    pub fn get_profile(&self, name: &str) -> Result<ProfileMetadata> {
        let file = self.load_profiles_file()?;
        file.profiles
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))
    }

    /// Create a new profile.
    pub fn create_profile(&self, name: &str, opts: CreateOpts) -> Result<ProfileMetadata> {
        let mut file = self.load_profiles_file()?;

        if file.profiles.contains_key(name) {
            anyhow::bail!("Profile '{}' already exists", name);
        }

        let profile_data_dir = self.profiles_dir.join(name);
        std::fs::create_dir_all(&profile_data_dir)
            .with_context(|| format!("Failed to create profile data dir: {}", profile_data_dir.display()))?;

        let now = Utc::now();
        let metadata = ProfileMetadata {
            name: name.to_string(),
            description: opts.description,
            user_data_dir: profile_data_dir,
            browser_channel: opts.browser_channel,
            created_at: now,
            last_used: now,
            usage_count: 0,
            requires_human_login: opts.requires_human_login,
            login_notes: opts.login_notes,
            session_timeout_hours: opts.session_timeout_hours,
        };

        file.profiles.insert(name.to_string(), metadata.clone());
        self.save_profiles_file(&file)?;

        Ok(metadata)
    }

    /// Delete a profile and its data directory.
    pub fn delete_profile(&self, name: &str) -> Result<()> {
        let mut file = self.load_profiles_file()?;

        let profile = file
            .profiles
            .remove(name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))?;

        // Remove the user-data-dir
        if profile.user_data_dir.exists() {
            std::fs::remove_dir_all(&profile.user_data_dir)
                .with_context(|| format!("Failed to remove profile data: {}", profile.user_data_dir.display()))?;
        }

        self.save_profiles_file(&file)?;
        Ok(())
    }

    /// Update last_used timestamp and increment usage count.
    pub fn touch_profile(&self, name: &str) -> Result<()> {
        let mut file = self.load_profiles_file()?;
        let profile = file
            .profiles
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))?;

        profile.last_used = Utc::now();
        profile.usage_count += 1;
        self.save_profiles_file(&file)?;
        Ok(())
    }

    /// Check if the profile's session is still valid (not expired).
    pub fn is_session_valid(&self, name: &str) -> Result<bool> {
        let profile = self.get_profile(name)?;
        let elapsed = Utc::now() - profile.last_used;
        let timeout = chrono::Duration::hours(profile.session_timeout_hours as i64);
        Ok(elapsed < timeout)
    }

    /// Get the user-data-dir path for a profile.
    pub fn user_data_dir(&self, name: &str) -> Result<PathBuf> {
        let profile = self.get_profile(name)?;
        Ok(profile.user_data_dir)
    }

    /// Validate a profile: check existence, cookies, session.
    pub fn validate_profile(&self, name: &str) -> Result<ProfileValidation> {
        let profile = match self.get_profile(name) {
            Ok(p) => p,
            Err(_) => {
                return Ok(ProfileValidation {
                    exists: false,
                    has_cookies: false,
                    session_valid: false,
                });
            }
        };

        let has_cookies = profile.user_data_dir.exists()
            && (profile.user_data_dir.join("Default/Cookies").exists()
                || profile.user_data_dir.join("Cookies").exists());

        let session_valid = self.is_session_valid(name).unwrap_or(false);

        Ok(ProfileValidation {
            exists: true,
            has_cookies,
            session_valid,
        })
    }

    /// Get or create a profile (used by setup-login and serve).
    pub fn get_or_create_profile(&self, name: &str, opts: CreateOpts) -> Result<ProfileMetadata> {
        match self.get_profile(name) {
            Ok(profile) => Ok(profile),
            Err(_) => self.create_profile(name, opts),
        }
    }

    fn profiles_file_path(&self) -> PathBuf {
        self.profiles_dir.join("profiles.json")
    }

    fn load_profiles_file(&self) -> Result<ProfilesFile> {
        let path = self.profiles_file_path();
        if !path.exists() {
            return Ok(ProfilesFile::default());
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let file: ProfilesFile = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(file)
    }

    fn save_profiles_file(&self, file: &ProfilesFile) -> Result<()> {
        let path = self.profiles_file_path();
        let contents = serde_json::to_string_pretty(file)?;
        std::fs::write(&path, contents)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }
}

/// Resolve the profiles directory using OS-appropriate paths.
///
/// Priority:
/// 1. `BROWSER_PROFILES_DIR` env var
/// 2. OS-specific data directory
fn resolve_profiles_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("BROWSER_PROFILES_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let proj_dirs = directories::ProjectDirs::from("com", "openclaw", "mcp-browser-server")
        .context("Failed to determine data directory for this OS")?;

    Ok(proj_dirs.data_dir().join("profiles"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manager() -> (ProfileManager, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let manager = ProfileManager::with_dir(tmp.path().to_path_buf()).unwrap();
        (manager, tmp)
    }

    #[test]
    fn test_create_and_get_profile() {
        let (manager, _tmp) = test_manager();

        let profile = manager
            .create_profile("test", CreateOpts::default())
            .unwrap();
        assert_eq!(profile.name, "test");
        assert_eq!(profile.usage_count, 0);

        let fetched = manager.get_profile("test").unwrap();
        assert_eq!(fetched.name, "test");
    }

    #[test]
    fn test_create_duplicate_fails() {
        let (manager, _tmp) = test_manager();
        manager
            .create_profile("dup", CreateOpts::default())
            .unwrap();
        assert!(manager.create_profile("dup", CreateOpts::default()).is_err());
    }

    #[test]
    fn test_list_profiles() {
        let (manager, _tmp) = test_manager();
        manager
            .create_profile("a", CreateOpts::default())
            .unwrap();
        manager
            .create_profile("b", CreateOpts::default())
            .unwrap();

        let profiles = manager.list_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_delete_profile() {
        let (manager, _tmp) = test_manager();
        manager
            .create_profile("del", CreateOpts::default())
            .unwrap();
        manager.delete_profile("del").unwrap();
        assert!(manager.get_profile("del").is_err());
    }

    #[test]
    fn test_touch_profile() {
        let (manager, _tmp) = test_manager();
        manager
            .create_profile("touch", CreateOpts::default())
            .unwrap();

        let before = manager.get_profile("touch").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.touch_profile("touch").unwrap();
        let after = manager.get_profile("touch").unwrap();

        assert!(after.last_used >= before.last_used);
        assert_eq!(after.usage_count, 1);
    }

    #[test]
    fn test_session_validity() {
        let (manager, _tmp) = test_manager();
        manager
            .create_profile(
                "session",
                CreateOpts {
                    session_timeout_hours: 24,
                    ..Default::default()
                },
            )
            .unwrap();

        // Just created, should be valid
        assert!(manager.is_session_valid("session").unwrap());
    }

    #[test]
    fn test_validate_nonexistent_profile() {
        let (manager, _tmp) = test_manager();
        let validation = manager.validate_profile("nope").unwrap();
        assert!(!validation.exists);
        assert!(!validation.has_cookies);
        assert!(!validation.session_valid);
    }

    #[test]
    fn test_profiles_json_roundtrip() {
        let (manager, _tmp) = test_manager();
        manager
            .create_profile(
                "rt",
                CreateOpts {
                    description: "test profile".to_string(),
                    browser_channel: "msedge".to_string(),
                    requires_human_login: true,
                    login_notes: "Log into Okta".to_string(),
                    session_timeout_hours: 8,
                },
            )
            .unwrap();

        // Re-read from disk
        let profile = manager.get_profile("rt").unwrap();
        assert_eq!(profile.description, "test profile");
        assert_eq!(profile.browser_channel, "msedge");
        assert!(profile.requires_human_login);
        assert_eq!(profile.login_notes, "Log into Okta");
        assert_eq!(profile.session_timeout_hours, 8);
    }

    #[test]
    fn test_get_or_create_profile() {
        let (manager, _tmp) = test_manager();

        // First call creates
        let p1 = manager
            .get_or_create_profile("goc", CreateOpts::default())
            .unwrap();
        assert_eq!(p1.name, "goc");

        // Second call gets existing
        let p2 = manager
            .get_or_create_profile("goc", CreateOpts::default())
            .unwrap();
        assert_eq!(p2.name, "goc");
        assert_eq!(p2.created_at, p1.created_at);
    }

    #[test]
    fn test_user_data_dir() {
        let (manager, tmp) = test_manager();
        manager
            .create_profile("udd", CreateOpts::default())
            .unwrap();

        let dir = manager.user_data_dir("udd").unwrap();
        assert_eq!(dir, tmp.path().join("udd"));
        assert!(dir.exists());
    }
}
