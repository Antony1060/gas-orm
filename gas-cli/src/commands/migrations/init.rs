use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::GasManifest;
use crate::util::common::migrations_cli_common_program_state;
use console::style;
use tokio::fs;
use tokio::runtime::Handle;

const MANIFEST_FILE_NAME: &str = "manifest.json";

pub struct MigrationInitCommand {
    pub(super) args: MigrationArgs,
}

#[async_trait::async_trait]
impl Command for MigrationInitCommand {
    async fn execute(&self) -> GasCliResult<()> {
        let state = migrations_cli_common_program_state(&self.args).await?;

        let migrations_dir_path = self.args.project_path.join(&self.args.migrations_dir_path);

        if migrations_dir_path.exists() {
            println!(
                "\n{}: {}",
                style("Target directory is already occupied")
                    .red()
                    .bright()
                    .bold(),
                migrations_dir_path.canonicalize()?.display()
            );

            return Err(GasCliError::GeneralFailure);
        }

        fs::create_dir_all(&migrations_dir_path).await?;

        let manifest_file_path = migrations_dir_path.join(MANIFEST_FILE_NAME);

        Handle::current()
            .spawn_blocking(move || {
                // I assume fs::File::create doesn't queue anything inflight
                let file = std::fs::File::create(manifest_file_path)?;

                serde_json::to_writer_pretty(file, &GasManifest::new(state.fields.clone()))?;

                Ok::<(), GasCliError>(())
            })
            .await
            .expect("join should have worked")?;

        println!(
            "\n{}: {}",
            style("Migrations successfully initialized")
                .green()
                .bright()
                .bold(),
            migrations_dir_path.canonicalize()?.display()
        );

        Ok(())
    }
}
