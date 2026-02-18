use std::path::PathBuf;

use clap::Parser;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

use conductor_dashboard::mcp::ConductorService;

/// Conductor MCP Server — read-only access to track data via Model Context Protocol.
#[derive(Parser, Debug)]
#[command(name = "conductor-mcp", version, about)]
struct Cli {
    /// Path to the conductor directory
    #[arg(long, default_value = "./conductor")]
    conductor_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr (MCP uses stdio for JSON-RPC)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("conductor_dashboard=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let cli = Cli::parse();

    // Validate conductor directory
    if !cli.conductor_dir.join("tracks.md").exists() {
        anyhow::bail!("tracks.md not found in {}", cli.conductor_dir.display());
    }

    tracing::info!(
        conductor_dir = %cli.conductor_dir.display(),
        "Starting Conductor MCP server"
    );

    let service = ConductorService::new(&cli.conductor_dir)?;

    let server = service.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    server.waiting().await?;

    Ok(())
}
