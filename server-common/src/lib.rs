//! Shared HTTP bootstrap for all MCP servers in this workspace.
//!
//! Binary servers call `run_http()` with their configured server (~6 LOC).

use pmcp::server::streamable_http_server::{StreamableHttpServer, StreamableHttpServerConfig};
use pmcp::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// CLI arguments shared across all MCP servers.
#[derive(Debug, Clone, clap::Args)]
pub struct CliArgs {
    /// Host to bind to
    #[clap(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port to bind to
    #[clap(long, default_value = "3100")]
    pub port: u16,
}

/// Run an MCP server over Streamable HTTP transport.
///
/// Initializes tracing, binds to the given host:port, and starts the server.
pub async fn run_http(server: Server, args: &CliArgs) -> anyhow::Result<()> {
    init_logging();

    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    tracing::info!(host = %args.host, port = args.port, "Starting MCP HTTP server");

    let server = Arc::new(Mutex::new(server));

    let config = StreamableHttpServerConfig {
        session_id_generator: None,
        enable_json_response: true,
        event_store: None,
        on_session_initialized: None,
        on_session_closed: None,
        http_middleware: None,
    };

    let http_server = StreamableHttpServer::with_config(addr, server, config);
    let (_bound_addr, server_handle) = http_server.start().await?;

    tracing::info!("MCP server listening on http://{}:{}/mcp", args.host, args.port);

    server_handle.await?;

    Ok(())
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_defaults() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[clap(flatten)]
            server: CliArgs,
        }

        let cli = TestCli::parse_from(["test"]);
        assert_eq!(cli.server.host, "127.0.0.1");
        assert_eq!(cli.server.port, 3100);
    }

    #[test]
    fn test_cli_args_custom() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[clap(flatten)]
            server: CliArgs,
        }

        let cli = TestCli::parse_from(["test", "--host", "0.0.0.0", "--port", "8080"]);
        assert_eq!(cli.server.host, "0.0.0.0");
        assert_eq!(cli.server.port, 8080);
    }
}
