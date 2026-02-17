//! Switch the active browser page (tab) by index.

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct SelectPageInput {
    /// Index of the page to switch to (from list_pages)
    #[schemars(description = "Index of the page to switch to (use list_pages to see available indices)")]
    pub index: usize,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: SelectPageInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .select_page(input.index)
        .await
        .map_err(|e| Error::internal(format!("Failed to select page: {}", e)))?;

    let url = page
        .url()
        .await
        .map_err(|e| Error::internal(format!("Failed to get URL: {}", e)))?
        .unwrap_or_default()
        .to_string();

    Ok(json!({
        "status": "selected",
        "index": input.index,
        "url": url
    }))
}
