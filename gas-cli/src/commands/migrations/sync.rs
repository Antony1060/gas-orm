use crate::binary::ProjectModelState;
use crate::commands::migrations::{MigrationArgs, SyncOptions};
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::{GasManifest, GasManifestController, GasManifestError};
use crate::sync::MigrationScript;
use crate::util::common::{diff_summary_visitor_fn, migrations_cli_common_program_state};
use crate::util::styles::{STYLE_ERR, STYLE_OK, STYLE_WARN};
use crate::{sync, util};
use console::Term;
use dialoguer::Input;

pub struct MigrationSyncCommand {
    pub(super) args: MigrationArgs,
    pub(super) sync_options: SyncOptions,
}

pub struct SyncContext {
    controller: GasManifestController,
    state: ProjectModelState,
    manifest: GasManifest,
}

pub async fn handle_sync(
    options: &SyncOptions,
    SyncContext {
        controller,
        state,
        manifest,
    }: SyncContext,
) -> GasCliResult<()> {
    let script = if options.manual {
        Some(MigrationScript {
            forward: String::from("-- put your forward migration here\n\n"),
            backward: String::from("\n-- put your backward migration here"),
        })
    } else {
        // bad aah code
        sync::helpers::diff::find_and_collect_diffs(
            &state.fields,
            &manifest,
            diff_summary_visitor_fn,
        )?
    };

    let Some(script) = script else {
        println!(
            "{}",
            STYLE_OK.apply_to("Nothing to do, migrations are synced with the codebase")
        );

        return Ok(());
    };

    if options.manual {
        println!(
            "{}: You are responsible for bringing the state of the database to the current state of your project",
            STYLE_WARN.apply_to("WARNING")
        )
    }
    println!();

    let name: String = Input::new()
        .with_prompt("Migration script name")
        .interact_text_on(&Term::stdout())?;

    let script_path = controller.save_script(&name, &script).await?;

    let _ = controller.save_fields(state.fields).await?;

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

        match manifest_controller.load().await {
            Err(GasCliError::ManifestError(GasManifestError::NotInitialized)) => {
                println!(
                    "{}",
                    STYLE_ERR.apply_to("Migrations don't seem to be initialized"),
                );

                Err(GasCliError::GeneralFailure)
            }
            Err(err) => Err(err),
            Ok(manifest) => {
                handle_sync(
                    &self.sync_options,
                    SyncContext {
                        controller: manifest_controller,
                        state,
                        manifest,
                    },
                )
                .await
            }
        }
    }
}
