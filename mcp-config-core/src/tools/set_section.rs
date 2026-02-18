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
    /// Config section name (e.g. "memory", "gateway", "autonomy", "channels")
    #[validate(length(min = 1))]
    #[schemars(description = "Config section name")]
    pub section: String,

    /// JSON value to set the section to. Must match the section's schema.
    #[schemars(description = "JSON value to set the section to. Must match the section's schema.")]
    pub value: serde_json::Value,
}

pub async fn execute(
    manager: &Arc<ConfigManager>,
    input: Input,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let section_name = match input.section.as_str() {
        "channels" => "channels_config",
        other => other,
    };

    // Validate by deserializing into the correct type, then assign
    manager
        .write(|config| {
            match section_name {
                "autonomy" => {
                    config.autonomy = deser(&input.value)?;
                }
                "runtime" => {
                    config.runtime = deser(&input.value)?;
                }
                "reliability" => {
                    config.reliability = deser(&input.value)?;
                }
                "scheduler" => {
                    config.scheduler = deser(&input.value)?;
                }
                "agent" => {
                    config.agent = deser(&input.value)?;
                }
                "model_routes" => {
                    config.model_routes = deser(&input.value)?;
                }
                "heartbeat" => {
                    config.heartbeat = deser(&input.value)?;
                }
                "channels_config" => {
                    config.channels_config = deser(&input.value)?;
                }
                "memory" => {
                    config.memory = deser(&input.value)?;
                }
                "tunnel" => {
                    config.tunnel = deser(&input.value)?;
                }
                "gateway" => {
                    config.gateway = deser(&input.value)?;
                }
                "composio" => {
                    config.composio = deser(&input.value)?;
                }
                "secrets" => {
                    config.secrets = deser(&input.value)?;
                }
                "browser" => {
                    config.browser = deser(&input.value)?;
                }
                "http_request" => {
                    config.http_request = deser(&input.value)?;
                }
                "identity" => {
                    config.identity = deser(&input.value)?;
                }
                "cost" => {
                    config.cost = deser(&input.value)?;
                }
                "peripherals" => {
                    config.peripherals = deser(&input.value)?;
                }
                "agents" => {
                    config.agents = deser(&input.value)?;
                }
                "hardware" => {
                    config.hardware = deser(&input.value)?;
                }
                "mcp_servers" => {
                    config.mcp_servers = deser(&input.value)?;
                }
                "observability" => {
                    config.observability = deser(&input.value)?;
                }
                _ => {
                    return Err(Error::validation(format!(
                        "Unknown section: '{}'. Use list_sections to see available sections.",
                        input.section
                    )));
                }
            }
            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to save config: {}", e)))?
        .map_err(|e: Error| e)?;

    Ok(json!({
        "status": "updated",
        "section": input.section
    }))
}

fn deser<T: serde::de::DeserializeOwned>(value: &serde_json::Value) -> Result<T, Error> {
    serde_json::from_value(value.clone())
        .map_err(|e| Error::validation(format!("Invalid value for section: {}", e)))
}
