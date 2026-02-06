use crate::error::{GasCliError, GasCliResult};
use crate::sync::variants::add_default::AddDefaultModelActor;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortableFieldMeta;
use std::borrow::Cow;
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

impl<'a> AddNullableModelActor<'a> {
    fn get_default_actor(&self) -> Box<dyn ModelChangeActor + '_> {
        AddDefaultModelActor::new_boxed(self.field)
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
        if self.field.default_sql.is_none() {
            return Err(GasCliError::MigrationsGenerationError {
                reason: Cow::from(
                    "can not remove nullable property without a defined default behaviour",
                ),
            });
        };

        let mut sql = SqlQuery::new();

        sql.push_str(&self.get_default_actor().forward_sql()?);
        sql.push_str(";\n");
        sql.push_str(&format!(
            "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        ));

        Ok(sql)
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

    fn depends_on_inverted(&self) -> Box<[FieldDependency<'_>]> {
        self.depends_on()
    }
}
