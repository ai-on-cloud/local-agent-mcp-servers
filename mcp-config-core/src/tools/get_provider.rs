use crate::manager::ConfigManager;
use crate::tools::mask_secret_string;
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
            let api_key_status = match &config.api_key {
                Some(k) if k.is_empty() => "not set",
                Some(_) => "set",
                None => "not set",
            };
            let api_key_display = config
                .api_key
                .as_deref()
                .map(mask_secret_string)
                .unwrap_or_else(|| "not set".to_string());

            let model_routes: Vec<serde_json::Value> = config
                .model_routes
                .iter()
                .map(|r| {
                    json!({
                        "hint": r.hint,
                        "provider": r.provider,
                        "model": r.model,
                        "api_key": r.api_key.as_deref().map(mask_secret_string).unwrap_or_else(|| "not set".to_string()),
                    })
                })
                .collect();

            Ok(json!({
                "default_provider": config.default_provider,
                "default_model": config.default_model,
                "default_temperature": config.default_temperature,
                "api_key_status": api_key_status,
                "api_key": api_key_display,
                "model_routes": model_routes,
            }))
        })
        .await
}
