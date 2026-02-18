use crate::manager::ConfigManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct Input {
    /// Name of the MCP server to remove
    #[validate(length(min = 1))]
    #[schemars(description = "Name of the MCP server to remove")]
    pub name: String,
}

pub async fn execute(
    manager: &Arc<ConfigManager>,
    input: Input,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let removed = manager
        .write(|config| {
            let before = config.mcp_servers.len();
            config.mcp_servers.retain(|s| s.name != input.name);
            before != config.mcp_servers.len()
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to save config: {}", e)))?;

    if removed {
        Ok(json!({
            "status": "removed",
            "name": input.name,
        }))
    } else {
        Err(Error::validation(format!(
            "MCP server '{}' not found. Use list_mcp_servers to see configured servers.",
            input.name
        )))
    }
}
