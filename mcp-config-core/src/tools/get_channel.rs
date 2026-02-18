use crate::manager::ConfigManager;
use crate::tools::mask_secrets;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct Input {
    /// Channel name (telegram, discord, slack, webhook, imessage, matrix, whatsapp, email, irc, lark, dingtalk, activity)
    #[validate(length(min = 1))]
    #[schemars(description = "Channel name (e.g. \"telegram\", \"discord\", \"slack\")")]
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
        .read(|config| {
            let ch = &config.channels_config;
            let value = match input.channel.as_str() {
                "cli" => Ok(json!({"enabled": ch.cli})),
                "telegram" => to_masked_json(&ch.telegram),
                "discord" => to_masked_json(&ch.discord),
                "slack" => to_masked_json(&ch.slack),
                "webhook" => to_masked_json(&ch.webhook),
                "imessage" => to_masked_json(&ch.imessage),
                "matrix" => to_masked_json(&ch.matrix),
                "whatsapp" => to_masked_json(&ch.whatsapp),
                "email" => to_masked_json(&ch.email),
                "irc" => to_masked_json(&ch.irc),
                "lark" => to_masked_json(&ch.lark),
                "dingtalk" => to_masked_json(&ch.dingtalk),
                "activity" => to_masked_json(&ch.activity),
                _ => Err(Error::validation(format!(
                    "Unknown channel: '{}'. Use list_channels to see available channels.",
                    input.channel
                ))),
            }?;

            Ok(json!({
                "channel": input.channel,
                "configured": !value.is_null(),
                "config": value,
            }))
        })
        .await
}

fn to_masked_json<T: Serialize>(opt: &Option<T>) -> Result<serde_json::Value, Error> {
    match opt {
        Some(v) => {
            let mut val = serde_json::to_value(v)
                .map_err(|e| Error::internal(format!("Serialization failed: {}", e)))?;
            mask_secrets(&mut val);
            Ok(val)
        }
        None => Ok(serde_json::Value::Null),
    }
}
