use crate::binary::{FieldEntry, ProjectModelState};
use crate::error::GasCliResult;
use crate::manifest::GasManifest;
use crate::sync::variants::create_table::CreateTableModelActor;
use crate::sync::variants::drop_table::DropTableModelActor;
use crate::util::sql_query::SqlQuery;
use itertools::{Either, Itertools};

pub trait ModelChangeActor {
    fn forward_sql(&self) -> GasCliResult<SqlQuery>;

    fn backward_sql(&self) -> GasCliResult<SqlQuery>;
}

// TODO: implement actual "diffing"
pub fn find_diffs<'a>(
    project_state: &'a ProjectModelState,
    manifest: &'a GasManifest,
) -> GasCliResult<Box<[Box<dyn ModelChangeActor + 'a>]>> {
    let new_tables: Vec<_> = project_state
        .fields
        .iter()
        .filter(|(table, ..)| !manifest.state.contains_key(*table))
        .map(FieldEntry::from)
        .collect();

    let (old_tables, common_tables): (Vec<_>, Vec<_>) = manifest
        .state
        .iter()
        .map(FieldEntry::from)
        .partition_map(
            |entry| match !project_state.fields.contains_key(entry.table) {
                true => Either::Left(entry),
                false => Either::Right(entry),
            },
        );

    let mut result: Vec<Box<dyn ModelChangeActor>> = Vec::new();
    result.extend(new_tables.into_iter().map(CreateTableModelActor::new_boxed));
    result.extend(old_tables.into_iter().map(DropTableModelActor::new_boxed));

    for common_table in common_tables {
        dbg!(common_table.table);
    }

    Ok(result.into_boxed_slice())
}
