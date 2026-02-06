use crate::binary::{BinaryFields, TableSpec};
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::GasManifest;
use crate::sync::graph::order_diffs;
use crate::sync::variants::create_table::CreateTableModelActor;
use crate::sync::variants::rename_table::RenameTableModelActor;
use crate::sync::{MigrationScript, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use crate::util::styles::STYLE_ERR;
use crate::{sync, util};
use gas_shared::link::FixedStr;
use itertools::{Either, Itertools};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
struct TableSplit<'a> {
    new: Vec<TableSpec<'a>>,
    common: Vec<TableSpec<'a>>,
    old: Vec<TableSpec<'a>>,
}

fn handle_common_table<'a>(_diffs: &mut [Box<dyn ModelChangeActor + 'a>], table: &TableSpec<'a>) {
    // TODO:
    dbg!(&table);
}

// returns mapping of renamed tables (old, new)
fn handle_table_rename<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    tables: &mut TableSplit<'a>,
) {
    let mut common: Vec<(usize, usize)> = Vec::new();

    // quite inefficient ngl
    for (new_index, new_table) in tables.new.iter().enumerate() {
        for (old_index, old_table) in tables.old.iter().enumerate() {
            if new_table.fields.len() != old_table.fields.len() {
                continue;
            }

            println!("{:?} - {:?}", new_table, old_table);

            let mut all_fields = HashSet::new();
            all_fields.extend(new_table.fields.iter().cloned());
            all_fields.extend(old_table.fields.iter().cloned().map(|mut field| {
                field.table_name = FixedStr::try_from(new_table.name)
                    .expect("new_table.name can not be converted to FixedStr");
                field
            }));

            if all_fields.len() == new_table.fields.len() {
                common.push((new_index, old_index));
            }
        }
    }

    // rust has no clean way to remove multiple indices and get their values
    let old_tables = util::container::vec_remove_indices(
        &mut tables.old,
        &common.iter().map(|(old, _)| *old).collect::<Vec<_>>(),
    );

    let new_tables = util::container::vec_remove_indices(
        &mut tables.new,
        &common.iter().map(|(_, new)| *new).collect::<Vec<_>>(),
    );

    diffs.extend(
        old_tables
            .into_iter()
            .zip(new_tables)
            .map(|(old, new)| RenameTableModelActor::new_boxed(old, new)),
    );
}

// will figure out which tables are new, old and renamed
//  returns common tables that need deeper field diffing
fn handle_tables<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    state_fields: &'a BinaryFields,
    manifest: &'a GasManifest,
) -> Vec<TableSpec<'a>> {
    let new_tables: Vec<_> = state_fields
        .iter()
        .filter(|(table, ..)| !manifest.state.contains_key(*table))
        .map(TableSpec::from)
        .collect();

    let (old_tables, common_tables): (Vec<_>, Vec<_>) = manifest
        .state
        .iter()
        .map(TableSpec::from)
        .partition_map(|entry| match !state_fields.contains_key(entry.name) {
            true => Either::Left(entry),
            false => Either::Right(entry),
        });

    let mut table_split = TableSplit {
        new: new_tables,
        common: common_tables,
        old: old_tables,
    };

    handle_table_rename(diffs, &mut table_split);

    diffs.extend(
        table_split
            .new
            .into_iter()
            .map(CreateTableModelActor::new_boxed),
    );

    diffs.extend(
        table_split
            .old
            .into_iter()
            .map(CreateTableModelActor::new_boxed)
            .map(sync::helpers::diff::invert),
    );

    table_split.common
}

pub fn find_diffs<'a>(
    state_fields: &'a BinaryFields,
    manifest: &'a GasManifest,
) -> GasCliResult<Vec<Box<dyn ModelChangeActor + 'a>>> {
    let mut result: Vec<Box<dyn ModelChangeActor>> = Vec::new();

    let common_tables = handle_tables(&mut result, state_fields, manifest);

    for common_table in common_tables {
        handle_common_table(&mut result, &common_table);
    }

    Ok(result)
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
type DiffVisitorFn = fn((usize, &Box<dyn ModelChangeActor + '_>));

// very weird function
//  basically I want to sometimes iterate through all diffs
//  returning them causes lifetime headache
//  it's fiiiiinneeee
// and yes it's a function pointer
fn find_visit_collect_diffs(
    state_fields: &BinaryFields,
    manifest: &GasManifest,
    visitor: Option<DiffVisitorFn>,
) -> GasCliResult<Option<MigrationScript>> {
    let diffs = find_diffs(state_fields, manifest)?;
    if diffs.is_empty() {
        return Ok(None);
    }

    let diffs = order_diffs(manifest, diffs)?;

    if let Some(visitor) = visitor {
        for (index, diff) in diffs.iter().enumerate() {
            visitor((index, diff));
        }
    }

    collect_diffs(&diffs).map(Some)
}

pub fn find_and_collect_diffs(
    state_fields: &BinaryFields,
    manifest: &GasManifest,
) -> GasCliResult<Option<MigrationScript>> {
    find_visit_collect_diffs(state_fields, manifest, None)
}

pub fn find_visit_and_collect_diffs(
    state_fields: &BinaryFields,
    manifest: &GasManifest,
    visitor: DiffVisitorFn,
) -> GasCliResult<Option<MigrationScript>> {
    find_visit_collect_diffs(state_fields, manifest, Some(visitor))
}
