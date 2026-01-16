use crate::binary::ProjectModelState;
use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::{GasManifest, GasManifestController, GasManifestError};
use crate::sync::diff::{ModelChangeActor, SampleModelActor};
use crate::sync::MigrationScript;
use crate::util;
use crate::util::common::migrations_cli_common_program_state;
use crate::util::sql_query::SqlQuery;
use crate::util::styles::{STYLE_ERR, STYLE_OK};
use console::{style, Term};
use dialoguer::Input;
use std::fmt::{Display, Formatter};

pub struct MigrationSyncCommand {
    #[allow(unused)]
    pub(super) args: MigrationArgs,
}

pub struct SyncContext {
    controller: GasManifestController,
    state: ProjectModelState,
    manifest: GasManifest,
}

// TODO: implement actual "diffing"
pub fn find_diffs<'a>(
    project_state: &'a ProjectModelState,
    manifest: &'a GasManifest,
) -> GasCliResult<Box<[Box<dyn ModelChangeActor>]>> {
    // TODO: very crude now, just logs if any change
    for (table, fields) in &project_state.fields {
        print!("Checking: {} ", style(table).green());
        let manifest_fields = manifest.state.get(table);

        let Some(manifest_fields) = manifest_fields else {
            println!("{}", STYLE_ERR.apply_to("DIFFER"));
            continue;
        };

        if fields.len() != manifest_fields.len() {
            println!("{}", STYLE_ERR.apply_to("DIFFER"));
            continue;
        };

        let mut fields = fields.clone();
        fields.sort_by_cached_key(|it| it.name.clone());

        let mut manifest_fields = manifest_fields.clone();
        manifest_fields.sort_by_cached_key(|it| it.name.clone());

        for (field, manifest_field) in fields.into_iter().zip(manifest_fields) {
            if field != manifest_field {
                println!("{}", STYLE_ERR.apply_to("DIFFER"));
                continue;
            }
        }

        println!("{}", STYLE_OK.apply_to("SAME"));
    }

    println!();

    Ok(Box::from([
        Box::from(SampleModelActor {}) as Box<dyn ModelChangeActor>
    ]))
}

enum ChangeDirection {
    Forward,
    Backward,
}

impl Display for ChangeDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeDirection::Forward => write!(f, "forward"),
            ChangeDirection::Backward => write!(f, "backward"),
        }
    }
}

fn handle_change_actor(
    direction: ChangeDirection,
    change_result: GasCliResult<SqlQuery>,
) -> GasCliResult<SqlQuery> {
    change_result.map_err(|err| match err {
        GasCliError::MigrationsGenerationError { reason } => {
            println!(
                "{}: {}",
                STYLE_ERR.apply_to(format!("Failed to determine changes ({direction})")),
                reason
            );

            GasCliError::GeneralFailure
        }
        err => err,
    })
}

pub async fn handle_sync(
    SyncContext {
        controller,
        state,
        manifest,
    }: SyncContext,
) -> GasCliResult<()> {
    let diffs = find_diffs(&state, &manifest)?;

    if diffs.is_empty() {
        println!(
            "{}",
            STYLE_OK.apply_to("Nothing to do, migrations are synced with the codebase")
        );
    }

    let mut script = MigrationScript {
        forward: String::new(),
        backward: String::new(),
    };

    for diff in diffs {
        script.forward.push_str(&handle_change_actor(
            ChangeDirection::Forward,
            diff.forward_sql(),
        )?);
        script.forward.push('\n');

        script.backward.push_str(&handle_change_actor(
            ChangeDirection::Backward,
            diff.backward_sql(),
        )?);
        script.backward.push('\n');
    }

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
