#![allow(unused)]
use crate::error::GasCliResult;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::{PortableFieldMeta, PortablePgType};
use std::fmt::{Display, Formatter};

pub struct AddForeignKeyModelActor<'a> {
    field: &'a PortableFieldMeta,
}

impl<'a> AddForeignKeyModelActor<'a> {
    pub fn new_boxed(field: &'a PortableFieldMeta) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(AddForeignKeyModelActor { field })
    }
}

impl<'a> Display for AddForeignKeyModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddForeignKey[{}.{}]",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        )
    }
}

impl<'a> ModelChangeActor for AddForeignKeyModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        let PortablePgType::ForeignKey {
            ref target_table_name,
            ref target_column_name,
            ..
        } = self.field.pg_type
        else {
            unreachable!("field should be a foreign key but is not a foreign key")
        };

        Ok(format!(
            "ALTER TABLE {} ADD FOREIGN KEY({}) REFERENCES {}({})",
            self.field.table_name.as_ref(),
            self.field.name.as_ref(),
            target_table_name.as_ref(),
            target_column_name.as_ref(),
        ))
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE DROP CONSTRAINT {}_{}_fkey",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        ))
    }

    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        Box::from([])
    }

    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        let PortablePgType::ForeignKey {
            ref target_table_name,
            ref target_column_name,
            ..
        } = self.field.pg_type
        else {
            unreachable!("field should be a foreign key but is not a foreign key")
        };

        Box::from([
            FieldDependency {
                table_name: self.field.table_name.as_ref(),
                name: self.field.name.as_ref(),
                state: FieldState::Existing,
            },
            FieldDependency {
                table_name: target_table_name.as_ref(),
                name: target_column_name.as_ref(),
                state: FieldState::Existing,
            },
        ])
    }
}
