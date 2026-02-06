#![allow(unused)]

use crate::error::{GasCliError, GasCliResult};
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortableFieldMeta;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};

pub struct AddDefaultModelActor<'a> {
    field: &'a PortableFieldMeta,
}

impl<'a> AddDefaultModelActor<'a> {
    pub fn new_boxed(field: &'a PortableFieldMeta) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(AddDefaultModelActor { field })
    }
}

impl<'a> Display for AddDefaultModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddDefault[{}.{}]",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        )
    }
}

impl<'a> ModelChangeActor for AddDefaultModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        let Some(ref default_sql) = self.field.default_sql else {
            return Err(GasCliError::MigrationsGenerationError {
                reason: Cow::from("invalid state: field expected to have a default sql expression"),
            });
        };

        Ok(format!(
            "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT ({})",
            self.field.table_name.as_ref(),
            self.field.name.as_ref(),
            default_sql.as_ref()
        ))
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT",
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
