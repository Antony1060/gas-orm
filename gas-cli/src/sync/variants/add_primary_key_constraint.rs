use crate::binary::TableSpec;
use crate::error::{GasCliError, GasCliResult};
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortableFieldMeta;
use gas_shared::FieldFlag;
use itertools::Itertools;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};

pub struct AddPrimaryKeyModelActor<'a> {
    old_table: TableSpec<'a>,
    fields: &'a [PortableFieldMeta],
}

impl<'a> AddPrimaryKeyModelActor<'a> {
    pub fn new_boxed(
        entry: TableSpec<'a>,
        fields: &'a [PortableFieldMeta],
    ) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(AddPrimaryKeyModelActor {
            old_table: entry,
            fields,
        })
    }
}

impl<'a> Display for AddPrimaryKeyModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddPrimaryKey[{}]",
            self.fields
                .iter()
                .map(|field| format!("{}.{}", field.table_name.as_ref(), field.name.as_ref()))
                .join(", ")
        )
    }
}

impl<'a> ModelChangeActor for AddPrimaryKeyModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        if self
            .old_table
            .fields
            .iter()
            .any(|field| field.flags.has_flag(FieldFlag::PrimaryKey))
        {
            return Err(GasCliError::MigrationsGenerationError {
                reason: Cow::from(
                    "can not add a primary key to a table with already existing primary keys",
                ),
            });
        }

        Ok(format!(
            "ALTER TABLE {} ADD PRIMARY KEY({})",
            self.old_table.name,
            self.fields
                .iter()
                .map(|field| field.name.as_ref())
                .join(", ")
        ))
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} DROP CONSTRAINT {}_pkey",
            self.old_table.name, self.old_table.name
        ))
    }

    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        Box::from([])
    }

    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        self.fields
            .iter()
            .map(|field| FieldDependency {
                table_name: field.table_name.as_ref(),
                name: field.name.as_ref(),
                state: FieldState::Existing,
            })
            .collect()
    }
}
