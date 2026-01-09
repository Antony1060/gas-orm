mod binary;
mod commands;
mod error;
mod project;
mod util;

use crate::commands::{CommandDef, CommandImplProvider};
use clap::Parser;
use std::process::ExitCode;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CommandDef,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let handler = cli.command.get_command();

    if let Err(err) = handler.execute().await {
        eprintln!("command failed: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
