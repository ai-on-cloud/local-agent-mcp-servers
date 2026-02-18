use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "config-server", about = "Zeroclaw Config MCP Server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Start the MCP server (default)
    Serve(ServeArgs),
}

#[derive(Parser)]
struct ServeArgs {
    #[clap(flatten)]
    server: server_common::CliArgs,

    /// Path to config.toml (defaults to ~/.zeroclaw/config.toml)
    #[clap(long)]
    config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let args = match cli.command {
        Some(Command::Serve(args)) => args,
        None => ServeArgs::parse_from(["config-server", "serve"]),
    };

    let server = mcp_config_core::build_server(args.config_path)?;
    server_common::run_http(server, &args.server).await
}
