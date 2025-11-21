use crate::internals::AsPgType;
use crate::{GasResult, NaiveDecodable};
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
    fn from_row(row: &Row) -> GasResult<Self>;
}

pub trait FromRowNamed: Sized {
    fn from_row_named(row: &Row, name: &str) -> GasResult<Self>;
}

impl<T: AsPgType + NaiveDecodable> FromRowNamed for T {
    fn from_row_named(row: &Row, name: &str) -> GasResult<Self> {
        row.try_get::<T>(name)
    }
}
