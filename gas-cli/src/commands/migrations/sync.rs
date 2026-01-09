use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::GasCliResult;

pub struct MigrationSyncCommand {
    #[allow(unused)]
    pub(super) args: MigrationArgs,
}

#[async_trait::async_trait]
impl Command for MigrationSyncCommand {
    async fn execute(&self) -> GasCliResult<()> {
        todo!()
    }
}
