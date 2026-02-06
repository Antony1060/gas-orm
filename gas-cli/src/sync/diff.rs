use crate::binary::{BinaryFields, TableSpec};
use crate::error::GasCliResult;
use crate::manifest::GasManifest;
use crate::sync::variants::add_column::AddColumnModelActor;
use crate::sync::variants::create_table::CreateTableModelActor;
use crate::sync::variants::rename_column::RenameColumnModelActor;
use crate::sync::variants::rename_table::RenameTableModelActor;
use crate::sync::ModelChangeActor;
use crate::{sync, util};
use gas_shared::link::{FixedStr, PortableFieldMeta};
use itertools::{Either, Itertools};
use std::collections::HashSet;

#[derive(Debug)]
struct ColumnSplit<'a> {
    new: Vec<&'a PortableFieldMeta>,
    common: Vec<(&'a PortableFieldMeta, &'a PortableFieldMeta)>,
    old: Vec<&'a PortableFieldMeta>,
}

fn handle_common_column<'a>(
    _diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old_column: &PortableFieldMeta,
    new_column: &PortableFieldMeta,
) {
    println!("common column: {} -> {}", old_column, new_column);
}

fn handle_column_rename<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    columns: &mut ColumnSplit<'a>,
) {
    let mut common: Vec<(usize, usize)> = Vec::new();

    for (new_index, new_column) in columns.new.iter().enumerate() {
        for (old_index, old_column) in columns.old.iter().enumerate() {
            let old_column = {
                let mut col = (*old_column).clone();
                col.name = new_column.name.clone();
                col
            };

            println!("{:?} - {:?}", new_column, old_column);

            if **new_column != old_column {
                continue;
            }

            common.push((new_index, old_index));
        }
    }

    let old_columns = util::container::vec_remove_indices(
        &mut columns.old,
        &common.iter().map(|(old, _)| *old).collect::<Vec<_>>(),
    );

    let new_columns = util::container::vec_remove_indices(
        &mut columns.new,
        &common.iter().map(|(_, new)| *new).collect::<Vec<_>>(),
    );

    diffs.extend(
        old_columns
            .into_iter()
            .zip(new_columns)
            .map(|(old, new)| RenameColumnModelActor::new_boxed(old, new)),
    );
}

fn handle_common_table<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old_table: TableSpec<'a>,
    new_table: TableSpec<'a>,
) {
    assert_eq!(old_table.name, new_table.name);

    let new_columns: Vec<_> = new_table
        .fields
        .iter()
        .filter(|field| {
            !old_table
                .fields
                .iter()
                .any(|other| other.name == field.name)
        })
        .collect();

    let (old_columns, common_columns): (Vec<_>, Vec<_>) =
        old_table.fields.iter().partition_map(|field| {
            match new_table
                .fields
                .iter()
                .find(|other| other.name == field.name)
            {
                Some(new_field) => Either::Right((field, new_field)),
                None => Either::Left(field),
            }
        });

    let mut column_split = ColumnSplit {
        new: new_columns,
        common: common_columns,
        old: old_columns,
    };

    handle_column_rename(diffs, &mut column_split);

    diffs.extend(
        column_split
            .new
            .into_iter()
            .map(|field| AddColumnModelActor::new_boxed(old_table.clone(), field)),
    );

    diffs.extend(
        column_split
            .old
            .into_iter()
            .map(|field| AddColumnModelActor::new_boxed(old_table.clone(), field))
            .map(sync::helpers::diff::invert),
    );

    for (old, new) in column_split.common {
        handle_common_column(diffs, old, new);
    }
}

#[derive(Debug)]
struct TableSplit<'a> {
    new: Vec<TableSpec<'a>>,
    common: Vec<TableSpec<'a>>,
    old: Vec<TableSpec<'a>>,
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
) -> Vec<(TableSpec<'a>, TableSpec<'a>)> {
    let new_tables: Vec<_> = state_fields
        .iter()
        .filter(|(table, ..)| !manifest.state.contains_key(*table))
        .map(TableSpec::from)
        .collect();

    let (old_tables, common_tables): (Vec<_>, Vec<_>) = manifest
        .state
        .iter()
        .map(TableSpec::from)
        .partition_map(|entry| match state_fields.contains_key(entry.name) {
            true => Either::Right(entry),
            false => Either::Left(entry),
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

    // table_split.common contains entries from the manifest (i.e. old tables)
    table_split
        .common
        .into_iter()
        .map(|spec| {
            (
                TableSpec {
                    name: spec.name,
                    fields: &state_fields[spec.name],
                },
                spec,
            )
        })
        .map(|(new, old)| (old, new))
        .collect()
}

pub fn find_diffs<'a>(
    state_fields: &'a BinaryFields,
    manifest: &'a GasManifest,
) -> GasCliResult<Vec<Box<dyn ModelChangeActor + 'a>>> {
    let mut result: Vec<Box<dyn ModelChangeActor>> = Vec::new();

    let common_tables = handle_tables(&mut result, state_fields, manifest);

    for (old, new) in common_tables {
        handle_common_table(&mut result, old, new);
    }

    Ok(result)
}
