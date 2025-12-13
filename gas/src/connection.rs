use crate::error::GasError;
use crate::internals::SqlQuery;
use crate::internals::{pg_param_all, PgParam};
use crate::row::{FromRow, ResponseCtx, Row};
use crate::GasResult;
use sqlx::postgres::{PgArguments, PgPoolOptions};
use sqlx::Arguments;
use sqlx::PgPool;
use std::mem;
use std::sync::Arc;
use tokio::sync::OnceCell;

// crimes
pub(crate) static DEFAULT_CONNECTION: OnceCell<PgConnection> = OnceCell::const_new();

#[derive(Clone)]
pub struct PgConnection {
    pool: Arc<PgPool>,
}

pub struct PgTransaction {
    connection: PgConnection,
    transaction: sqlx::postgres::PgTransaction<'static>,
}

impl PgConnection {
    pub async fn new_connection_pool(connection_string: impl AsRef<str>) -> GasResult<Self> {
        let connection_string = connection_string.as_ref();

        let pool: PgPool = PgPoolOptions::new().connect(connection_string).await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub async fn transaction(&self) -> GasResult<PgTransaction> {
        let tx = self.pool.begin().await?;

        Ok(PgTransaction {
            connection: self.clone(),
            transaction: tx,
        })
    }
}

pub fn set_default_connection(connection: &PgConnection) {
    let Ok(_) = DEFAULT_CONNECTION.set(connection.clone()) else {
        panic!("cannot set default connection twice")
    };
}

pub fn get_default_connection() -> PgConnection {
    let Some(connection) = DEFAULT_CONNECTION.get() else {
        panic!("default connection not set")
    };

    connection.clone()
}

impl PgTransaction {
    pub async fn save(&mut self) -> GasResult<()> {
        let tx = mem::replace(self, self.connection.transaction().await?);
        tx.transaction.commit().await?;

        Ok(())
    }

    pub async fn discard(&mut self) -> GasResult<()> {
        let tx = mem::replace(self, self.connection.transaction().await?);
        tx.transaction.rollback().await?;

        Ok(())
    }
}

pub(crate) trait PgExecutionContext: Sized {
    async fn execute(self, sql: SqlQuery, params: &[PgParam]) -> GasResult<Vec<Row>>;

    async fn execute_parsed<T: FromRow>(
        self,
        sql: SqlQuery<'_>,
        params: &[PgParam],
    ) -> GasResult<Vec<T>> {
        let rows = self.execute(sql, params).await?;

        tokio::task::spawn_blocking(move || {
            let ctx = ResponseCtx { all_rows: &rows };

            rows.iter()
                .map(|row| FromRow::from_row(&ctx, row))
                .collect::<Result<Vec<T>, _>>()
        })
        .await
        .unwrap()
    }

    fn prepare_query(sql: SqlQuery, params: &[PgParam]) -> GasResult<(String, PgArguments)> {
        let query = sql.finish()?;

        tracing::trace!(sql = query, params = ?params, "executing query");

        let mut arguments = PgArguments::default();
        for param in params {
            let res = pg_param_all!(param, |_, value| arguments.add(value));

            if res.is_err() {
                return Err(GasError::TypeError(param.clone()));
            }
        }

        Ok((query, arguments))
    }
}

impl PgExecutionContext for &PgConnection {
    async fn execute(self, sql: SqlQuery<'_>, params: &[PgParam]) -> GasResult<Vec<Row>> {
        let (query, arguments) = Self::prepare_query(sql, params)?;

        let rows = sqlx::query_with(&query, arguments)
            .fetch_all(self.pool.as_ref())
            .await?;

        Ok(rows.into_iter().map(Row::from).collect())
    }
}

impl PgExecutionContext for &mut PgTransaction {
    async fn execute(self, sql: SqlQuery<'_>, params: &[PgParam]) -> GasResult<Vec<Row>> {
        let (query, arguments) = Self::prepare_query(sql, params)?;

        let rows = sqlx::query_with(&query, arguments)
            .fetch_all(&mut *self.transaction)
            .await?;

        Ok(rows.into_iter().map(Row::from).collect())
    }
}
