use crate::binary::ProjectModelState;
use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::diff::DiffScript;
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::{GasManifest, GasManifestController, GasManifestError};
use crate::util::common::migrations_cli_common_program_state;
use crate::util::styles::{STYLE_ERR, STYLE_OK};
use console::style;

pub struct MigrationSyncCommand {
    #[allow(unused)]
    pub(super) args: MigrationArgs,
}

pub fn sync_states(
    project_state: &ProjectModelState,
    manifest: &GasManifest,
) -> GasCliResult<Option<DiffScript>> {
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
    Ok(None)
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
                    "\n{}",
                    STYLE_ERR.apply_to("Migrations don't seem to be initalized"),
                );

                Err(GasCliError::GeneralFailure)
            }
            Err(e) => Err(e),
            Ok(manifest) => {
                let script = sync_states(&state, &manifest)?;

                match script {
                    Some(_) => unimplemented!(),
                    None => {
                        println!(
                            "\n{}",
                            STYLE_OK.apply_to("Nothing to do, migrations are at the latest state")
                        );
                    }
                }

                Ok(())
            }
        }
    }
}
