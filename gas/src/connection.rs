use crate::error::GasError;
use crate::row::{FromRow, Row};
use crate::sql_query::SqlQuery;
use crate::{pg_param_all, GasResult, PgParams};
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
    async fn execute<T: FromRow>(&self, sql: SqlQuery, params: &[PgParams]) -> GasResult<Vec<T>>;
}

impl PgExecutionContext for PgConnection {
    async fn execute<T: FromRow>(&self, sql: SqlQuery, _params: &[PgParams]) -> GasResult<Vec<T>> {
        let query = sql.finish()?;

        dbg!(&query);

        let mut arguments = PgArguments::default();
        for param in _params {
            // eh
            let res = pg_param_all!(param, |_, value| arguments.add(value));

            if res.is_err() {
                return Err(GasError::TypeError(param.clone()));
            }
        }

        let rows = sqlx::query_with(&query, arguments)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(Row::from)
            .map(|row| FromRow::from_row(&row))
            .collect::<Result<Vec<T>, _>>()
    }
}

impl PgExecutionContext for PgTransaction {
    async fn execute<T: FromRow>(&self, sql: SqlQuery, _params: &[PgParams]) -> GasResult<Vec<T>> {
        let _query = sql.finish()?;
        let _a = &self.transaction; // mute warning for now
        todo!()
    }
}
