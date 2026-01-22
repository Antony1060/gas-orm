use crate::binary::{BinaryFields, FieldEntry};
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::GasManifest;
use crate::sync;
use crate::sync::variants::create_table::CreateTableModelActor;
use crate::sync::MigrationScript;
use crate::util::sql_query::SqlQuery;
use crate::util::styles::STYLE_ERR;
use itertools::{Either, Itertools};
use std::fmt::{Display, Formatter};

pub trait ModelChangeActor {
    fn forward_sql(&self) -> GasCliResult<SqlQuery>;

    fn backward_sql(&self) -> GasCliResult<SqlQuery>;
}

// TODO: implement actual "diffing"
pub fn find_diffs<'a>(
    state_fields: &'a BinaryFields,
    manifest: &'a GasManifest,
) -> GasCliResult<Box<[Box<dyn ModelChangeActor + 'a>]>> {
    let new_tables: Vec<_> = state_fields
        .iter()
        .filter(|(table, ..)| !manifest.state.contains_key(*table))
        .map(FieldEntry::from)
        .collect();

    let (old_tables, common_tables): (Vec<_>, Vec<_>) = manifest
        .state
        .iter()
        .map(FieldEntry::from)
        .partition_map(|entry| match !state_fields.contains_key(entry.table) {
            true => Either::Left(entry),
            false => Either::Right(entry),
        });

    let mut result: Vec<Box<dyn ModelChangeActor>> = Vec::new();
    result.extend(new_tables.into_iter().map(CreateTableModelActor::new_boxed));
    result.extend(
        old_tables
            .into_iter()
            .map(CreateTableModelActor::new_boxed)
            .map(sync::helpers::diff::invert),
    );

    for common_table in common_tables {
        dbg!(common_table.table);
    }

    Ok(result.into_boxed_slice())
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

        script.backward.push_str(&handle_change_actor(
            ChangeDirection::Backward,
            diff.backward_sql(),
        )?);
        script.backward.push_str(";\n");
    }

    Ok(script)
}

pub fn find_and_collect_diffs(
    state_fields: &BinaryFields,
    manifest: &GasManifest,
) -> GasCliResult<Option<MigrationScript>> {
    let diffs = find_diffs(state_fields, manifest)?;
    if diffs.is_empty() {
        return Ok(None);
    }

    collect_diffs(&diffs).map(Some)
}
