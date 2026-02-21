//! Diagnostic check for browser-server deployments.
//!
//! Runs a sequence of checks: detect browsers, launch headless, navigate,
//! extract text, screenshot, evaluate JS, and optionally validate a profile.
//! Used by `browser-server check` and can be called from tests.

use crate::browser::{BrowserManager, BrowserManagerConfig};
use crate::profile::ProfileManager;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// A detected browser installation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DetectedBrowser {
    pub name: String,
    pub channel: String,
    pub path: String,
    pub version: Option<String>,
    pub would_auto_select: bool,
}

/// Status of a single check step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StepStatus {
    Pass,
    Fail,
    Skip,
}

/// Result of a single check step.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckStep {
    pub name: String,
    pub status: StepStatus,
    pub message: String,
    pub duration_ms: u64,
}

/// Result of a profile check.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileCheckResult {
    pub name: String,
    pub exists: bool,
    pub has_cookies: bool,
    pub session_valid: bool,
}

/// Overall check report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckReport {
    pub timestamp: String,
    pub platform: String,
    pub browsers: Vec<DetectedBrowser>,
    pub steps: Vec<CheckStep>,
    pub profile: Option<ProfileCheckResult>,
    pub overall: bool,
}

/// Options for run_check.
pub struct CheckOptions {
    pub browser_path: Option<String>,
    pub profile: Option<String>,
    pub verbose: bool,
}

/// Candidate browser paths to scan.
struct BrowserCandidate {
    name: &'static str,
    channel: &'static str,
    paths: Vec<&'static str>,
}

fn browser_candidates() -> Vec<BrowserCandidate> {
    if cfg!(target_os = "macos") {
        vec![
            BrowserCandidate {
                name: "Google Chrome",
                channel: "chrome",
                paths: vec!["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"],
            },
            BrowserCandidate {
                name: "Microsoft Edge",
                channel: "msedge",
                paths: vec!["/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"],
            },
            BrowserCandidate {
                name: "Chromium",
                channel: "chromium",
                paths: vec!["/Applications/Chromium.app/Contents/MacOS/Chromium"],
            },
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            BrowserCandidate {
                name: "Google Chrome",
                channel: "chrome",
                paths: vec![
                    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
                ],
            },
            BrowserCandidate {
                name: "Microsoft Edge",
                channel: "msedge",
                paths: vec![
                    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
                    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
                ],
            },
        ]
    } else {
        vec![
            BrowserCandidate {
                name: "Google Chrome",
                channel: "chrome",
                paths: vec!["google-chrome-stable", "google-chrome"],
            },
            BrowserCandidate {
                name: "Microsoft Edge",
                channel: "msedge",
                paths: vec!["microsoft-edge-stable"],
            },
            BrowserCandidate {
                name: "Chromium",
                channel: "chromium",
                paths: vec!["chromium-browser", "chromium"],
            },
        ]
    }
}

/// Detect installed browsers by scanning known paths.
pub fn detect_browsers() -> Vec<DetectedBrowser> {
    let candidates = browser_candidates();
    let mut found = Vec::new();
    let mut first = true;

    for candidate in &candidates {
        for path in &candidate.paths {
            let path_buf = PathBuf::from(path);
            let exists = if path_buf.is_absolute() {
                path_buf.exists()
            } else {
                // For non-absolute paths (Linux), try `which`
                std::process::Command::new("which")
                    .arg(path)
                    .output()
                    .is_ok_and(|o| o.status.success())
            };

            if exists {
                let version = get_browser_version(path);
                found.push(DetectedBrowser {
                    name: candidate.name.to_string(),
                    channel: candidate.channel.to_string(),
                    path: path.to_string(),
                    version,
                    would_auto_select: first,
                });
                first = false;
                break; // only first matching path per candidate
            }
        }
    }

    found
}

fn get_browser_version(path: &str) -> Option<String> {
    let output = std::process::Command::new(path)
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Extract version number from output like "Google Chrome 124.0.6367.91" or
    // "Microsoft Edge 120.0.2210.144"
    let version = stdout
        .trim()
        .rsplit(' ')
        .next()
        .map(|v| v.trim().to_string());
    version
}

/// Run the full check sequence.
pub async fn run_check(opts: CheckOptions) -> CheckReport {
    let browsers = detect_browsers();
    let mut steps = Vec::new();
    let mut overall = true;

    // Step 1: launch
    let profile_manager = match ProfileManager::new() {
        Ok(pm) => Arc::new(pm),
        Err(e) => {
            steps.push(CheckStep {
                name: "launch".into(),
                status: StepStatus::Fail,
                message: format!("Failed to init ProfileManager: {}", e),
                duration_ms: 0,
            });
            return CheckReport {
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: current_platform(),
                browsers,
                steps,
                profile: None,
                overall: false,
            };
        }
    };

    let config = BrowserManagerConfig {
        headless: true,
        browser_path: opts.browser_path,
        ..Default::default()
    };

    let manager = Arc::new(BrowserManager::new(config, profile_manager.clone()));

    let t = Instant::now();
    match manager.ensure_browser().await {
        Ok(()) => {
            steps.push(CheckStep {
                name: "launch".into(),
                status: StepStatus::Pass,
                message: "Browser launched in headless mode".into(),
                duration_ms: t.elapsed().as_millis() as u64,
            });
        }
        Err(e) => {
            steps.push(CheckStep {
                name: "launch".into(),
                status: StepStatus::Fail,
                message: format!("Failed to launch browser: {}", e),
                duration_ms: t.elapsed().as_millis() as u64,
            });
            return CheckReport {
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: current_platform(),
                browsers,
                steps,
                profile: None,
                overall: false,
            };
        }
    }

    // Use code_mode for the remaining steps, running a single script
    let check_script = r#"
        await api.post("/navigate", { url: "data:text/html,<html><head><title>Check</title></head><body><h1>Check</h1></body></html>" });

        const heading = await api.post("/get_text", { selector: "h1" });
        const shot = await api.post("/screenshot", {});
        const title = await api.post("/evaluate", { expression: "document.title" });

        return { heading: heading, screenshot: shot, title: title };
    "#;

    let validation = match crate::code_mode::validate_script(check_script) {
        Ok(v) if v.is_valid => v,
        Ok(_) => {
            steps.push(CheckStep {
                name: "navigate".into(),
                status: StepStatus::Fail,
                message: "Check script validation failed".into(),
                duration_ms: 0,
            });
            return CheckReport {
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: current_platform(),
                browsers,
                steps,
                profile: None,
                overall: false,
            };
        }
        Err(e) => {
            steps.push(CheckStep {
                name: "navigate".into(),
                status: StepStatus::Fail,
                message: format!("Script validation error: {}", e),
                duration_ms: 0,
            });
            return CheckReport {
                timestamp: chrono::Utc::now().to_rfc3339(),
                platform: current_platform(),
                browsers,
                steps,
                profile: None,
                overall: false,
            };
        }
    };

    let t = Instant::now();
    let script_result = crate::code_mode::execute_script(
        manager.clone(),
        check_script,
        &validation.approval_token,
        None,
    )
    .await;

    let script_elapsed = t.elapsed().as_millis() as u64;

    match script_result {
        Ok(result) => {
            let inner = &result["result"];

            // navigate step
            steps.push(CheckStep {
                name: "navigate".into(),
                status: StepStatus::Pass,
                message: "Navigated to test page".into(),
                duration_ms: script_elapsed / 4, // approximate split
            });

            // get_text step
            let heading_text = inner["heading"]["text"].as_str().unwrap_or("");
            if heading_text.contains("Check") {
                steps.push(CheckStep {
                    name: "get_text".into(),
                    status: StepStatus::Pass,
                    message: format!("Extracted heading: '{}'", heading_text),
                    duration_ms: script_elapsed / 4,
                });
            } else {
                overall = false;
                steps.push(CheckStep {
                    name: "get_text".into(),
                    status: StepStatus::Fail,
                    message: format!("Expected 'Check', got: '{}'", heading_text),
                    duration_ms: script_elapsed / 4,
                });
            }

            // screenshot step
            let screenshot = &inner["screenshot"];
            let shot_data = screenshot["screenshot"]
                .as_str()
                .or_else(|| screenshot["data"].as_str())
                .or_else(|| screenshot.as_str())
                .unwrap_or("");
            if !shot_data.is_empty() {
                steps.push(CheckStep {
                    name: "screenshot".into(),
                    status: StepStatus::Pass,
                    message: format!("Screenshot captured ({} bytes)", shot_data.len()),
                    duration_ms: script_elapsed / 4,
                });
            } else {
                overall = false;
                steps.push(CheckStep {
                    name: "screenshot".into(),
                    status: StepStatus::Fail,
                    message: "Screenshot returned empty data".into(),
                    duration_ms: script_elapsed / 4,
                });
            }

            // evaluate step
            let title_val = &inner["title"];
            let title = title_val
                .as_str()
                .or_else(|| title_val["result"].as_str())
                .or_else(|| title_val["value"].as_str())
                .unwrap_or("");
            if title.contains("Check") {
                steps.push(CheckStep {
                    name: "evaluate".into(),
                    status: StepStatus::Pass,
                    message: "JS evaluation returned expected result".into(),
                    duration_ms: script_elapsed / 4,
                });
            } else {
                overall = false;
                steps.push(CheckStep {
                    name: "evaluate".into(),
                    status: StepStatus::Fail,
                    message: format!("Expected 'Check', got: '{}'", title),
                    duration_ms: script_elapsed / 4,
                });
            }
        }
        Err(e) => {
            overall = false;
            steps.push(CheckStep {
                name: "navigate".into(),
                status: StepStatus::Fail,
                message: format!("Script execution failed: {}", e),
                duration_ms: script_elapsed,
            });
        }
    }

    // Shutdown step
    let t = Instant::now();
    manager.shutdown().await;
    steps.push(CheckStep {
        name: "shutdown".into(),
        status: StepStatus::Pass,
        message: "Browser shut down gracefully".into(),
        duration_ms: t.elapsed().as_millis() as u64,
    });

    // Profile check (optional)
    let profile_result = if let Some(ref profile_name) = opts.profile {
        match profile_manager.validate_profile(profile_name) {
            Ok(validation) => Some(ProfileCheckResult {
                name: profile_name.clone(),
                exists: validation.exists,
                has_cookies: validation.has_cookies,
                session_valid: validation.session_valid,
            }),
            Err(_) => Some(ProfileCheckResult {
                name: profile_name.clone(),
                exists: false,
                has_cookies: false,
                session_valid: false,
            }),
        }
    } else {
        None
    };

    CheckReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        platform: current_platform(),
        browsers,
        steps,
        profile: profile_result,
        overall,
    }
}

/// Format a CheckReport as human-readable text.
pub fn format_text(report: &CheckReport, verbose: bool) -> String {
    let mut out = String::new();
    out.push_str("Browser Server Check\n\n");

    // Detected browsers
    out.push_str("  Detected browsers:\n");
    if report.browsers.is_empty() {
        out.push_str("    (none found)\n");
    } else {
        for b in &report.browsers {
            let version = b.version.as_deref().unwrap_or("unknown");
            let auto = if b.would_auto_select {
                " (auto-selected)"
            } else {
                ""
            };
            out.push_str(&format!(
                "    {} v{} at {}{}\n",
                b.name, version, b.path, auto
            ));
        }
    }
    out.push('\n');

    // Steps
    for step in &report.steps {
        let tag = match step.status {
            StepStatus::Pass => "PASS",
            StepStatus::Fail => "FAIL",
            StepStatus::Skip => "SKIP",
        };
        out.push_str(&format!(
            "  [{}] {} ({}ms) - {}\n",
            tag, step.name, step.duration_ms, step.message
        ));
    }

    // Profile
    if let Some(ref p) = report.profile {
        out.push('\n');
        out.push_str(&format!("  Profile '{}':\n", p.name));
        out.push_str(&format!("    exists: {}\n", p.exists));
        out.push_str(&format!("    has_cookies: {}\n", p.has_cookies));
        out.push_str(&format!("    session_valid: {}\n", p.session_valid));
    }

    // Verbose: platform + timestamp
    if verbose {
        out.push('\n');
        out.push_str(&format!("  Platform: {}\n", report.platform));
        out.push_str(&format!("  Timestamp: {}\n", report.timestamp));
    }

    // Overall
    out.push('\n');
    if report.overall {
        out.push_str("  ALL CHECKS PASSED\n");
    } else {
        out.push_str("  SOME CHECKS FAILED\n");
    }

    out
}

fn current_platform() -> String {
    format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH)
}
