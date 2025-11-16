use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum PgParam {
    TEXT(String),
    SMALLINT(i16),
    INTEGER(i32),
    BIGINT(i64),
    REAL(f32),
    DOUBLE(f64),
    DECIMAL(Decimal),
    NULL,
}

// very good 👍
macro_rules! pg_param_all {
    ($param:ident, $ex:expr) => {
        match $param {
            PgParam::TEXT(value) => $ex("TEXT", value),
            PgParam::SMALLINT(value) => $ex("SMALLINT", value),
            PgParam::INTEGER(value) => $ex("INTEGER", value),
            PgParam::BIGINT(value) => $ex("BIGINT", value),
            PgParam::REAL(value) => $ex("REAL", value),
            PgParam::DOUBLE(value) => $ex("DOUBLE", value),
            PgParam::DECIMAL(value) => $ex("DECIMAL", value),
            PgParam::NULL => $ex("NULL", Option::<i8>::None),
        }
    };
}

pub(crate) use pg_param_all;

impl Display for PgParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        pg_param_all!(self, |variant, value| {
            write!(f, "PgParams::{}({:?})", variant, value)
        })
    }
}
