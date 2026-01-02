use crate::connection::PgConnection;
use crate::internals::{AsPgType, NaiveDecodable};
use crate::GasResult;
use sqlx::postgres::PgRow;
use sqlx::Row as SqlxRow;

pub struct Row {
    pg_row: PgRow,
}

impl From<PgRow> for Row {
    fn from(pg_row: PgRow) -> Self {
        Row { pg_row }
    }
}

impl Row {
    pub fn try_get<T>(&self, index: &str) -> GasResult<T>
    where
        T: AsPgType + NaiveDecodable,
    {
        Ok(self.pg_row.try_get::<T, &str>(index)?)
    }
}

pub trait FromRow: Sized + Send + 'static {
    fn from_row(ctx: &ResponseCtx, row: &Row) -> GasResult<Self>;
}

pub struct ResponseCtx<'a> {
    // a reference to a connection that initiated the request
    //  if it was with a &PgConnection itself, it will be that
    //  if it was with a &mut PgTransaction, it will be with a connection that made the transaction
    //  This field is useful sometimes if additional selects are needed while handling the from_row,
    //      concretely by the InverseRelation (currently implemented in a very bad way)
    //  That being said, don't use the connection for anything other than selects (without side effects)
    // NOTE: this currently has bug where when data changed in a transaction, the select that
    //  queries the database will partially have new data, and partially not,
    //  depending on if the query included an inverse field
    // TODO (low priority): use the actual PgExecutor implementor
    pub(crate) connection: PgConnection,
    pub all_rows: &'a [Row],
}

pub trait FromRowNamed: Sized + Send + 'static {
    fn from_row_named(ctx: &ResponseCtx, row: &Row, name: &str) -> GasResult<Self>;
}

impl<T: AsPgType + NaiveDecodable> FromRowNamed for T {
    fn from_row_named(_ctx: &ResponseCtx, row: &Row, name: &str) -> GasResult<Self> {
        row.try_get::<T>(name)
    }
}
