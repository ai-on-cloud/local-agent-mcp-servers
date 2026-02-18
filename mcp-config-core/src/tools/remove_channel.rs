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
    /// Channel name to disable
    #[validate(length(min = 1))]
    #[schemars(description = "Channel name to disable (e.g. \"telegram\", \"discord\")")]
    pub channel: String,
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
            let ch = &mut config.channels_config;
            match input.channel.as_str() {
                "cli" => {
                    ch.cli = false;
                }
                "telegram" => {
                    ch.telegram = None;
                }
                "discord" => {
                    ch.discord = None;
                }
                "slack" => {
                    ch.slack = None;
                }
                "webhook" => {
                    ch.webhook = None;
                }
                "imessage" => {
                    ch.imessage = None;
                }
                "matrix" => {
                    ch.matrix = None;
                }
                "whatsapp" => {
                    ch.whatsapp = None;
                }
                "email" => {
                    ch.email = None;
                }
                "irc" => {
                    ch.irc = None;
                }
                "lark" => {
                    ch.lark = None;
                }
                "dingtalk" => {
                    ch.dingtalk = None;
                }
                "activity" => {
                    ch.activity = None;
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
        "status": "removed",
        "channel": input.channel,
    }))
}
