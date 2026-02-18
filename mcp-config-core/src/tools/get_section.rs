use crate::manager::ConfigManager;
use crate::tools::mask_secrets;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct Input {
    /// Config section name (e.g. "memory", "gateway", "autonomy", "channels")
    #[validate(length(min = 1))]
    #[schemars(
        description = "Config section name (e.g. \"memory\", \"gateway\", \"autonomy\", \"channels\")"
    )]
    pub section: String,
}

pub async fn execute(
    manager: &Arc<ConfigManager>,
    input: Input,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let section_name = normalize_section_name(&input.section);

    manager
        .read(|config| {
            let value = match section_name.as_str() {
                "autonomy" => serde_json::to_value(&config.autonomy),
                "runtime" => serde_json::to_value(&config.runtime),
                "reliability" => serde_json::to_value(&config.reliability),
                "scheduler" => serde_json::to_value(&config.scheduler),
                "agent" => serde_json::to_value(&config.agent),
                "model_routes" => serde_json::to_value(&config.model_routes),
                "heartbeat" => serde_json::to_value(&config.heartbeat),
                "channels_config" => serde_json::to_value(&config.channels_config),
                "memory" => serde_json::to_value(&config.memory),
                "tunnel" => serde_json::to_value(&config.tunnel),
                "gateway" => serde_json::to_value(&config.gateway),
                "composio" => serde_json::to_value(&config.composio),
                "secrets" => serde_json::to_value(&config.secrets),
                "browser" => serde_json::to_value(&config.browser),
                "http_request" => serde_json::to_value(&config.http_request),
                "identity" => serde_json::to_value(&config.identity),
                "cost" => serde_json::to_value(&config.cost),
                "peripherals" => serde_json::to_value(&config.peripherals),
                "agents" => serde_json::to_value(&config.agents),
                "hardware" => serde_json::to_value(&config.hardware),
                "mcp_servers" => serde_json::to_value(&config.mcp_servers),
                "observability" => serde_json::to_value(&config.observability),
                _ => {
                    return Err(Error::validation(format!(
                        "Unknown section: '{}'. Use list_sections to see available sections.",
                        input.section
                    )));
                }
            };

            match value {
                Ok(mut v) => {
                    mask_secrets(&mut v);
                    Ok(v)
                }
                Err(e) => Err(Error::internal(format!(
                    "Failed to serialize section '{}': {}",
                    section_name, e
                ))),
            }
        })
        .await
}

fn normalize_section_name(name: &str) -> String {
    match name {
        "channels" => "channels_config".to_string(),
        other => other.to_string(),
    }
}
