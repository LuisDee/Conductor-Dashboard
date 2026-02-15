mod app;
mod event;
pub mod model;
pub mod parser;
mod theme;

use std::io::stdout;
use std::path::PathBuf;

use clap::Parser;
use crossterm::execute;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use crate::model::FilterMode;

/// Conductor Dashboard — live terminal dashboard for Conductor track progress.
#[derive(Parser, Debug)]
#[command(name = "conductor-dashboard", version, about)]
struct Cli {
    /// Path to the conductor directory
    #[arg(long, default_value = "./conductor")]
    conductor_dir: PathBuf,

    /// Disable file watching (static mode)
    #[arg(long)]
    no_watch: bool,

    /// Initial filter mode
    #[arg(long, default_value = "all")]
    filter: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    // Validate conductor directory
    if !cli.conductor_dir.join("tracks.md").exists() {
        eprintln!(
            "Error: tracks.md not found in {}",
            cli.conductor_dir.display()
        );
        std::process::exit(1);
    }

    // Set up logging to file (we own the terminal)
    let log_dir = std::env::var("CONDUCTOR_DASHBOARD_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("conductor-dashboard"));
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "dashboard.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("conductor_dashboard=info".parse()?),
        )
        .init();

    let initial_filter = match cli.filter.to_lowercase().as_str() {
        "active" => FilterMode::Active,
        "blocked" => FilterMode::Blocked,
        "complete" => FilterMode::Complete,
        _ => FilterMode::All,
    };

    // Install panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture);
        ratatui::restore();
        original_hook(panic_info);
    }));

    // Set up terminal with mouse capture enabled
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = ratatui::init();

    // Run the app
    let mut app = app::App::new(cli.conductor_dir, cli.no_watch, initial_filter)?;
    let result = app.run(&mut terminal).await;

    // Restore terminal — disable mouse capture before restoring
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    ratatui::restore();

    result
}
