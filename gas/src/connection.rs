use crate::error::GasError;
use crate::sql_query::SqlQuery;
use crate::{pg_param_all, PgParams};
use sqlx::postgres::{PgArguments, PgPoolOptions};
use sqlx::Arguments;
use sqlx::PgPool;

pub type GasResult<T> = Result<T, GasError>;

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
    async fn execute(&self, sql: SqlQuery, params: &[PgParams]) -> GasResult<()>;
}

impl PgExecutionContext for PgConnection {
    async fn execute(&self, sql: SqlQuery, _params: &[PgParams]) -> GasResult<()> {
        let query = sql.finish()?;

        let mut arguments = PgArguments::default();
        for param in _params {
            // eh
            let res = pg_param_all!(param, |_, value| arguments.add(value));

            if let Err(_) = res {
                return Err(GasError::TypeError(param.clone()));
            }
        }

        // TODO:
        let rows = sqlx::query_with(&query, arguments)
            .fetch_all(&self.pool)
            .await?;
        dbg!(&rows);
        Ok(())
    }
}

impl PgExecutionContext for PgTransaction {
    async fn execute(&self, sql: SqlQuery, _params: &[PgParams]) -> GasResult<()> {
        let _query = sql.finish()?;
        let _a = &self.transaction; // mute warning for now
        todo!()
    }
}
