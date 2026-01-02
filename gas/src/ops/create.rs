use crate::connection::PgExecutor;
use crate::internals::SqlQuery;
use crate::model::ModelMeta;
use crate::{FieldFlag, GasResult};
use std::marker::PhantomData;

// struct is kinda useless ngl
pub(crate) struct CreateOp<T: ModelMeta> {
    ignore_existing: bool,
    _marker: PhantomData<T>,
}

impl<T: ModelMeta> CreateOp<T> {
    pub(crate) fn new(if_not_exists: bool) -> Self {
        Self {
            ignore_existing: if_not_exists,
            _marker: PhantomData,
        }
    }

    // could be at compile-time, but I don't care, it's create_table, who cares
    pub(crate) async fn run<E: PgExecutor>(self, ctx: E) -> GasResult<()> {
        let mut sql = SqlQuery::from("CREATE TABLE ");

        if self.ignore_existing {
            sql.append_str(" IF NOT EXISTS ");
        }

        sql.append_str(T::TABLE_NAME);
        sql.append_str("(");

        let mut primary_keys: Vec<String> = Vec::new();

        for field in T::FIELDS {
            if field.flags.has_flag(FieldFlag::PrimaryKey) {
                primary_keys.push(field.name.to_string())
            }

            let sql_type = field
                .pg_type
                .as_sql_type(field.flags.has_flag(FieldFlag::Serial));
            sql.append_str(field.name);
            sql.append_str(" ");
            sql.append_str(&sql_type);

            if !field.flags.has_flag(FieldFlag::Nullable) {
                sql.append_str(" NOT NULL");
            }

            if field.flags.has_flag(FieldFlag::Unique) {
                sql.append_str(" UNIQUE");
            }

            sql.append_str(", ");
        }

        if !primary_keys.is_empty() {
            sql.append_str("PRIMARY KEY (");
            sql.append_str(
                &primary_keys
                    .into_iter()
                    .reduce(|acc, curr| format!("{acc}, {curr}"))
                    .unwrap_or(String::new()),
            );
            sql.append_str(")");
        }

        sql.append_str(")");

        ctx.execute(sql, &[]).await.map(|_| ())
    }
}
