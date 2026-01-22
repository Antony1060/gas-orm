use crate::binary::FieldEntry;
use crate::error::GasCliResult;
use crate::sync::diff::ModelChangeActor;
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortablePgType;
use gas_shared::FieldFlag;
use std::borrow::Cow;
use std::ops::Deref;

pub struct CreateTableModelActor<'a> {
    pub entry: FieldEntry<'a>,
}

impl<'a> CreateTableModelActor<'a> {
    pub fn new_boxed(entry: FieldEntry) -> Box<dyn ModelChangeActor + '_> {
        Box::from(CreateTableModelActor { entry })
    }
}

impl<'a> Deref for CreateTableModelActor<'a> {
    type Target = FieldEntry<'a>;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl<'a> ModelChangeActor for CreateTableModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        let mut sql = SqlQuery::from("CREATE TABLE IF NOT EXISTS ");

        sql.push_str(self.table);
        sql.push('(');

        let mut primary_keys: Vec<String> = Vec::new();

        for (index, field) in self.fields.iter().enumerate() {
            if field.flags.has_flag(FieldFlag::PrimaryKey) {
                primary_keys.push(String::from(&field.name))
            }

            let sql_type: Cow<'_, str> = match field.pg_type {
                PortablePgType::Raw(ref pg_type) => {
                    pg_type.as_sql_type(field.flags.has_flag(FieldFlag::Serial))
                }
                PortablePgType::ForeignKey {
                    ref key_sql_type, ..
                } => Cow::from(key_sql_type.as_ref()),
            };

            sql.push_str(field.name.as_ref());
            sql.push(' ');
            sql.push_str(&sql_type);

            if !field.flags.has_flag(FieldFlag::Nullable) {
                sql.push_str(" NOT NULL");
            }

            if field.flags.has_flag(FieldFlag::Unique) {
                sql.push_str(" UNIQUE");
            }

            if index < self.fields.len() - 1 {
                sql.push_str(", ");
            }
        }

        if !primary_keys.is_empty() {
            sql.push_str(", ");

            sql.push_str("PRIMARY KEY (");
            sql.push_str(
                &primary_keys
                    .into_iter()
                    .reduce(|acc, curr| format!("{acc}, {curr}"))
                    .unwrap_or(String::new()),
            );
            sql.push(')');
        }

        sql.push(')');

        Ok(sql)
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok(format!("DROP TABLE {}", self.entry.table))
    }
}
