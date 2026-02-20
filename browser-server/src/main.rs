//! Browser Automation MCP Server binary.
//!
//! Provides two subcommands:
//! - `serve` (default): Start the MCP server over Streamable HTTP
//! - `setup-login`: Open browser for manual login, save profile for reuse

use clap::{Parser, Subcommand};
use mcp_browser_core::browser::{BrowserManager, BrowserManagerConfig};
use mcp_browser_core::profile::{CreateOpts, ProfileManager};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "browser-server", about = "Browser Automation MCP Server")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start the MCP server (default when no subcommand given)
    Serve(ServeArgs),

    /// Open browser for manual login, save profile for reuse
    SetupLogin(SetupLoginArgs),
}

#[derive(Parser)]
struct ServeArgs {
    #[clap(flatten)]
    server: server_common::CliArgs,

    /// Custom Chrome/Edge binary path
    #[clap(long)]
    browser_path: Option<String>,

    /// Connect to already-running browser via CDP URL
    #[clap(long)]
    cdp_url: Option<String>,

    /// Run browser in headless mode
    #[clap(long, default_value = "true")]
    headless: bool,

    /// Named profile to use for session persistence
    #[clap(long)]
    profile: Option<String>,
}

#[derive(Parser)]
struct SetupLoginArgs {
    /// Profile name to create or reuse
    #[clap(long)]
    profile: String,

    /// URL to navigate to for login
    #[clap(long)]
    url: String,

    /// Timeout in seconds to wait for user to complete login
    #[clap(long, default_value = "300")]
    timeout_secs: u64,

    /// Description for this profile
    #[clap(long)]
    description: Option<String>,

    /// Notes about what login is needed
    #[clap(long)]
    login_notes: Option<String>,

    /// Custom Chrome/Edge binary path
    #[clap(long)]
    browser_path: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Command::Serve(_)) => {
            // Extract serve args (default or explicit)
            let args = match cli.command {
                Some(Command::Serve(args)) => args,
                _ => ServeArgs::parse_from(["browser-server", "serve"]),
            };
            run_serve(args).await
        }
        Some(Command::SetupLogin(args)) => run_setup_login(args).await,
    }
}

async fn run_serve(args: ServeArgs) -> anyhow::Result<()> {
    let config = BrowserManagerConfig {
        browser_path: args.browser_path,
        cdp_url: args.cdp_url,
        headless: args.headless,
        window_size: (1280, 720),
        profile: args.profile,
    };

    let (server, manager) = mcp_browser_core::build_server(config)?;

    tokio::select! {
        result = server_common::run_http(server, &args.server) => result,
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Ctrl+C received — shutting down browser");
            manager.shutdown().await;
            Ok(())
        }
    }
}

async fn run_setup_login(args: SetupLoginArgs) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let profile_manager = Arc::new(ProfileManager::new()?);

    // Create or reuse profile
    let _profile = profile_manager.get_or_create_profile(
        &args.profile,
        CreateOpts {
            description: args
                .description
                .unwrap_or_else(|| format!("Login profile for {}", args.url)),
            requires_human_login: true,
            login_notes: args
                .login_notes
                .unwrap_or_else(|| format!("Log in at {}", args.url)),
            ..Default::default()
        },
    )?;

    tracing::info!(
        profile = %args.profile,
        url = %args.url,
        "Launching browser for manual login"
    );

    // Launch non-headless browser pointed at the login URL
    let _browser = BrowserManager::launch_for_login(
        profile_manager.clone(),
        &args.profile,
        &args.url,
        args.browser_path,
    )
    .await?;

    println!();
    println!("Browser opened at: {}", args.url);
    println!(
        "Please log in. Press Enter when done (or wait {}s)...",
        args.timeout_secs
    );
    println!();

    // Wait for user input or timeout
    let timeout = tokio::time::Duration::from_secs(args.timeout_secs);
    let stdin_future = tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)
    });

    tokio::select! {
        _ = stdin_future => {}
        _ = tokio::time::sleep(timeout) => {
            println!("Timeout reached.");
        }
    }

    // Update profile metadata
    profile_manager.touch_profile(&args.profile)?;

    println!();
    println!("Profile '{}' saved.", args.profile);
    println!(
        "Use --profile {} to reuse this session.",
        args.profile
    );

    Ok(())
}
