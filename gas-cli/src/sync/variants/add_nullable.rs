#![allow(unused)]
use crate::error::GasCliResult;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortableFieldMeta;
use std::fmt::{Display, Formatter};

pub struct AddNullableModelActor<'a> {
    field: &'a PortableFieldMeta,
}

impl<'a> AddNullableModelActor<'a> {
    pub fn new_boxed(field: &'a PortableFieldMeta) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(AddNullableModelActor { field })
    }
}

impl<'a> Display for AddNullableModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddNullable[{}.{}]",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        )
    }
}

impl<'a> ModelChangeActor for AddNullableModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        ))
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        // TODO: if not default, invoke add default
        Ok(format!(
            "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
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
