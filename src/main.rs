mod cli;
mod docker;
mod github;
mod parser;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use cli::Cli;
use std::process::exit;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.image.is_empty() {
        let _ = Cli::command().print_help();
        exit(1);
    }

    cli.run().await?;

    Ok(())
}
