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
    /// Default provider name (e.g. "openrouter", "anthropic", "ollama")
    #[serde(default)]
    #[schemars(
        description = "Default provider name (e.g. \"openrouter\", \"anthropic\", \"ollama\")"
    )]
    pub default_provider: Option<String>,

    /// Default model name
    #[serde(default)]
    #[schemars(description = "Default model name (e.g. \"anthropic/claude-sonnet-4\")")]
    pub default_model: Option<String>,

    /// Default temperature (0.0-2.0)
    #[serde(default)]
    #[schemars(description = "Default temperature (0.0-2.0)")]
    pub temperature: Option<f64>,

    /// API key (will be auto-encrypted if secrets.encrypt is enabled)
    #[serde(default)]
    #[schemars(description = "API key (will be auto-encrypted if secrets.encrypt is enabled)")]
    pub api_key: Option<String>,
}

pub async fn execute(
    manager: &Arc<ConfigManager>,
    input: Input,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    if let Some(temp) = input.temperature {
        if !(0.0..=2.0).contains(&temp) {
            return Err(Error::validation(
                "Temperature must be between 0.0 and 2.0".to_string(),
            ));
        }
    }

    manager
        .write(|config| {
            let mut changed = Vec::new();

            if let Some(ref provider) = input.default_provider {
                config.default_provider = Some(provider.clone());
                changed.push("default_provider");
            }
            if let Some(ref model) = input.default_model {
                config.default_model = Some(model.clone());
                changed.push("default_model");
            }
            if let Some(temp) = input.temperature {
                config.default_temperature = temp;
                changed.push("default_temperature");
            }
            if let Some(ref key) = input.api_key {
                let store = SecretStore::new(
                    config
                        .config_path
                        .parent()
                        .unwrap_or(std::path::Path::new(".")),
                    config.secrets.encrypt,
                );
                if !SecretStore::is_encrypted(key) {
                    match store.encrypt(key) {
                        Ok(encrypted) => config.api_key = Some(encrypted),
                        Err(e) => {
                            tracing::warn!("Failed to encrypt API key, storing as-is: {}", e);
                            config.api_key = Some(key.clone());
                        }
                    }
                } else {
                    config.api_key = Some(key.clone());
                }
                changed.push("api_key");
            }

            changed
        })
        .await
        .map(|changed| {
            json!({
                "status": "updated",
                "changed_fields": changed,
            })
        })
        .map_err(|e| Error::internal(format!("Failed to save config: {}", e)))
}
