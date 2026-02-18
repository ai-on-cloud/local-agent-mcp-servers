use crate::manager::ConfigManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;
use zeroclaw::config::schema::McpServerConfig;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct Input {
    /// MCP server configuration as JSON
    #[schemars(
        description = "MCP server configuration as JSON. Must include at least a 'name' field."
    )]
    pub server: serde_json::Value,
}

pub async fn execute(
    manager: &Arc<ConfigManager>,
    input: Input,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let new_server: McpServerConfig = serde_json::from_value(input.server.clone())
        .map_err(|e| Error::validation(format!("Invalid MCP server config: {}", e)))?;

    if new_server.name.is_empty() {
        return Err(Error::validation(
            "MCP server name cannot be empty".to_string(),
        ));
    }

    let name = new_server.name.clone();

    let action = manager
        .write(|config| {
            // Upsert: find by name, replace if found, push if not
            if let Some(existing) = config.mcp_servers.iter_mut().find(|s| s.name == name) {
                *existing = new_server;
                "replaced"
            } else {
                config.mcp_servers.push(new_server);
                "added"
            }
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to save config: {}", e)))?;

    Ok(json!({
        "status": action,
        "name": name,
    }))
}
