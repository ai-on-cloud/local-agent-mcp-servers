pub mod manager;
pub mod tools;

use manager::ConfigManager;
use pmcp::types::{ServerCapabilities, ToolCapabilities};
use pmcp::Server;
use std::path::PathBuf;
use std::sync::Arc;
use zeroclaw::config::schema::Config;

/// Build a fully-configured MCP server with config management capabilities.
pub fn build_server(config_path: Option<PathBuf>) -> pmcp::Result<Server> {
    let config = if let Some(path) = config_path {
        // Load from explicit path
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| pmcp::Error::internal(format!("Failed to read config: {}", e)))?;
        let mut config: Config = toml::from_str(&contents)
            .map_err(|e| pmcp::Error::internal(format!("Failed to parse config: {}", e)))?;
        config.config_path = path;
        config
    } else {
        Config::load_or_init()
            .map_err(|e| pmcp::Error::internal(format!("Failed to load config: {}", e)))?
    };

    let manager = Arc::new(ConfigManager::new(config));

    let builder = Server::builder()
        .name("zeroclaw-config")
        .version(env!("CARGO_PKG_VERSION"))
        .capabilities(ServerCapabilities {
            tools: Some(ToolCapabilities {
                list_changed: Some(true),
            }),
            ..Default::default()
        });

    let builder = tools::register_tools(builder, manager);

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_server() {
        let server = build_server(None);
        assert!(server.is_ok());
    }
}
