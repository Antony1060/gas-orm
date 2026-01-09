use crate::binary::ProjectModelState;
use crate::commands::migrations::MigrationArgs;
use crate::commands::Command;
use crate::error::GasCliResult;
use crate::project::CargoProject;
use crate::util;
use console::style;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;

pub struct MigrationInfoCommand {
    pub(super) args: MigrationArgs,
}

impl Deref for MigrationInfoCommand {
    type Target = MigrationArgs;

    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

#[async_trait::async_trait]
impl Command for MigrationInfoCommand {
    async fn execute(&self) -> GasCliResult<()> {
        let binary_path = {
            let spinner = util::progress::default_spinner("Building...");
            spinner.enable_steady_tick(Duration::from_millis(100));

            let project = CargoProject::from_path(PathBuf::from(&self.project_path)).await?;
            let binary_path = project.build().await?;

            spinner.finish_and_clear();

            binary_path
        };

        let project_state = ProjectModelState::from_binary(&binary_path).await?;
        let fields = project_state.get_organized();

        if fields.is_empty() {
            println!("{}", style("Project doesn't contain any models").yellow());
            return Ok(());
        }

        println!("Models found in project:");
        for (model, fields) in fields.iter() {
            let joined_fields: String = fields
                .iter()
                .map(|field| style(field.name.as_ref()).green().bright().to_string())
                .reduce(|acc, curr| format!("{acc}, {curr}"))
                .unwrap_or(style("empty").red().bright().to_string());

            println!("  - {} ({})", style(model.as_ref()).green(), joined_fields);
        }

        Ok(())
    }
}
