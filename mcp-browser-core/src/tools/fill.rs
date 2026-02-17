//! Fill a form field.

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct FillInput {
    /// CSS selector of the form field to fill
    #[validate(length(min = 1))]
    #[schemars(description = "CSS selector of the form field to fill")]
    pub selector: String,

    /// Value to type into the field
    #[schemars(description = "Text value to type into the field")]
    pub value: String,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: FillInput,
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

    // Click to focus first, then type
    element
        .click()
        .await
        .map_err(|e| Error::internal(format!("Failed to focus '{}': {}", input.selector, e)))?;

    element
        .type_str(&input.value)
        .await
        .map_err(|e| Error::internal(format!("Failed to type into '{}': {}", input.selector, e)))?;

    Ok(json!({
        "status": "filled",
        "selector": input.selector
    }))
}
