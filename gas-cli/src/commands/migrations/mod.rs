use crate::commands::migrations::info::MigrationInfoCommand;
use crate::commands::migrations::init::MigrationInitCommand;
use crate::commands::migrations::migrate::MigrationMigrateCommand;
use crate::commands::migrations::sync::MigrationSyncCommand;
use crate::commands::{Command, CommandImplProvider};
use std::path::PathBuf;

mod info;
mod init;
mod migrate;
#[cfg(debug_assertions)]
mod reset;
mod sync;

#[derive(Debug, Clone, clap::Parser)]
pub struct SyncOptions {
    #[arg(long, default_value = "false", help = "generate a manual migration")]
    manual: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct MigrateOptions {
    #[arg(
        long,
        default_value = "false",
        help = "migrate backwards",
        default_value = "false"
    )]
    back: bool,

    #[arg(long, help = "amount of migrations to execute")]
    count: Option<usize>,
}

#[derive(Debug, clap::Subcommand)]
pub enum MigrationOperation {
    Info,
    Init,
    Sync(SyncOptions),
    Migrate(MigrateOptions),
    #[cfg(debug_assertions)]
    Reset,
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
        match self.operation {
            MigrationOperation::Info => Box::from(MigrationInfoCommand { args: self }),
            MigrationOperation::Init => Box::from(MigrationInitCommand { args: self }),
            MigrationOperation::Sync(ref options) => Box::from(MigrationSyncCommand {
                sync_options: options.clone(),
                args: self,
            }),
            MigrationOperation::Migrate(ref options) => Box::from(MigrationMigrateCommand {
                migrate_options: options.clone(),
                args: self,
            }),
            #[cfg(debug_assertions)]
            MigrationOperation::Reset => Box::from(reset::MigrationResetCommand { args: self }),
        }
    }
}

impl MigrationArgs {
    pub fn migrations_dir_path(&self) -> PathBuf {
        self.project_path.join(&self._migrations_dir_path)
    }
}
