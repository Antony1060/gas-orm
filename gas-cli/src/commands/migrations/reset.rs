use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::GasCliResult;
use crate::util::styles::STYLE_OK;

// does rm -r ./migrations most of the time lol
//  debug only
pub struct MigrationResetCommand {
    pub(super) args: MigrationArgs,
}

#[async_trait::async_trait]
impl Command for MigrationResetCommand {
    async fn execute(&self) -> GasCliResult<()> {
        let migrations_dir = self.args.migrations_dir_path();

        if !migrations_dir.exists() {
            println!(
                "{}",
                STYLE_OK.apply_to("Migrations are not initialized, nothing to do")
            );

            return Ok(());
        }

        tokio::fs::remove_dir_all(&migrations_dir).await?;

        println!("{}", STYLE_OK.apply_to("Gone 👋"));

        Ok(())
    }
}
