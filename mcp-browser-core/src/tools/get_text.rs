//! Get text content of an element.

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct GetTextInput {
    /// CSS selector of the element to get text from
    #[validate(length(min = 1))]
    #[schemars(description = "CSS selector of the element to get text from")]
    pub selector: String,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: GetTextInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .page()
        .await
        .map_err(|e| Error::internal(format!("Browser error: {}", e)))?;

    let element = page
        .find_element(&input.selector)
        .await
        .map_err(|e| Error::internal(format!("Element not found '{}': {}", input.selector, e)))?;

    let text = element
        .inner_text()
        .await
        .map_err(|e| Error::internal(format!("Failed to get text from '{}': {}", input.selector, e)))?
        .unwrap_or_default();

    Ok(json!({
        "text": text,
        "selector": input.selector
    }))
}
