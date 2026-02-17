//! Code Mode for browser automation.
//!
//! Enables script-based browser automation using a safe JavaScript subset.
//! Scripts use the `api` object to call browser operations:
//!
//! ```javascript
//! await api.post("/navigate", { url: "https://example.com/login" });
//! await api.post("/fill", { selector: "#email", value: "user@example.com" });
//! await api.post("/click", { selector: "#submit" });
//! const text = await api.post("/get_text", { selector: ".welcome" });
//! return { message: text };
//! ```
//!
//! ## Available Endpoints
//!
//! | Method | Path | Body | Description |
//! |--------|------|------|-------------|
//! | POST | `/navigate` | `{ url, timeout_ms? }` | Navigate to URL |
//! | POST | `/click` | `{ selector }` | Click element |
//! | POST | `/fill` | `{ selector, value }` | Fill form field |
//! | POST | `/screenshot` | `{ selector?, full_page? }` | Screenshot (base64 PNG) |
//! | POST | `/get_text` | `{ selector }` | Get element text |
//! | POST | `/extract_table` | `{ selector }` | Extract HTML table as JSON |
//! | POST | `/wait` | `{ selector?, timeout_ms? }` | Wait for selector/duration |
//! | POST | `/press_key` | `{ key, selector? }` | Press keyboard key |
//! | POST | `/hover` | `{ selector }` | Hover over element |
//! | POST | `/evaluate` | `{ expression }` | Evaluate JavaScript |
//! | POST | `/new_page` | `{ url }` | Open new tab |
//! | POST | `/select_page` | `{ index }` | Switch tab |
//! | GET | `/dom` | — | Get page DOM |
//! | GET | `/url` | — | Get page URL |
//! | GET | `/pages` | — | List open pages |

use crate::browser::BrowserManager;
use crate::tools;
use mcp_server_common::code_mode::{
    ExecutionConfig, ExecutionError, HttpExecutor, PlanCompiler, PlanExecutor,
};
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// Browser-backed executor for the code mode engine.
///
/// Routes `api.method("/path", body)` calls to `BrowserManager` operations.
/// Each browser tool is exposed as a REST-like endpoint.
pub struct BrowserHttpExecutor {
    manager: Arc<BrowserManager>,
}

impl BrowserHttpExecutor {
    pub fn new(manager: Arc<BrowserManager>) -> Self {
        Self { manager }
    }

    /// Dispatch a POST request to the appropriate browser tool.
    async fn handle_post(
        &self,
        path: &str,
        body: Option<JsonValue>,
    ) -> Result<JsonValue, ExecutionError> {
        let body = body.unwrap_or(JsonValue::Object(serde_json::Map::new()));

        match path {
            "/navigate" => {
                let input: tools::navigate::NavigateInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/navigate: invalid input: {}", e),
                    })?;
                tools::navigate::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/navigate failed: {}", e),
                    })
            }

            "/click" => {
                let input: tools::click::ClickInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/click: invalid input: {}", e),
                    })?;
                tools::click::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/click failed: {}", e),
                    })
            }

            "/fill" => {
                let input: tools::fill::FillInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/fill: invalid input: {}", e),
                    })?;
                tools::fill::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/fill failed: {}", e),
                    })
            }

            "/screenshot" => {
                let input: tools::screenshot::ScreenshotInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/screenshot: invalid input: {}", e),
                    })?;
                tools::screenshot::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/screenshot failed: {}", e),
                    })
            }

            "/get_text" => {
                let input: tools::get_text::GetTextInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/get_text: invalid input: {}", e),
                    })?;
                tools::get_text::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/get_text failed: {}", e),
                    })
            }

            "/extract_table" => {
                let input: tools::extract_table::ExtractTableInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/extract_table: invalid input: {}", e),
                    })?;
                tools::extract_table::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/extract_table failed: {}", e),
                    })
            }

            "/wait" => {
                let input: tools::wait::WaitInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/wait: invalid input: {}", e),
                    })?;
                tools::wait::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/wait failed: {}", e),
                    })
            }

            "/press_key" => {
                let input: tools::press_key::PressKeyInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/press_key: invalid input: {}", e),
                    })?;
                tools::press_key::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/press_key failed: {}", e),
                    })
            }

            "/hover" => {
                let input: tools::hover::HoverInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/hover: invalid input: {}", e),
                    })?;
                tools::hover::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/hover failed: {}", e),
                    })
            }

            "/evaluate" => {
                let input: tools::evaluate_script::EvaluateScriptInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/evaluate: invalid input: {}", e),
                    })?;
                tools::evaluate_script::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/evaluate failed: {}", e),
                    })
            }

            "/handle_dialog" => {
                let input: tools::handle_dialog::HandleDialogInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/handle_dialog: invalid input: {}", e),
                    })?;
                tools::handle_dialog::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/handle_dialog failed: {}", e),
                    })
            }

            "/new_page" => {
                let url = body
                    .get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("about:blank");
                let (idx, _page) = self.manager.create_new_page(url).await.map_err(|e| {
                    ExecutionError::RuntimeError {
                        message: format!("/new_page failed: {}", e),
                    }
                })?;
                Ok(serde_json::json!({ "index": idx, "url": url }))
            }

            "/select_page" => {
                let input: tools::select_page::SelectPageInput =
                    serde_json::from_value(body).map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/select_page: invalid input: {}", e),
                    })?;
                tools::select_page::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/select_page failed: {}", e),
                    })
            }

            _ => Err(ExecutionError::RuntimeError {
                message: format!("Unknown browser endpoint: POST {}", path),
            }),
        }
    }

    /// Dispatch a GET request.
    async fn handle_get(&self, path: &str) -> Result<JsonValue, ExecutionError> {
        match path {
            "/dom" => {
                let page = self.manager.page().await.map_err(|e| {
                    ExecutionError::RuntimeError {
                        message: format!("/dom: browser error: {}", e),
                    }
                })?;
                let html = page.content().await.map_err(|e| {
                    ExecutionError::RuntimeError {
                        message: format!("/dom: failed to get content: {}", e),
                    }
                })?;
                Ok(serde_json::json!({ "dom": html }))
            }

            "/url" => {
                let page = self.manager.page().await.map_err(|e| {
                    ExecutionError::RuntimeError {
                        message: format!("/url: browser error: {}", e),
                    }
                })?;
                let url = page
                    .url()
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/url: failed to get URL: {}", e),
                    })?
                    .unwrap_or_default()
                    .to_string();
                Ok(serde_json::json!({ "url": url }))
            }

            "/pages" => {
                let input = tools::list_pages::ListPagesInput {};
                tools::list_pages::execute(&self.manager, input)
                    .await
                    .map_err(|e| ExecutionError::RuntimeError {
                        message: format!("/pages failed: {}", e),
                    })
            }

            _ => Err(ExecutionError::RuntimeError {
                message: format!("Unknown browser endpoint: GET {}", path),
            }),
        }
    }
}

#[async_trait::async_trait]
impl HttpExecutor for BrowserHttpExecutor {
    async fn execute_request(
        &self,
        method: &str,
        path: &str,
        body: Option<JsonValue>,
    ) -> Result<JsonValue, ExecutionError> {
        match method.to_uppercase().as_str() {
            "GET" => self.handle_get(path).await,
            "POST" => self.handle_post(path, body).await,
            _ => Err(ExecutionError::RuntimeError {
                message: format!("Unsupported HTTP method for browser: {}", method),
            }),
        }
    }
}

/// Validate a browser automation script.
///
/// Parses the JavaScript subset and compiles it to an execution plan.
/// Returns the plan metadata and a simple approval token (code hash).
///
/// Security and policy validation are intentionally minimal for now —
/// the script must parse and compile successfully, that's all.
pub fn validate_script(code: &str) -> Result<ValidationResult, String> {
    let code = code.trim();

    let config = ExecutionConfig {
        max_api_calls: 100,
        timeout_seconds: 60,
        max_loop_iterations: 100,
        ..Default::default()
    };

    let mut compiler = PlanCompiler::with_config(&config);
    let plan = compiler
        .compile_code(code)
        .map_err(|e| format!("Script compilation failed: {}", e))?;

    // Simple token: hex-encoded hash of code
    let token = format!("{:x}", md5_simple(code.as_bytes()));

    Ok(ValidationResult {
        is_valid: true,
        approval_token: token,
        normalized_code: code.to_string(),
        api_call_count: plan.metadata.api_call_count,
        has_mutations: plan.metadata.has_mutations,
        endpoints: plan.metadata.endpoints,
    })
}

/// Execute a validated browser automation script.
///
/// Parses, compiles, and runs the script against the browser via CDP.
/// The approval token must match the one returned by `validate_script`.
pub async fn execute_script(
    manager: Arc<BrowserManager>,
    code: &str,
    approval_token: &str,
    variables: Option<JsonValue>,
) -> Result<JsonValue, String> {
    let code = code.trim();

    // Verify token matches
    let expected_token = format!("{:x}", md5_simple(code.as_bytes()));
    if approval_token != expected_token {
        return Err(
            "Code mismatch: the code sent to execute_code does not match the validated code"
                .to_string(),
        );
    }

    let config = ExecutionConfig {
        max_api_calls: 100,
        timeout_seconds: 60,
        max_loop_iterations: 100,
        ..Default::default()
    };

    // Compile
    let mut compiler = PlanCompiler::with_config(&config);
    let plan = compiler
        .compile_code(code)
        .map_err(|e| format!("Script compilation failed: {}", e))?;

    // Execute
    let http = BrowserHttpExecutor::new(manager);
    let mut executor = PlanExecutor::new(http, config);

    // Bind user-provided variables
    if let Some(JsonValue::Object(vars)) = variables {
        for (key, value) in vars {
            executor.set_variable(key, value);
        }
    }

    let result = executor
        .execute(&plan)
        .await
        .map_err(|e| format!("Script execution failed: {}", e))?;

    Ok(serde_json::json!({
        "result": result.value,
        "api_calls": result.api_calls.len(),
        "execution_time_ms": result.execution_time_ms,
    }))
}

/// Result of script validation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub approval_token: String,
    /// The normalized (trimmed) code that the token was computed from.
    /// Pass this exact string to `execute_code` to avoid code-mismatch errors.
    pub normalized_code: String,
    pub api_call_count: usize,
    pub has_mutations: bool,
    pub endpoints: Vec<String>,
}

/// Simple hash for token generation (djb2 — not cryptographic, intentionally simple).
/// Security note: this is a placeholder. We skip HMAC signing for now per user request.
fn md5_simple(data: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for &byte in data {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_simple_script() {
        let code = r#"
            const page = await api.post("/navigate", { url: "https://example.com" });
            return page;
        "#;
        let result = validate_script(code).unwrap();
        assert!(result.is_valid);
        assert!(!result.approval_token.is_empty());
        assert_eq!(result.api_call_count, 1);
    }

    #[test]
    fn test_validate_multi_step_script() {
        let code = r##"
            await api.post("/navigate", { url: "https://example.com/login" });
            await api.post("/fill", { selector: "#email", value: "user@test.com" });
            await api.post("/fill", { selector: "#password", value: "secret" });
            await api.post("/click", { selector: "#submit" });
            await api.post("/wait", { selector: ".dashboard" });
            const text = await api.post("/get_text", { selector: ".welcome" });
            return { message: text };
        "##;
        let result = validate_script(code).unwrap();
        assert!(result.is_valid);
        assert!(result.api_call_count >= 6);
    }

    #[test]
    fn test_validate_with_conditionals() {
        let code = r##"
            const page = await api.post("/navigate", { url: "https://example.com" });
            const text = await api.post("/get_text", { selector: ".status" });
            if (text.text === "logged_out") {
                await api.post("/click", { selector: "#login" });
            }
            return { status: "done" };
        "##;
        let result = validate_script(code).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_with_loop() {
        let code = r#"
            const items = await api.post("/extract_table", { selector: "table" });
            const results = [];
            for (const row of items.rows.slice(0, 10)) {
                const detail = await api.post("/get_text", { selector: row.link });
                results.push(detail);
            }
            return results;
        "#;
        let result = validate_script(code).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_invalid_syntax() {
        let code = "this is not javascript }{}{";
        let result = validate_script(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_consistency() {
        let code = "const x = await api.get(\"/url\");";
        let r1 = validate_script(code).unwrap();
        let r2 = validate_script(code).unwrap();
        assert_eq!(r1.approval_token, r2.approval_token);
    }
}
