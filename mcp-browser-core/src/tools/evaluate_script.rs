//! Execute arbitrary JavaScript in the browser page.

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct EvaluateScriptInput {
    /// JavaScript code to execute in the page context
    #[validate(length(min = 1))]
    #[schemars(
        description = "JavaScript expression or code to execute in the page context. The result of the last expression is returned."
    )]
    pub expression: String,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: EvaluateScriptInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .page()
        .await
        .map_err(|e| Error::internal(format!("Browser error: {}", e)))?;

    let result = page
        .evaluate_expression(&input.expression)
        .await
        .map_err(|e| Error::internal(format!("Script evaluation failed: {}", e)))?;

    // Try to extract the result as a JSON value
    match result.into_value::<serde_json::Value>() {
        Ok(value) => Ok(json!({
            "result": value
        })),
        Err(_) => {
            // If we can't deserialize to Value, return null
            Ok(json!({
                "result": null,
                "note": "Expression returned a non-serializable value"
            }))
        }
    }
}
