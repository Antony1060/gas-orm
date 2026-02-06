#![allow(unused)]
use crate::binary::TableSpec;
use crate::error::GasCliResult;
use crate::sync::variants::add_primary_key_constraint::AddPrimaryKeyModelActor;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util;
use crate::util::sql_query::SqlQuery;
use gas_shared::link::{PortableFieldMeta, PortablePgType};
use gas_shared::FieldFlag;
use std::fmt::{Display, Formatter};

pub struct AddColumnModelActor<'a> {
    old_table: TableSpec<'a>,
    field: &'a PortableFieldMeta,
}

impl<'a> AddColumnModelActor<'a> {
    pub fn new_boxed(
        table: TableSpec<'a>,
        field: &'a PortableFieldMeta,
    ) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(AddColumnModelActor {
            old_table: table,
            field,
        })
    }
}

impl<'a> Display for AddColumnModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AddColumn[{}.{}]",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        )
    }
}
impl<'a> AddColumnModelActor<'a> {
    fn get_add_primary_key_actor(&self) -> Box<dyn ModelChangeActor + '_> {
        AddPrimaryKeyModelActor::new_boxed(self.old_table.clone(), std::slice::from_ref(self.field))
    }
}

impl<'a> ModelChangeActor for AddColumnModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        let mut sql = SqlQuery::from("ALTER TABLE ");
        let field = &self.field;

        sql.push_str(field.table_name.as_ref());

        sql.push_str(" ADD COLUMN ");

        sql.push_str(&util::sql_query::gen_column_descriptor_sql(field));

        if field.flags.has_flag(FieldFlag::PrimaryKey) {
            sql.push(';');

            sql.push_str(&self.get_add_primary_key_actor().forward_sql()?)
        };

        Ok(sql)
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        let mut sql = SqlQuery::default();

        if self.field.flags.has_flag(FieldFlag::PrimaryKey) {
            sql.push_str(&self.get_add_primary_key_actor().backward_sql()?);

            sql.push(';');
        }

        sql.push_str(&format!(
            "ALTER TABLE {} DROP COLUMN {}",
            self.field.table_name.as_ref(),
            self.field.name.as_ref()
        ));

        Ok(sql)
    }

    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        Box::from([FieldDependency {
            table_name: self.field.table_name.as_ref(),
            name: self.field.name.as_ref(),
            state: FieldState::Existing,
        }])
    }

    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        if !self.field.flags.has_flag(FieldFlag::ForeignKey) {
            return Box::from([]);
        }

        let PortablePgType::ForeignKey {
            ref target_table_name,
            ref target_column_name,
            ..
        } = self.field.pg_type
        else {
            unreachable!("field is marked as foreign key but is not a foreign key")
        };

        Box::from([FieldDependency {
            table_name: target_table_name.as_ref(),
            name: target_column_name.as_ref(),
            state: FieldState::Existing,
        }])
    }
}
