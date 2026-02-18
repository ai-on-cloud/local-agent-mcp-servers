pub mod add_mcp_server;
pub mod get_channel;
pub mod get_provider;
pub mod get_secret;
pub mod get_section;
pub mod list_channels;
pub mod list_mcp_servers;
pub mod list_sections;
pub mod reload_config;
pub mod remove_channel;
pub mod remove_mcp_server;
pub mod set_channel;
pub mod set_provider;
pub mod set_secret;
pub mod set_section;

use crate::manager::ConfigManager;
use pmcp::TypedTool;
use std::sync::Arc;
use zeroclaw::security::SecretStore;

/// Field names that should be auto-encrypted when writing and masked when reading.
pub const SECRET_FIELD_NAMES: &[&str] = &[
    "bot_token",
    "api_key",
    "access_token",
    "app_secret",
    "client_secret",
    "token",
    "auth_token",
    "server_password",
    "nickserv_password",
    "sasl_password",
    "encrypt_key",
    "verification_token",
    "secret",
];

/// Recursively mask encrypted values in a JSON value tree.
pub fn mask_secrets(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            if SecretStore::is_encrypted(s) {
                *s = "[encrypted]".to_string();
            }
        }
        serde_json::Value::Object(map) => {
            for val in map.values_mut() {
                mask_secrets(val);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                mask_secrets(item);
            }
        }
        _ => {}
    }
}

/// Mask a single string value — returns "[encrypted]" if encrypted, otherwise the original.
pub fn mask_secret_string(s: &str) -> String {
    if SecretStore::is_encrypted(s) {
        "[encrypted]".to_string()
    } else {
        s.to_string()
    }
}

pub fn register_tools(
    builder: pmcp::ServerBuilder,
    manager: Arc<ConfigManager>,
) -> pmcp::ServerBuilder {
    // --- Section-level CRUD ---

    let m = manager.clone();
    let builder = builder.tool(
        "list_sections",
        TypedTool::new(
            "list_sections",
            move |input: list_sections::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { list_sections::execute(&m, input).await })
            },
        )
        .with_description("List all top-level config sections with brief descriptions.")
        .read_only(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "get_section",
        TypedTool::new(
            "get_section",
            move |input: get_section::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { get_section::execute(&m, input).await })
            },
        )
        .with_description(
            "Read a config section by name (e.g. \"memory\", \"gateway\", \"autonomy\", \"channels\"). Masks encrypted values.",
        )
        .read_only(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "set_section",
        TypedTool::new(
            "set_section",
            move |input: set_section::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { set_section::execute(&m, input).await })
            },
        )
        .with_description(
            "Write an entire config section. Deserializes JSON into the section's Rust type for validation.",
        )
        .idempotent(),
    );

    // --- Provider management ---

    let m = manager.clone();
    let builder = builder.tool(
        "get_provider",
        TypedTool::new(
            "get_provider",
            move |input: get_provider::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { get_provider::execute(&m, input).await })
            },
        )
        .with_description(
            "Get current default_provider, default_model, api_key status (masked), and model_routes.",
        )
        .read_only(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "set_provider",
        TypedTool::new(
            "set_provider",
            move |input: set_provider::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { set_provider::execute(&m, input).await })
            },
        )
        .with_description(
            "Set default_provider, default_model, temperature. Optionally set api_key (auto-encrypted).",
        )
        .idempotent(),
    );

    // --- Channel management ---

    let m = manager.clone();
    let builder = builder.tool(
        "list_channels",
        TypedTool::new(
            "list_channels",
            move |input: list_channels::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { list_channels::execute(&m, input).await })
            },
        )
        .with_description("List all channels with enabled/disabled status.")
        .read_only(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "get_channel",
        TypedTool::new("get_channel", move |input: get_channel::Input, _extra| {
            let m = m.clone();
            Box::pin(async move { get_channel::execute(&m, input).await })
        })
        .with_description(
            "Get config for a specific channel (telegram/discord/slack/etc). Masks secrets.",
        )
        .read_only(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "set_channel",
        TypedTool::new("set_channel", move |input: set_channel::Input, _extra| {
            let m = m.clone();
            Box::pin(async move { set_channel::execute(&m, input).await })
        })
        .with_description("Enable/configure a channel. Auto-encrypts token/secret fields.")
        .idempotent(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "remove_channel",
        TypedTool::new(
            "remove_channel",
            move |input: remove_channel::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { remove_channel::execute(&m, input).await })
            },
        )
        .with_description("Disable a channel (remove its configuration).")
        .destructive(),
    );

    // --- MCP server management ---

    let m = manager.clone();
    let builder = builder.tool(
        "list_mcp_servers",
        TypedTool::new(
            "list_mcp_servers",
            move |input: list_mcp_servers::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { list_mcp_servers::execute(&m, input).await })
            },
        )
        .with_description("List configured MCP servers with name, transport, and enabled status.")
        .read_only(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "add_mcp_server",
        TypedTool::new(
            "add_mcp_server",
            move |input: add_mcp_server::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { add_mcp_server::execute(&m, input).await })
            },
        )
        .with_description("Add or update an MCP server by name (upsert).")
        .idempotent(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "remove_mcp_server",
        TypedTool::new(
            "remove_mcp_server",
            move |input: remove_mcp_server::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { remove_mcp_server::execute(&m, input).await })
            },
        )
        .with_description("Remove an MCP server by name.")
        .destructive(),
    );

    // --- Secrets & utility ---

    let m = manager.clone();
    let builder = builder.tool(
        "get_secret",
        TypedTool::new("get_secret", move |input: get_secret::Input, _extra| {
            let m = m.clone();
            Box::pin(async move { get_secret::execute(&m, input).await })
        })
        .with_description("Decrypt a specific secret value by field path. Returns plaintext.")
        .read_only(),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "set_secret",
        TypedTool::new("set_secret", move |input: set_secret::Input, _extra| {
            let m = m.clone();
            Box::pin(async move { set_secret::execute(&m, input).await })
        })
        .with_description("Encrypt a value and store at a dotted config path.")
        .idempotent(),
    );

    let m = manager;
    let builder = builder.tool(
        "reload_config",
        TypedTool::new(
            "reload_config",
            move |input: reload_config::Input, _extra| {
                let m = m.clone();
                Box::pin(async move { reload_config::execute(&m, input).await })
            },
        )
        .with_description("Re-read config from disk (after manual edits).")
        .idempotent(),
    );

    builder
}
