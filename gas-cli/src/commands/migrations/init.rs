use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::{GasManifestController, GasManifestError};
use crate::util;
use crate::util::common::migrations_cli_common_program_state;
use crate::util::styles::{STYLE_ERR, STYLE_OK};
use console::style;

pub struct MigrationInitCommand {
    pub(super) args: MigrationArgs,
}

#[async_trait::async_trait]
impl Command for MigrationInitCommand {
    async fn execute(&self) -> GasCliResult<()> {
        let state = migrations_cli_common_program_state(&self.args).await?;

        let migrations_dir = self.args.migrations_dir_path();
        let manifest_controller = GasManifestController::new(migrations_dir.clone());

        match manifest_controller.init_with(state.fields).await {
            Err(GasCliError::ManifestError(GasManifestError::AlreadyInitialized)) => {
                println!(
                    "{}: {}",
                    STYLE_ERR.apply_to(style("Target directory is already occupied")),
                    util::path::canonicalize_relative_pwd(migrations_dir)?.display()
                );

                Err(GasCliError::GeneralFailure)
            }
            Err(e) => Err(e),
            Ok(_) => {
                println!(
                    "{}: {}",
                    STYLE_OK.apply_to("Migrations successfully initialized"),
                    util::path::canonicalize_relative_pwd(migrations_dir)?.display()
                );

                Ok(())
            }
        }
    }
}
