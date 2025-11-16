use crate::internals::PgParam;
use crate::types::Decimal;
use sqlx::{Decode, Type};

#[derive(Debug)]
pub enum PgType {
    TEXT,
    SMALLINT,
    INTEGER,
    BIGINT,
    REAL,
    DOUBLE,
    DECIMAL,
}

impl PgType {
    pub const fn __to_pg_type<T: AsPgType>() -> PgType {
        T::PG_TYPE
    }

    pub fn as_sql_type(&self, is_serial: bool) -> &'static str {
        match self {
            PgType::TEXT => "TEXT",
            PgType::SMALLINT if is_serial => "SMALLSERIAL",
            PgType::SMALLINT => "SMALLINT",
            PgType::INTEGER if is_serial => "SERIAL",
            PgType::INTEGER => "INTEGER",
            PgType::BIGINT if is_serial => "BIGSERIAL",
            PgType::BIGINT => "BIGINT",
            PgType::REAL => "REAL",
            PgType::DOUBLE => "DOUBLE",
            PgType::DECIMAL => "DECIMAL",
        }
    }
}

pub trait AsPgType: for<'a> Decode<'a, sqlx::Postgres> + Type<sqlx::Postgres> {
    const PG_TYPE: PgType;
}

pub trait IsOptional {
    const FACTOR: u8;
}

impl<T: AsPgType> IsOptional for Option<T> {
    const FACTOR: u8 = 1;
}

macro_rules! pg_type_impl {
    ($field_type:ty as $pg_type:expr, $pg_param_conv:expr) => {
        impl AsPgType for $field_type {
            const PG_TYPE: PgType = $pg_type;
        }

        impl AsPgType for Option<$field_type> {
            const PG_TYPE: PgType = $pg_type;
        }

        // default to 0, blanked implemented to 1 for all Option<T: AsPgType>
        impl IsOptional for $field_type {
            const FACTOR: u8 = 0;
        }

        impl From<$field_type> for PgParam {
            fn from(value: $field_type) -> Self {
                $pg_param_conv(value)
            }
        }

        impl From<Option<$field_type>> for PgParam {
            fn from(value: Option<$field_type>) -> Self {
                match value {
                    Some(value) => PgParam::from(value),
                    None => PgParam::NULL,
                }
            }
        }
    };
}

pg_type_impl!(String as PgType::TEXT, PgParam::TEXT);
pg_type_impl!(i16 as PgType::SMALLINT, PgParam::SMALLINT);
pg_type_impl!(i32 as PgType::INTEGER, PgParam::INTEGER);
pg_type_impl!(i64 as PgType::BIGINT, PgParam::BIGINT);
pg_type_impl!(f32 as PgType::REAL, PgParam::REAL);
pg_type_impl!(f64 as PgType::DOUBLE, PgParam::DOUBLE);
pg_type_impl!(Decimal as PgType::DECIMAL, PgParam::DECIMAL);
