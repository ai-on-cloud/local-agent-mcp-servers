use crate::manager::ConfigManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;
use zeroclaw::security::SecretStore;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct Input {
    /// Dotted path to the secret field (e.g. "api_key", "channels_config.telegram.bot_token")
    #[validate(length(min = 1))]
    #[schemars(
        description = "Dotted path to the secret field (e.g. \"api_key\", \"channels_config.telegram.bot_token\")"
    )]
    pub path: String,

    /// Plaintext value to encrypt and store
    #[validate(length(min = 1))]
    #[schemars(description = "Plaintext value to encrypt and store")]
    pub value: String,
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
        .write(|config| {
            let store = SecretStore::new(
                config
                    .config_path
                    .parent()
                    .unwrap_or(std::path::Path::new(".")),
                config.secrets.encrypt,
            );

            let encrypted = store
                .encrypt(&input.value)
                .map_err(|e| Error::internal(format!("Failed to encrypt value: {}", e)))?;

            // Serialize config to JSON, set the value, deserialize back
            let mut config_value = serde_json::to_value(&*config)
                .map_err(|e| Error::internal(format!("Failed to serialize config: {}", e)))?;

            set_nested_value(
                &mut config_value,
                &path,
                serde_json::Value::String(encrypted),
            )
            .map_err(|e| {
                Error::validation(format!("Failed to set path '{}': {}", input.path, e))
            })?;

            // Deserialize back, preserving computed fields
            let config_path = config.config_path.clone();
            let workspace_dir = config.workspace_dir.clone();
            *config = serde_json::from_value(config_value)
                .map_err(|e| Error::internal(format!("Failed to update config: {}", e)))?;
            config.config_path = config_path;
            config.workspace_dir = workspace_dir;

            Ok(())
        })
        .await
        .map_err(|e| Error::internal(format!("Failed to save config: {}", e)))?
        .map_err(|e: Error| e)?;

    Ok(json!({
        "status": "encrypted_and_stored",
        "path": input.path,
    }))
}

fn normalize_path(path: &str) -> String {
    if path == "channels" || path.starts_with("channels.") {
        path.replacen("channels", "channels_config", 1)
    } else {
        path.to_string()
    }
}

fn set_nested_value(
    root: &mut serde_json::Value,
    path: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last segment: set the value
            match current {
                serde_json::Value::Object(map) => {
                    map.insert(part.to_string(), value);
                    return Ok(());
                }
                _ => return Err(format!("Path segment '{}' is not an object", part)),
            }
        } else {
            current = current
                .get_mut(*part)
                .ok_or_else(|| format!("Path segment '{}' not found", part))?;
        }
    }

    Err("Empty path".to_string())
}
