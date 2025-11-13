use crate::pg_type::AsPgType;
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
    pub fn try_get<T: AsPgType>(&self, index: usize) -> GasResult<T> {
        let a: T = self.pg_row.try_get(index)?;

        Ok(a)
    }
}

pub trait FromRow: Sized {
    fn from_row(row: &Row) -> GasResult<Self>;
}
