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
    let sections = manager
        .read(|_config| {
            vec![
                json!({"name": "autonomy", "description": "Autonomy level, allowed commands, forbidden paths, action limits"}),
                json!({"name": "runtime", "description": "Runtime kind (native/docker), Docker container settings"}),
                json!({"name": "reliability", "description": "Provider retries, backoff, fallback chains, API key rotation"}),
                json!({"name": "scheduler", "description": "Built-in scheduler enable, max tasks, concurrency"}),
                json!({"name": "agent", "description": "Agent context settings, tool iterations, history limits"}),
                json!({"name": "model_routes", "description": "Model routing rules — route hint names to provider+model combos"}),
                json!({"name": "heartbeat", "description": "Heartbeat enable and interval"}),
                json!({"name": "channels", "description": "Channel configurations (CLI, Telegram, Discord, Slack, etc.)"}),
                json!({"name": "memory", "description": "Memory backend, auto-save, embeddings, response cache, snapshots"}),
                json!({"name": "tunnel", "description": "Tunnel provider (cloudflare, tailscale, ngrok, custom)"}),
                json!({"name": "gateway", "description": "Gateway host/port, pairing, rate limits"}),
                json!({"name": "composio", "description": "Composio integration for OAuth tools"}),
                json!({"name": "secrets", "description": "Secret encryption settings"}),
                json!({"name": "browser", "description": "Browser automation backend and settings"}),
                json!({"name": "http_request", "description": "HTTP request tool settings and domain allowlist"}),
                json!({"name": "identity", "description": "Identity format (openclaw/aieos)"}),
                json!({"name": "cost", "description": "Cost tracking, daily/monthly limits, per-model pricing"}),
                json!({"name": "peripherals", "description": "Peripheral board configs (STM32, RPi GPIO, etc.)"}),
                json!({"name": "agents", "description": "Delegate agent configurations for multi-agent workflows"}),
                json!({"name": "hardware", "description": "Hardware transport, serial port, probe target"}),
                json!({"name": "mcp_servers", "description": "External MCP server configurations"}),
                json!({"name": "observability", "description": "Observability backend (none/log/prometheus/otel)"}),
            ]
        })
        .await;

    Ok(json!({ "sections": sections }))
}
