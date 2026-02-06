use crate::binary::ProjectModelState;
use crate::commands::migrations::MigrationArgs;
use crate::error::GasCliResult;
use crate::project::CargoProject;
use crate::sync::ModelChangeActor;
use crate::util;
use console::{style, Style};
use std::time::Duration;

// compiles the binary and gets the current state of the binary while logging,
//  this is invoked by all migrations subcommands
pub async fn migrations_cli_common_program_state(
    args: &MigrationArgs,
) -> GasCliResult<ProjectModelState> {
    let binary_path = {
        let spinner = util::progress::default_spinner("Building...");
        spinner.enable_steady_tick(Duration::from_millis(100));

        let project = CargoProject::from_path(args.project_path.clone()).await?;
        let binary_path = project.build().await?;

        spinner.finish_and_clear();

        binary_path
    };

    let state = ProjectModelState::from_binary(&binary_path).await?;

    if state.fields.is_empty() {
        println!("{}", style("Project doesn't contain any models").yellow());
        return Ok(state);
    }

    println!("Models found in project:");
    for (model, fields) in state.fields.iter() {
        let joined_fields: String = fields
            .iter()
            .map(|field| style(field.name.as_ref()).green().bright().to_string())
            .reduce(|acc, curr| format!("{acc}, {curr}"))
            .unwrap_or(style("empty").red().bright().to_string());

        println!("  - {} ({})", style(model).green(), joined_fields);
    }

    println!();

    Ok(state)
}

pub fn diff_summary_visitor_fn<'a>((index, diff): (usize, &(dyn ModelChangeActor + 'a))) {
    if index == 0 {
        println!("Summary:")
    }

    // uhly
    println!(
        "  {} {}",
        Style::new().white().dim().apply_to("-"),
        Style::new().bold().apply_to(diff)
    )
}
