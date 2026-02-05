use crate::binary::TableSpec;
use crate::error::GasCliResult;
use crate::sync::{FieldDependency, FieldState, ModelChangeActor};
use crate::util;
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortablePgType;
use gas_shared::FieldFlag;
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

pub struct CreateTableModelActor<'a> {
    entry: TableSpec<'a>,
}

impl<'a> CreateTableModelActor<'a> {
    pub fn new_boxed(entry: TableSpec) -> Box<dyn ModelChangeActor + '_> {
        Box::from(CreateTableModelActor { entry })
    }
}

impl<'a> Deref for CreateTableModelActor<'a> {
    type Target = TableSpec<'a>;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl<'a> Display for CreateTableModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CreateTable[{}]",
            self.fields
                .iter()
                .map(|field| format!("{}.{}", field.table_name.as_ref(), field.name.as_ref()))
                .join(", "),
        )
    }
}

impl<'a> ModelChangeActor for CreateTableModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        let mut sql = SqlQuery::from("CREATE TABLE IF NOT EXISTS ");

        sql.push_str(self.table);
        sql.push_str("(\n\t");

        let mut primary_keys: Vec<String> = Vec::new();

        for (index, field) in self.fields.iter().enumerate() {
            if field.flags.has_flag(FieldFlag::PrimaryKey) {
                primary_keys.push(String::from(&field.name))
            }

            sql.push_str(&util::sql_query::gen_column_descriptor_sql(field));

            if index < self.fields.len() - 1 {
                sql.push_str(",\n\t");
            }
        }

        if !primary_keys.is_empty() {
            sql.push_str(", \n\t");

            sql.push_str("PRIMARY KEY (");
            sql.push_str(&primary_keys.into_iter().join(", "));
            sql.push(')');
            sql.push('\n');
        } else {
            sql.push('\n');
        }

        sql.push(')');

        Ok(sql)
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!("DROP TABLE {}", self.entry.table))
    }

    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        self.fields
            .iter()
            .map(|field| FieldDependency {
                table_name: field.table_name.as_ref(),
                name: field.name.as_ref(),
                state: FieldState::Existing,
            })
            .collect()
    }

    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        let mut dependencies = Vec::new();

        for field in self.fields.iter() {
            if !field.flags.has_flag(FieldFlag::ForeignKey) {
                continue;
            }

            let PortablePgType::ForeignKey {
                ref target_table_name,
                ref target_column_name,
                ..
            } = field.pg_type
            else {
                unreachable!("unexpected field state, hee hee :/")
            };

            dependencies.push(FieldDependency {
                table_name: target_table_name.as_ref(),
                name: target_column_name.as_ref(),
                state: FieldState::Existing,
            })
        }

        dependencies.into_boxed_slice()
    }
}
