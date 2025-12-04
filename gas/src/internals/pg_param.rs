use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum PgParam {
    TEXT(Option<String>),
    SMALLINT(Option<i16>),
    INTEGER(Option<i32>),
    BIGINT(Option<i64>),
    REAL(Option<f32>),
    DOUBLE(Option<f64>),
    DECIMAL(Option<Decimal>),
    TIMESTAMP(Option<NaiveDateTime>),
    // I don't think there's a more ergonomic way to do this
    #[allow(nonstandard_style)]
    TIMESTAMP_TZ_UTC(Option<DateTime<Utc>>),
    #[allow(nonstandard_style)]
    TIMESTAMP_TZ_LOCAL(Option<DateTime<Local>>),
    #[allow(nonstandard_style)]
    TIMESTAMP_TZ_FIXED_OFFSET(Option<DateTime<FixedOffset>>),
    DATE(Option<NaiveDate>),
    TIME(Option<NaiveTime>),
    RAW(Option<&'static str>),
    IGNORED,
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
            PgParam::TIMESTAMP(value) => $ex("TIMESTAMP", value),
            PgParam::TIMESTAMP_TZ_UTC(value) => $ex("TIMESTAMP_TZ_UTC", value),
            PgParam::TIMESTAMP_TZ_LOCAL(value) => $ex("TIMESTAMP_TZ_LOCAL", value),
            PgParam::TIMESTAMP_TZ_FIXED_OFFSET(value) => $ex("TIMESTAMP_TZ_FIXED_OFFSET", value),
            PgParam::DATE(value) => $ex("DATE", value),
            PgParam::TIME(value) => $ex("TIME", value),
            PgParam::RAW(value) => $ex("RAW", value),
            PgParam::IGNORED => $ex("IGNORED", Option::<i8>::None),
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
