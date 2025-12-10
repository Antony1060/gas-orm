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

pub trait FromRow: Sized {
    fn from_row(ctx: &ResponseCtx, row: &Row) -> GasResult<Self>;
}

pub struct ResponseCtx<'a> {
    pub all_rows: &'a [Row],
}

pub trait FromRowNamed: Sized {
    fn from_row_named(ctx: &ResponseCtx, row: &Row, name: &str) -> GasResult<Self>;
}

impl<T: AsPgType + NaiveDecodable> FromRowNamed for T {
    fn from_row_named(_ctx: &ResponseCtx, row: &Row, name: &str) -> GasResult<Self> {
        row.try_get::<T>(name)
    }
}
