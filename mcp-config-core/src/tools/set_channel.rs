use crate::manager::ConfigManager;
use crate::tools::SECRET_FIELD_NAMES;
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
    /// Channel name (telegram, discord, slack, webhook, imessage, matrix, whatsapp, email, irc, lark, dingtalk, activity)
    #[validate(length(min = 1))]
    #[schemars(description = "Channel name (e.g. \"telegram\", \"discord\", \"slack\")")]
    pub channel: String,

    /// Channel configuration as JSON. Must match the channel's schema.
    #[schemars(description = "Channel configuration as JSON. Must match the channel's schema.")]
    pub config: serde_json::Value,
}

pub async fn execute(
    manager: &Arc<ConfigManager>,
    input: Input,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    manager
        .write(|config| {
            // Auto-encrypt secret fields in the JSON value
            let mut value = input.config.clone();
            let store = SecretStore::new(
                config
                    .config_path
                    .parent()
                    .unwrap_or(std::path::Path::new(".")),
                config.secrets.encrypt,
            );
            encrypt_secret_fields(&mut value, &store);

            let ch = &mut config.channels_config;
            match input.channel.as_str() {
                "cli" => {
                    if let Some(enabled) = value.get("enabled").and_then(|v| v.as_bool()) {
                        ch.cli = enabled;
                    }
                }
                "telegram" => {
                    ch.telegram = Some(deser(&value)?);
                }
                "discord" => {
                    ch.discord = Some(deser(&value)?);
                }
                "slack" => {
                    ch.slack = Some(deser(&value)?);
                }
                "webhook" => {
                    ch.webhook = Some(deser(&value)?);
                }
                "imessage" => {
                    ch.imessage = Some(deser(&value)?);
                }
                "matrix" => {
                    ch.matrix = Some(deser(&value)?);
                }
                "whatsapp" => {
                    ch.whatsapp = Some(deser(&value)?);
                }
                "email" => {
                    ch.email = Some(deser(&value)?);
                }
                "irc" => {
                    ch.irc = Some(deser(&value)?);
                }
                "lark" => {
                    ch.lark = Some(deser(&value)?);
                }
                "dingtalk" => {
                    ch.dingtalk = Some(deser(&value)?);
                }
                "activity" => {
                    ch.activity = Some(deser(&value)?);
                }
                _ => {
                    return Err(Error::validation(format!(
                        "Unknown channel: '{}'. Use list_channels to see available channels.",
                        input.channel
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
        "channel": input.channel,
    }))
}

fn deser<T: serde::de::DeserializeOwned>(value: &serde_json::Value) -> Result<T, Error> {
    serde_json::from_value(value.clone())
        .map_err(|e| Error::validation(format!("Invalid channel config: {}", e)))
}

fn encrypt_secret_fields(value: &mut serde_json::Value, store: &SecretStore) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if SECRET_FIELD_NAMES.contains(&key.as_str()) {
                    if let Some(s) = val.as_str() {
                        if !SecretStore::is_encrypted(s) && !s.is_empty() {
                            if let Ok(encrypted) = store.encrypt(s) {
                                *val = serde_json::Value::String(encrypted);
                            }
                        }
                    }
                } else {
                    encrypt_secret_fields(val, store);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                encrypt_secret_fields(item, store);
            }
        }
        _ => {}
    }
}
