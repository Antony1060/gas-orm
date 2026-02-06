use crate::error::GasCliResult;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortableFieldMeta;
use std::fmt::{Display, Formatter};

pub struct AddSerialModelActor<'a> {
    field: &'a PortableFieldMeta,
    _seq_name: String,
}

impl<'a> AddSerialModelActor<'a> {
    pub fn new_boxed(field: &'a PortableFieldMeta) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(AddSerialModelActor {
            field,
            _seq_name: format!("{}_{}_seq", field.table_name.as_ref(), field.name.as_ref()),
        })
    }
}

impl<'a> Display for AddSerialModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddSerial[{}.{}]",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        )
    }
}

impl<'a> ModelChangeActor for AddSerialModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        let mut sql = SqlQuery::new();

        sql.push_str(&format!("CREATE SEQUENCE {};\n", self._seq_name));

        sql.push_str(&format!(
            "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT (next_val('{}'));\n",
            self.field.table_name.as_ref(),
            self.field.name.as_ref(),
            self._seq_name
        ));

        sql.push_str(&format!(
            "ALTER SEQUENCE {} OWNED BY {}.{}",
            self._seq_name,
            self.field.table_name.as_ref(),
            self.field.name.as_ref(),
        ));

        Ok(sql)
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        let mut sql = SqlQuery::new();

        sql.push_str(&format!(
            "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;\n",
            self.field.table_name.as_ref(),
            self.field.name.as_ref(),
        ));

        sql.push_str(&format!("DROP SEQUENCE {}", self._seq_name));

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
