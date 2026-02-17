//! List all open browser pages (tabs).

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct ListPagesInput {}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: ListPagesInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let pages = manager
        .list_pages_info()
        .await
        .map_err(|e| Error::internal(format!("Failed to list pages: {}", e)))?;

    Ok(json!({
        "pages": pages,
        "count": pages.len()
    }))
}
