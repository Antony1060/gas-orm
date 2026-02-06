use crate::binary::{BinaryFields, TableSpec};
use crate::error::{GasCliError, GasCliResult};
use crate::manifest::GasManifest;
use crate::sync::variants::add_column::AddColumnModelActor;
use crate::sync::variants::add_default::AddDefaultModelActor;
use crate::sync::variants::add_foreign_key_constraint::AddForeignKeyModelActor;
use crate::sync::variants::add_nullable::AddNullableModelActor;
use crate::sync::variants::add_primary_key_constraint::AddPrimaryKeyModelActor;
use crate::sync::variants::add_serial::AddSerialModelActor;
use crate::sync::variants::add_unique_constraint::AddUniqueModelActor;
use crate::sync::variants::create_table::CreateTableModelActor;
use crate::sync::variants::rename_column::RenameColumnModelActor;
use crate::sync::variants::rename_table::RenameTableModelActor;
use crate::sync::variants::update_column_type::UpdateColumnTypeModelActor;
use crate::sync::{helpers, ModelChangeActor};
use crate::util::styles::{STYLE_WARN, STYLE_WARN_SOFT};
use crate::{sync, util};
use gas_shared::link::{FixedStr, PortableFieldMeta, PortablePgType};
use gas_shared::FieldFlag;
use itertools::{Either, Itertools};
use std::borrow::Cow;
use std::collections::HashSet;

#[derive(Debug)]
struct ColumnSplit<'a> {
    new: Vec<&'a PortableFieldMeta>,
    common: Vec<(&'a PortableFieldMeta, &'a PortableFieldMeta)>,
    old: Vec<&'a PortableFieldMeta>,
}

fn try_type<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old: &'a PortableFieldMeta,
    new: &'a PortableFieldMeta,
) {
    if old.pg_type == new.pg_type {
        return;
    }

    // promotion to a foreign key
    if let PortablePgType::ForeignKey { key_sql_type, .. } = &new.pg_type
        && key_sql_type.as_ref() == old.pg_type.as_sql_type(false)
    {
        diffs.push(AddForeignKeyModelActor::new_boxed(new));
        return;
    }

    // demotion from foreign key
    if let PortablePgType::ForeignKey { key_sql_type, .. } = &old.pg_type
        && key_sql_type.as_ref() == new.pg_type.as_sql_type(false)
    {
        diffs.push(helpers::diff::invert(AddForeignKeyModelActor::new_boxed(
            old,
        )));
        return;
    }

    if new.flags.has_flag(FieldFlag::PrimaryKey) || old.flags.has_flag(FieldFlag::PrimaryKey) {
        println!(
            "{} {}: Changing the type of a primary key field is unsupported, check the migration manually to verify everything was migrated correctly",
            STYLE_WARN.apply_to("WARNING"),
            STYLE_WARN_SOFT.apply_to(format!(
                "({}.{})",
                new.table_name.as_ref(),
                new.name.as_ref()
            ))
        )
    }

    diffs.push(UpdateColumnTypeModelActor::new_boxed(old, new))
}

fn try_default<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old: &'a PortableFieldMeta,
    new: &'a PortableFieldMeta,
) {
    if old.default_sql == new.default_sql {
        return;
    }

    if new.default_sql.is_some() {
        diffs.push(AddDefaultModelActor::new_boxed(new));
        return;
    }

    diffs.push(helpers::diff::invert(AddDefaultModelActor::new_boxed(new)));
}

fn try_nullable<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old: &'a PortableFieldMeta,
    new: &'a PortableFieldMeta,
) {
    if old.flags.has_flag(FieldFlag::Nullable) == new.flags.has_flag(FieldFlag::Nullable) {
        return;
    }

    let mut action = AddNullableModelActor::new_boxed(new);
    if !new.flags.has_flag(FieldFlag::Nullable) {
        action = helpers::diff::invert(action);
    }

    diffs.push(action);
}

fn try_serial<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old: &'a PortableFieldMeta,
    new: &'a PortableFieldMeta,
) {
    if old.flags.has_flag(FieldFlag::Serial) == new.flags.has_flag(FieldFlag::Serial) {
        return;
    }

    let mut action = AddSerialModelActor::new_boxed(new);
    if !new.flags.has_flag(FieldFlag::Serial) {
        action = helpers::diff::invert(action);
    }

    diffs.push(action);
}

fn try_unique<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old: &'a PortableFieldMeta,
    new: &'a PortableFieldMeta,
) {
    if old.flags.has_flag(FieldFlag::Unique) == new.flags.has_flag(FieldFlag::Unique) {
        return;
    }

    let mut action = AddUniqueModelActor::new_boxed(new);
    if !new.flags.has_flag(FieldFlag::Unique) {
        action = helpers::diff::invert(action);
    }

    diffs.push(action);
}

fn try_primary_key<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old_table: TableSpec<'a>,
    new_table: TableSpec<'a>,
) -> GasCliResult<()> {
    let new_primary_keys: Vec<_> = new_table
        .fields
        .iter()
        .filter(|field| field.flags.has_flag(FieldFlag::PrimaryKey))
        .collect();

    let old_primary_keys: Vec<_> = old_table
        .fields
        .iter()
        .filter(|field| field.flags.has_flag(FieldFlag::PrimaryKey))
        .collect();

    if old_primary_keys.is_empty() && !new_primary_keys.is_empty() {
        diffs.push(AddPrimaryKeyModelActor::new_boxed(
            old_table,
            new_primary_keys.into_boxed_slice(),
            true,
        ));

        return Ok(());
    }

    if new_primary_keys.is_empty() && !old_primary_keys.is_empty() {
        diffs.push(helpers::diff::invert(AddPrimaryKeyModelActor::new_boxed(
            old_table,
            old_primary_keys.into_boxed_slice(),
            true,
        )));

        return Ok(());
    }

    if new_primary_keys.len() != old_primary_keys.len()
        || new_primary_keys.into_iter().any(|field| {
            !old_primary_keys
                .iter()
                .any(|other| other.name == field.name)
        })
    {
        return Err(GasCliError::MigrationsGenerationError {
            reason: Cow::from(
                "can not add a primary key to a table with already existing primary keys",
            ),
        });
    }

    Ok(())
}

fn handle_common_column<'a>(
    diffs: &mut Vec<Box<dyn ModelChangeActor + 'a>>,
    old_column: &'a PortableFieldMeta,
    new_column: &'a PortableFieldMeta,
) {
    assert_eq!(old_column.name, new_column.name);

    try_type(diffs, old_column, new_column);
    try_default(diffs, old_column, new_column);
    try_nullable(diffs, old_column, new_column);
    try_serial(diffs, old_column, new_column);
    try_unique(diffs, old_column, new_column);
}

fn try_column_rename<'a>(
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
) -> GasCliResult<()> {
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

    try_column_rename(diffs, &mut column_split);

    diffs.extend(
        column_split
            .new
            .into_iter()
            .map(AddColumnModelActor::new_boxed),
    );

    diffs.extend(
        column_split
            .old
            .into_iter()
            .map(AddColumnModelActor::new_boxed)
            .map(helpers::diff::invert),
    );

    try_primary_key(diffs, old_table.clone(), new_table)?;

    for (old, new) in column_split.common {
        handle_common_column(diffs, old, new);
    }

    Ok(())
}

#[derive(Debug)]
struct TableSplit<'a> {
    new: Vec<TableSpec<'a>>,
    common: Vec<TableSpec<'a>>,
    old: Vec<TableSpec<'a>>,
}

fn try_table_rename<'a>(
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

    try_table_rename(diffs, &mut table_split);

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
        handle_common_table(&mut result, old, new)?;
    }

    Ok(result)
}
