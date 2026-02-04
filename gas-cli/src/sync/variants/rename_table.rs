use crate::binary::TableSpec;
use crate::error::GasCliResult;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use std::fmt::{Display, Formatter};

pub struct RenameTableModelActor<'a> {
    old_table: TableSpec<'a>,
    table: TableSpec<'a>,
}

impl<'a> RenameTableModelActor<'a> {
    pub fn new_boxed(
        old_table: TableSpec<'a>,
        table: TableSpec<'a>,
    ) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(RenameTableModelActor { old_table, table })
    }
}

impl<'a> Display for RenameTableModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RenameTable[{}->{}]",
            self.old_table.table, self.table.table
        )
    }
}

impl<'a> ModelChangeActor for RenameTableModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} RENAME TO {}",
            self.old_table.table, self.table.table,
        ))
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} RENAME TO {}",
            self.table.table, self.old_table.table,
        ))
    }

    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        Box::from([])
    }

    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        self.table
            .fields
            .iter()
            .map(|field| FieldDependency {
                table_name: field.table_name.as_ref(),
                name: field.name.as_ref(),
                state: FieldState::Existing,
            })
            .collect()
    }
}
