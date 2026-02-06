use crate::binary::BinaryFields;
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::GasManifest;
use crate::sync::diff::find_diffs;
use crate::sync::graph::order_diffs;
use crate::sync::helpers::InverseChangeActor;
use crate::sync::{MigrationScript, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use crate::util::styles::STYLE_ERR;
use std::fmt::{Display, Formatter};

pub fn invert<'a>(other: Box<dyn ModelChangeActor + 'a>) -> Box<dyn ModelChangeActor + 'a> {
    InverseChangeActor::new_boxed(other)
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

pub fn collect_diffs<'a>(
    diffs: &[Box<dyn ModelChangeActor + 'a>],
) -> GasCliResult<MigrationScript> {
    let mut script = MigrationScript {
        forward: String::new(),
        backward: String::new(),
    };

    for diff in diffs {
        script.forward.push_str(&handle_change_actor(
            ChangeDirection::Forward,
            diff.forward_sql(),
        )?);
        script.forward.push_str(";\n");
    }

    for diff in diffs.iter().rev() {
        script.backward.push_str(&handle_change_actor(
            ChangeDirection::Backward,
            diff.backward_sql(),
        )?);
        script.backward.push_str(";\n");
    }

    Ok(script)
}

// yes it's a function pointer
pub type DiffVisitorFn = fn((usize, &(dyn ModelChangeActor + '_)));

// very weird function
//  basically I want to sometimes iterate through all diffs
//  returning them causes lifetime headache
//  it's fiiiiinneeee
// and yes it's a function pointer
pub fn find_and_collect_diffs(
    state_fields: &BinaryFields,
    manifest: &GasManifest,
    visitor: DiffVisitorFn,
) -> GasCliResult<Option<MigrationScript>> {
    let diffs = find_diffs(state_fields, manifest).map_err(|err| match err {
        GasCliError::MigrationsGenerationError { reason } => {
            println!(
                "{}: {}",
                STYLE_ERR.apply_to("Failed to determine changes".to_string()),
                reason
            );

            GasCliError::GeneralFailure
        }
        err => err,
    })?;

    if diffs.is_empty() {
        return Ok(None);
    }

    let diffs = order_diffs(manifest, diffs)?;

    for (index, diff) in diffs.iter().enumerate() {
        visitor((index, diff.as_ref()));
    }

    collect_diffs(&diffs).map(Some)
}
