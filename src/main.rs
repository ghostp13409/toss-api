mod cli;
mod core;
mod engine;

mod tui;

use clap::Parser;
use cli::args::CliArgs;
use cli::run_cli;
use tui::run_tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    if let Some(command) = args.command {
        run_cli(command).await?;
    } else {
        run_tui().await?;
    }

    Ok(())
}
