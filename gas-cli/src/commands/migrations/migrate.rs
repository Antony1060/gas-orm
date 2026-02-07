use crate::commands::migrations::{MigrateOptions, MigrationArgs};
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::util::styles::STYLE_ERR;

pub struct MigrationMigrateCommand {
    pub(super) args: MigrationArgs,
    pub(super) migrate_options: MigrateOptions,
}

#[async_trait::async_trait]
impl Command for MigrationMigrateCommand {
    async fn execute(&self) -> GasCliResult<()> {
        if self.migrate_options.back && self.migrate_options.count.is_none() {
            println!(
                "{}",
                STYLE_ERR.apply_to("You must specify how many migrations to run (see --count option) when going backwards"),
            );

            return Err(GasCliError::GeneralFailure);
        }

        println!("TODO: {:#?}", self.migrate_options);

        Ok(())
    }
}
