use crate::connection::PgExecutionContext;
use crate::sql_query::SqlQuery;
use crate::{FieldFlags, GasResult, ModelMeta};
use std::marker::PhantomData;

// struct is kinda useless ngl
pub struct CreateOp<T: ModelMeta> {
    ignore_existing: bool,
    _marker: PhantomData<T>,
}

impl<T: ModelMeta> CreateOp<T> {
    pub fn new(if_not_exists: bool) -> Self {
        Self {
            ignore_existing: if_not_exists,
            _marker: PhantomData,
        }
    }

    pub async fn run<E: PgExecutionContext>(self, ctx: &E) -> GasResult<()> {
        let mut sql = SqlQuery::new("CREATE TABLE ");

        if self.ignore_existing {
            sql.append_str(" IF NOT EXISTS ");
        }

        sql.append_str(T::TABLE_NAME);
        sql.append_str("(");

        let mut primary_keys: Vec<String> = Vec::new();

        for field in T::FIELDS {
            if FieldFlags::PrimaryKey.in_bitmask(field.flags) {
                primary_keys.push(field.name.to_string())
            }

            let sql_type = field
                .pg_type
                .as_sql_type(FieldFlags::Serial.in_bitmask(field.flags));
            sql.append_str(field.name);
            sql.append_str(" ");
            sql.append_str(sql_type);

            if !FieldFlags::Nullable.in_bitmask(field.flags) {
                sql.append_str(" NOT NULL");
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

        sql.append_str(");");

        ctx.execute(sql, &[]).await.map(|_| ())
    }
}
