use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::{GasManifestController, GasManifestError};
use crate::util::common::migrations_cli_common_program_state;
use console::style;

pub struct MigrationInitCommand {
    pub(super) args: MigrationArgs,
}

#[async_trait::async_trait]
impl Command for MigrationInitCommand {
    async fn execute(&self) -> GasCliResult<()> {
        let state = migrations_cli_common_program_state(&self.args).await?;

        let migrations_dir = self.args.project_path.join(&self.args.migrations_dir_path);
        let manifest_controller = GasManifestController::new(migrations_dir.clone());

        if let Err(err) = manifest_controller.init_with(state.fields).await {
            // handle AlreadyInitialized with a nicer message
            let GasCliError::ManifestError(GasManifestError::AlreadyInitialized) = err else {
                return Err(err);
            };

            println!(
                "\n{}: {}",
                style("Target directory is already occupied")
                    .red()
                    .bright()
                    .bold(),
                migrations_dir.canonicalize()?.display()
            );

            return Err(GasCliError::GeneralFailure);
        }

        println!(
            "\n{}: {}",
            style("Migrations successfully initialized")
                .green()
                .bright()
                .bold(),
            migrations_dir.canonicalize()?.display()
        );

        Ok(())
    }
}
