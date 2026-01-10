use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::GasCliResult;
use crate::util::common::migrations_cli_common_program_state;

pub struct MigrationInfoCommand {
    pub(super) args: MigrationArgs,
}

#[async_trait::async_trait]
impl Command for MigrationInfoCommand {
    async fn execute(&self) -> GasCliResult<()> {
        migrations_cli_common_program_state(&self.args).await?;

        Ok(())
    }
}
