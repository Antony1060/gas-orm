use crate::error::GasCliResult;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortableFieldMeta;
use std::fmt::{Display, Formatter};

pub struct AddUniqueModelActor<'a> {
    field: &'a PortableFieldMeta,
}

impl<'a> AddUniqueModelActor<'a> {
    pub fn new_boxed(field: &'a PortableFieldMeta) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(AddUniqueModelActor { field })
    }
}

impl<'a> Display for AddUniqueModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddUnique[{}.{}]",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        )
    }
}

impl<'a> ModelChangeActor for AddUniqueModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} ADD UNIQUE({})",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        ))
    }

    // NOTE: may have unintended consequences for indexed columns
    //  but that's currently not supported so oh well
    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!(
            "ALTER TABLE {} DROP CONSTRAINT {}_{}_key",
            self.field.table_name.as_ref(),
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

    fn depends_on_inverted(&self) -> Box<[FieldDependency<'_>]> {
        self.depends_on()
    }
}
