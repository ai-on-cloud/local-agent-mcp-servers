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
        .read(|config| {
            let servers: Vec<serde_json::Value> = config
                .mcp_servers
                .iter()
                .map(|s| {
                    json!({
                        "name": s.name,
                        "transport": format!("{:?}", s.transport).to_lowercase(),
                        "enabled": s.enabled,
                        "url": s.url,
                        "command": s.command,
                    })
                })
                .collect();

            Ok(json!({ "mcp_servers": servers }))
        })
        .await
}
