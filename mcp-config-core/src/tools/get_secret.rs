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
    /// Dotted path to the secret field (e.g. "api_key", "channels_config.telegram.bot_token")
    #[validate(length(min = 1))]
    #[schemars(
        description = "Dotted path to the secret field (e.g. \"api_key\", \"channels_config.telegram.bot_token\")"
    )]
    pub path: String,
}

pub async fn execute(
    manager: &Arc<ConfigManager>,
    input: Input,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let path = normalize_path(&input.path);

    manager
        .read(|config| {
            // Serialize entire config to JSON, then navigate the path
            let config_value = serde_json::to_value(config)
                .map_err(|e| Error::internal(format!("Failed to serialize config: {}", e)))?;

            let mut current = &config_value;
            for part in path.split('.') {
                current = current.get(part).ok_or_else(|| {
                    Error::validation(format!(
                        "Path '{}' not found at segment '{}'",
                        input.path, part
                    ))
                })?;
            }

            let encrypted_value = current.as_str().ok_or_else(|| {
                Error::validation(format!(
                    "Value at path '{}' is not a string (type: {})",
                    input.path,
                    value_type_name(current)
                ))
            })?;

            let store = zeroclaw::security::SecretStore::new(
                config
                    .config_path
                    .parent()
                    .unwrap_or(std::path::Path::new(".")),
                config.secrets.encrypt,
            );

            let plaintext = store
                .decrypt(encrypted_value)
                .map_err(|e| Error::internal(format!("Failed to decrypt secret: {}", e)))?;

            Ok(json!({
                "path": input.path,
                "value": plaintext,
            }))
        })
        .await
}

fn normalize_path(path: &str) -> String {
    // Replace "channels." prefix with "channels_config."
    if path == "channels" || path.starts_with("channels.") {
        path.replacen("channels", "channels_config", 1)
    } else {
        path.to_string()
    }
}

fn value_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
