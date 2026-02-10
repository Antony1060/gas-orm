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
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct PgConnection {
    pool: Arc<PgPool>,
}

#[derive(Clone)]
pub struct PgTransaction {
    connection: PgConnection,
    // eeh, I don't like this, kinda forces tokio and also arc + mutex overhead
    //  fine for now
    transaction: Arc<Mutex<sqlx::postgres::PgTransaction<'static>>>,
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
        Ok(PgTransaction {
            connection: self.clone(),
            transaction: Arc::from(Mutex::from(self.transaction_raw().await?)),
        })
    }

    async fn transaction_raw(&self) -> GasResult<sqlx::postgres::PgTransaction<'static>> {
        Ok(self.pool.begin().await?)
    }
}

impl PgTransaction {
    async fn replace_tx(&self) -> GasResult<sqlx::postgres::PgTransaction<'static>> {
        let new_tx = self.connection.transaction_raw().await?;
        let mut curr_tx = self.transaction.lock().await;

        let tx = mem::replace(&mut *curr_tx, new_tx);

        Ok(tx)
    }

    pub async fn save(&self) -> GasResult<()> {
        let tx = self.replace_tx().await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn discard(&self) -> GasResult<()> {
        let tx = self.replace_tx().await?;
        tx.rollback().await?;

        Ok(())
    }
}

pub(crate) trait PgExecutor: Sized {
    async fn execute(self, sql: SqlQuery, params: &[PgParam]) -> GasResult<Vec<Row>>;

    fn get_backing_connection(&self) -> PgConnection;

    async fn execute_parsed<T: FromRow>(
        self,
        sql: SqlQuery<'_>,
        params: &[PgParam],
    ) -> GasResult<Vec<T>> {
        let connection = self.get_backing_connection();

        let rows = self.execute(sql, params).await?;

        tokio::task::spawn_blocking(move || {
            let ctx = ResponseCtx {
                all_rows: &rows,
                connection,
            };

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

impl PgExecutor for &PgConnection {
    async fn execute(self, sql: SqlQuery<'_>, params: &[PgParam]) -> GasResult<Vec<Row>> {
        let (query, arguments) = Self::prepare_query(sql, params)?;

        let rows = sqlx::query_with(&query, arguments)
            .fetch_all(self.pool.as_ref())
            .await?;

        Ok(rows.into_iter().map(Row::from).collect())
    }

    fn get_backing_connection(&self) -> PgConnection {
        PgConnection::clone(self)
    }
}

impl PgExecutor for &PgTransaction {
    async fn execute(self, sql: SqlQuery<'_>, params: &[PgParam]) -> GasResult<Vec<Row>> {
        let (query, arguments) = Self::prepare_query(sql, params)?;

        let mut tx = self.transaction.lock().await;

        let rows = sqlx::query_with(&query, arguments)
            .fetch_all(&mut **tx)
            .await?;

        Ok(rows.into_iter().map(Row::from).collect())
    }

    fn get_backing_connection(&self) -> PgConnection {
        self.connection.clone()
    }
}

pub trait PgRawExecutor: PgExecutor {
    fn execute_raw(
        self,
        sql: SqlQuery<'_>,
        params: &[PgParam],
    ) -> impl Future<Output = GasResult<Vec<Row>>> {
        self.execute(sql, params)
    }
}

impl<T: PgExecutor> PgRawExecutor for T {}
