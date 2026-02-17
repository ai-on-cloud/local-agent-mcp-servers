//! Wait for a selector to appear or a timeout.

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

fn default_timeout_ms() -> u64 {
    10000
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct WaitInput {
    /// CSS selector to wait for (if omitted, waits for timeout_ms)
    #[schemars(description = "CSS selector to wait for (optional)")]
    pub selector: Option<String>,

    /// Maximum time to wait in milliseconds
    #[serde(default = "default_timeout_ms")]
    #[validate(range(min = 100, max = 120000))]
    #[schemars(description = "Maximum time to wait in milliseconds (default: 10000)")]
    pub timeout_ms: u64,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: WaitInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .page()
        .await
        .map_err(|e| Error::internal(format!("Browser error: {}", e)))?;

    if let Some(ref selector) = input.selector {
        // Poll for the element with timeout
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(input.timeout_ms);

        loop {
            match page.find_element(selector).await {
                Ok(_) => {
                    let elapsed = start.elapsed().as_millis();
                    return Ok(json!({
                        "status": "ready",
                        "selector": selector,
                        "elapsed_ms": elapsed
                    }));
                }
                Err(_) if start.elapsed() < timeout => {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
                Err(e) => {
                    return Err(Error::internal(format!(
                        "Timeout waiting for '{}' after {}ms: {}",
                        selector, input.timeout_ms, e
                    )));
                }
            }
        }
    } else {
        // Just wait for the specified time
        tokio::time::sleep(std::time::Duration::from_millis(input.timeout_ms)).await;
        Ok(json!({
            "status": "ready",
            "waited_ms": input.timeout_ms
        }))
    }
}
