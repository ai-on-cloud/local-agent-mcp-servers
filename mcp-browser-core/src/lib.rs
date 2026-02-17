//! Browser automation MCP server core library.
//!
//! Provides `build_server()` which constructs a fully-configured MCP `Server`
//! with browser automation tools, ready to be served over HTTP.

pub mod browser;
pub mod profile;
pub mod resources;
pub mod tools;

use browser::{BrowserManager, BrowserManagerConfig};
use pmcp::types::{ServerCapabilities, ToolCapabilities};
use pmcp::Server;
use profile::ProfileManager;
use std::sync::Arc;

/// Build a fully-configured MCP server with browser automation capabilities.
pub fn build_server(config: BrowserManagerConfig) -> pmcp::Result<Server> {
    let profile_manager =
        Arc::new(ProfileManager::new().map_err(|e| pmcp::Error::internal(e.to_string()))?);

    let manager = Arc::new(BrowserManager::new(config, profile_manager));

    let builder = Server::builder()
        .name("browser")
        .version(env!("CARGO_PKG_VERSION"))
        .capabilities(ServerCapabilities {
            tools: Some(ToolCapabilities {
                list_changed: Some(true),
            }),
            ..Default::default()
        });

    // Register browser tools
    let builder = tools::register_tools(builder, manager.clone());

    // Register resource-like tools (get_dom, get_url)
    let builder = resources::register_resources(builder, manager);

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_server() {
        let server = build_server(BrowserManagerConfig::default());
        assert!(server.is_ok());
    }
}
