use crate::error::GasError;
use crate::sql_query::SqlQuery;
use crate::PgParams;
use sqlx::postgres::PgPoolOptions;
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

pub trait PgExecutionContext {
    async fn execute(&self, sql: SqlQuery, params: &[PgParams]) -> GasResult<()>;
}
