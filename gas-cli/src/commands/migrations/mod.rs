use crate::commands::migrations::info::MigrationInfoCommand;
use crate::commands::migrations::init::MigrationInitCommand;
use crate::commands::migrations::migrate::MigrationMigrateCommand;
use crate::commands::migrations::sync::MigrationSyncCommand;
use crate::commands::{Command, CommandImplProvider};
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::str::FromStr;

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

#[derive(Debug, Clone)]
pub enum MigrateCount {
    All,
    Specific(NonZeroU64),
}

impl FromStr for MigrateCount {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "all" => Ok(MigrateCount::All),
            val => Ok(MigrateCount::Specific(NonZeroU64::from_str(val)?)),
        }
    }
}

impl MigrateCount {
    pub fn as_signed_count(&self, is_back: bool, max: usize) -> i64 {
        match self {
            MigrateCount::All if !is_back => max as i64,
            MigrateCount::All if is_back => -(max as i64),
            MigrateCount::Specific(n) if !is_back => n.get().cast_signed(),
            MigrateCount::Specific(n) if is_back => -n.get().cast_signed(),
            _ => unreachable!(),
        }
    }
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
    count: Option<MigrateCount>,
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
