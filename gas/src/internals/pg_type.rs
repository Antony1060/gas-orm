use crate::internals::PgParam;
use crate::row::FromRowNamed;
use crate::types::Decimal;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use gas_shared::internals::pg_type::PgType;
use sqlx::{Decode, Postgres, Type};

pub trait AsPgType: Clone + Default + FromRowNamed {
    const PG_TYPE: PgType;
}

pub trait IsOptional {
    const FACTOR: u8;
}

impl<T: AsPgType> IsOptional for Option<T> {
    const FACTOR: u8 = 1;
}

pub(crate) trait NaiveDecodable: for<'a> Decode<'a, Postgres> + Type<Postgres> {}

macro_rules! pg_type_impl {
    ($field_type:ty as $pg_type:expr, $pg_param_conv:expr) => {
        impl AsPgType for $field_type {
            const PG_TYPE: PgType = $pg_type;
        }

        impl AsPgType for Option<$field_type> {
            const PG_TYPE: PgType = $pg_type;
        }

        impl NaiveDecodable for $field_type {}
        impl NaiveDecodable for Option<$field_type> {}

        // default to 0, blanked implemented to 1 for all Option<T: AsPgType>
        impl IsOptional for $field_type {
            const FACTOR: u8 = 0;
        }

        impl From<$field_type> for PgParam {
            fn from(value: $field_type) -> Self {
                PgParam::from(Some(value))
            }
        }

        impl From<Option<$field_type>> for PgParam {
            fn from(value: Option<$field_type>) -> Self {
                $pg_param_conv(value)
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
pg_type_impl!(NaiveDateTime as PgType::TIMESTAMP, PgParam::TIMESTAMP);

pg_type_impl!(DateTime<Utc> as PgType::TIMESTAMP_TZ, PgParam::TIMESTAMP_TZ_UTC);
pg_type_impl!(DateTime<Local> as PgType::TIMESTAMP_TZ, PgParam::TIMESTAMP_TZ_LOCAL);
pg_type_impl!(DateTime<FixedOffset> as PgType::TIMESTAMP_TZ, PgParam::TIMESTAMP_TZ_FIXED_OFFSET);

pg_type_impl!(NaiveDate as PgType::DATE, PgParam::DATE);
pg_type_impl!(NaiveTime as PgType::TIME, PgParam::TIME);
