use crate::binary::ProjectModelState;
use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::{GasManifest, GasManifestController, GasManifestError};
use crate::util::common::migrations_cli_common_program_state;
use crate::util::styles::{STYLE_ERR, STYLE_OK};
use crate::{sync, util};
use console::Term;
use dialoguer::Input;

pub struct MigrationSyncCommand {
    #[allow(unused)]
    pub(super) args: MigrationArgs,
}

pub struct SyncContext {
    controller: GasManifestController,
    state: ProjectModelState,
    manifest: GasManifest,
}

pub async fn handle_sync(
    SyncContext {
        controller,
        state,
        manifest,
    }: SyncContext,
) -> GasCliResult<()> {
    let script = sync::diff::find_and_collect_diffs(&state.fields, &manifest)?;

    let Some(script) = script else {
        println!(
            "{}",
            STYLE_OK.apply_to("Nothing to do, migrations are synced with the codebase")
        );

        return Ok(());
    };

    let name: String = Input::new()
        .with_prompt("Migrations script name")
        .interact_text_on(&Term::stdout())?;

    let script_path = controller.save_script(&name, &script).await?;

    println!(
        "{}: {}",
        STYLE_OK.apply_to("Migration saved"),
        util::path::canonicalize_relative_pwd(script_path)?.display()
    );

    Ok(())
}

#[async_trait::async_trait]
impl Command for MigrationSyncCommand {
    async fn execute(&self) -> GasCliResult<()> {
        let state = migrations_cli_common_program_state(&self.args).await?;

        let migrations_dir = self.args.migrations_dir_path();
        let manifest_controller = GasManifestController::new(migrations_dir.clone());

        // eh, I don't like the double match
        match manifest_controller.load().await {
            Err(GasCliError::ManifestError(GasManifestError::NotInitialized)) => {
                println!(
                    "{}",
                    STYLE_ERR.apply_to("Migrations don't seem to be initalized"),
                );

                Err(GasCliError::GeneralFailure)
            }
            Err(e) => Err(e),
            Ok(manifest) => {
                handle_sync(SyncContext {
                    controller: manifest_controller,
                    state,
                    manifest,
                })
                .await
            }
        }
    }
}
