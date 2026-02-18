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
            let ch = &config.channels_config;
            Ok(json!({
                "channels": [
                    {"name": "cli", "enabled": ch.cli},
                    {"name": "telegram", "enabled": ch.telegram.is_some()},
                    {"name": "discord", "enabled": ch.discord.is_some()},
                    {"name": "slack", "enabled": ch.slack.is_some()},
                    {"name": "webhook", "enabled": ch.webhook.is_some()},
                    {"name": "imessage", "enabled": ch.imessage.is_some()},
                    {"name": "matrix", "enabled": ch.matrix.is_some()},
                    {"name": "whatsapp", "enabled": ch.whatsapp.is_some()},
                    {"name": "email", "enabled": ch.email.is_some()},
                    {"name": "irc", "enabled": ch.irc.is_some()},
                    {"name": "lark", "enabled": ch.lark.is_some()},
                    {"name": "dingtalk", "enabled": ch.dingtalk.is_some()},
                    {"name": "activity", "enabled": ch.activity.is_some()},
                ]
            }))
        })
        .await
}
