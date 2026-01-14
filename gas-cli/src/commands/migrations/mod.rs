use crate::commands::migrations::info::MigrationInfoCommand;
use crate::commands::migrations::init::MigrationInitCommand;
use crate::commands::migrations::sync::MigrationSyncCommand;
use crate::commands::{Command, CommandImplProvider};
use std::path::PathBuf;

mod info;
mod init;
mod sync;

#[derive(Debug, clap::Subcommand)]
pub enum MigrationOperation {
    Info,
    Init,
    Sync,
}

#[derive(Debug, clap::Parser)]
pub struct MigrationArgs {
    #[arg(long, short = 'p', default_value = ".")]
    pub project_path: PathBuf,
    #[arg(
        long,
        short = 'm',
        default_value = "./migrations",
        help = "relative to project_path"
    )]
    _migrations_dir_path: PathBuf,
    #[command(subcommand)]
    pub operation: MigrationOperation,
}

impl CommandImplProvider for MigrationArgs {
    fn get_command(self) -> Box<dyn Command> {
        match &self.operation {
            MigrationOperation::Info => Box::from(MigrationInfoCommand { args: self }),
            MigrationOperation::Init => Box::from(MigrationInitCommand { args: self }),
            MigrationOperation::Sync => Box::from(MigrationSyncCommand { args: self }),
        }
    }
}

impl MigrationArgs {
    pub fn migrations_dir_path(&self) -> PathBuf {
        self.project_path.join(&self._migrations_dir_path)
    }
}
