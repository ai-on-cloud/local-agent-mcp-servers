//! Click an element by CSS selector.

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct ClickInput {
    /// CSS selector of the element to click
    #[validate(length(min = 1))]
    #[schemars(description = "CSS selector of the element to click")]
    pub selector: String,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: ClickInput,
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

    element
        .click()
        .await
        .map_err(|e| Error::internal(format!("Click failed on '{}': {}", input.selector, e)))?;

    Ok(json!({
        "status": "clicked",
        "selector": input.selector
    }))
}
