use crate::commands::migrations::info::MigrationInfoCommand;
use crate::commands::migrations::sync::MigrationSyncCommand;
use crate::commands::{Command, CommandImplProvider};

mod info;
mod sync;

#[derive(Debug, clap::Subcommand)]
enum MigrationOperation {
    Info,
    Sync,
}

#[derive(Debug, clap::Parser)]
pub struct MigrationArgs {
    #[arg(long, short = 'p', default_value = ".")]
    project_path: String,
    #[arg(
        long,
        short = 'm',
        default_value = "./migrations",
        help = "relative to project_path"
    )]
    migrations_dir: String,
    #[command(subcommand)]
    operation: MigrationOperation,
}

impl CommandImplProvider for MigrationArgs {
    fn get_command(self) -> Box<dyn Command> {
        match &self.operation {
            MigrationOperation::Info => Box::from(MigrationInfoCommand { args: self }),
            MigrationOperation::Sync => Box::from(MigrationSyncCommand { args: self }),
        }
    }
}
