mod cli;
mod config;
mod execution;
mod lessons;
mod ollama;
mod progress;

use anyhow::Result;

fn main() -> Result<()> {
    cli::banner::display_banner();
    cli::commands::run()
}
