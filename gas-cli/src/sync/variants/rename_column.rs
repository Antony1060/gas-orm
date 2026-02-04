use crate::error::GasCliResult;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortableFieldMeta;
use std::fmt::{Display, Formatter};

pub struct RenameColumnModelActor<'a> {
    old_field: &'a PortableFieldMeta,
    field: &'a PortableFieldMeta,
}

impl<'a> RenameColumnModelActor<'a> {
    pub fn new_boxed(
        old_field: &'a PortableFieldMeta,
        field: &'a PortableFieldMeta,
    ) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(RenameColumnModelActor { old_field, field })
    }
}

impl<'a> Display for RenameColumnModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RenameColumn[{}.{}]",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        )
    }
}

impl<'a> ModelChangeActor for RenameColumnModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            self.field.table_name.as_ref(),
            self.old_field.name.as_ref(),
            self.field.name.as_ref()
        ))
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
            self.old_field.name.as_ref(),
        ))
    }

    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        Box::from([])
    }

    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        Box::from([FieldDependency {
            table_name: self.field.table_name.as_ref(),
            name: self.field.name.as_ref(),
            state: FieldState::Existing,
        }])
    }
}
