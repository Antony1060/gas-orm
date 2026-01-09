use crate::commands::migrations::MigrationArgs;
use crate::error::GasCliResult;

pub mod migrations;

#[derive(clap::Subcommand)]
pub enum CommandDef {
    Migrations(MigrationArgs),
}

// ew
pub trait CommandImplProvider {
    fn get_command(self) -> Box<dyn Command>;
}

// really this could've been done without dynamic dispatch but oh well
#[async_trait::async_trait]
pub trait Command {
    async fn execute(&self) -> GasCliResult<()>;
}

impl CommandImplProvider for CommandDef {
    fn get_command(self) -> Box<dyn Command> {
        match self {
            CommandDef::Migrations(args) => args.get_command(),
        }
    }
}
