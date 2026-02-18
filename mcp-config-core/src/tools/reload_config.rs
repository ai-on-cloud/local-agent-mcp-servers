use crate::manager::ConfigManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct Input {}

pub async fn execute(
    manager: &Arc<ConfigManager>,
    _input: Input,
) -> Result<serde_json::Value, Error> {
    manager
        .reload()
        .await
        .map_err(|e| Error::internal(format!("Failed to reload config: {}", e)))?;

    Ok(json!({
        "status": "reloaded",
    }))
}
