//! Navigate to a URL.

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

fn default_timeout_ms() -> u64 {
    30000
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct NavigateInput {
    /// URL to navigate to
    #[schemars(description = "The URL to navigate to")]
    pub url: String,

    /// Navigation timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    #[validate(range(min = 1000, max = 120000))]
    #[schemars(description = "Navigation timeout in milliseconds (default: 30000)")]
    pub timeout_ms: u64,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: NavigateInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .page()
        .await
        .map_err(|e| Error::internal(format!("Browser error: {}", e)))?;

    page.goto(&input.url)
        .await
        .map_err(|e| Error::internal(format!("Navigation failed: {}", e)))?;

    // Get the final URL after any redirects
    let final_url = page
        .url()
        .await
        .map_err(|e| Error::internal(format!("Failed to get URL: {}", e)))?
        .unwrap_or_default()
        .to_string();

    Ok(json!({
        "url": final_url,
        "status": "navigated"
    }))
}
