use crate::error::GasError;
use crate::pg_param::{pg_param_all, PgParam};
use crate::row::{FromRow, Row};
use crate::sql_query::SqlQuery;
use crate::GasResult;
use sqlx::postgres::{PgArguments, PgPoolOptions};
use sqlx::Arguments;
use sqlx::PgPool;

pub struct PgConnection {
    pool: PgPool,
}

pub struct PgTransaction {
    transaction: Option<sqlx::postgres::PgTransaction<'static>>,
}

impl PgConnection {
    pub async fn new_connection_pool(connection_string: impl AsRef<str>) -> GasResult<Self> {
        let connection_string = connection_string.as_ref();

        let pool: PgPool = PgPoolOptions::new().connect(connection_string).await?;

        Ok(Self { pool })
    }

    pub async fn transaction(&self) -> GasResult<PgTransaction> {
        let tx = self.pool.begin().await?;

        Ok(PgTransaction {
            transaction: Some(tx),
        })
    }
}

pub(crate) trait PgExecutionContext {
    async fn execute(&self, sql: SqlQuery, params: &[PgParam]) -> GasResult<Vec<Row>>;

    async fn execute_parsed<T: FromRow>(
        &self,
        sql: SqlQuery,
        params: &[PgParam],
    ) -> GasResult<Vec<T>> {
        let rows = self.execute(sql, params).await?;

        rows.into_iter()
            .map(|row| FromRow::from_row(&row))
            .collect::<Result<Vec<T>, _>>()
    }
}

impl PgExecutionContext for PgConnection {
    async fn execute(&self, sql: SqlQuery, params: &[PgParam]) -> GasResult<Vec<Row>> {
        let query = sql.finish()?;

        dbg!(&query);
        dbg!(&params);

        let mut arguments = PgArguments::default();
        for param in params {
            // eh
            let res = pg_param_all!(param, |_, value| arguments.add(value));

            if res.is_err() {
                return Err(GasError::TypeError(param.clone()));
            }
        }

        let rows = sqlx::query_with(&query, arguments)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Row::from).collect())
    }
}

impl PgExecutionContext for PgTransaction {
    async fn execute(&self, sql: SqlQuery, _params: &[PgParam]) -> GasResult<Vec<Row>> {
        let _query = sql.finish()?;
        let _a = &self.transaction; // mute warning for now
        todo!()
    }
}
