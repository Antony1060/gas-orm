use crate::internals::PgParam;
use crate::row::FromRowNamed;
use crate::types::Decimal;
use crate::FieldMeta;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
#[cfg(feature = "orm")]
use sqlx::{Decode, Postgres, Type};
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum PgType {
    TEXT,
    SMALLINT,
    INTEGER,
    BIGINT,
    REAL,
    DOUBLE,
    DECIMAL,
    TIMESTAMP,
    #[allow(nonstandard_style)]
    TIMESTAMP_TZ,
    DATE,
    TIME,
    #[allow(nonstandard_style)]
    FOREIGN_KEY {
        key_type: &'static PgType,
        target_field: &'static FieldMeta,
    },
    IGNORED,
}

impl PgType {
    pub fn as_sql_type(&self, is_serial: bool) -> Cow<'static, str> {
        match self {
            PgType::FOREIGN_KEY {
                key_type,
                target_field,
            } => format!(
                "{} REFERENCES {}({})",
                key_type.as_sql_type(false),
                target_field.table_name,
                target_field.name
            )
            .into(),
            _ => unsafe { self.as_sql_type_const(is_serial) }.into(),
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub const unsafe fn as_sql_type_const(&self, is_serial: bool) -> &'static str {
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
            PgType::TIMESTAMP => "TIMESTAMP",
            PgType::TIMESTAMP_TZ => "TIMESTAMP WITH TIME ZONE",
            PgType::DATE => "DATE",
            PgType::TIME => "TIME",
            PgType::IGNORED => "",
            _ => panic!("unhandled state"),
        }
    }
}

pub trait AsPgType: Clone + Default + FromRowNamed {
    const PG_TYPE: PgType;
}

pub trait IsOptional {
    const FACTOR: u8;
}

impl<T: AsPgType> IsOptional for Option<T> {
    const FACTOR: u8 = 1;
}

#[cfg(feature = "orm")]
pub(crate) trait NaiveDecodable: for<'a> Decode<'a, Postgres> + Type<Postgres> {}
#[cfg(not(feature = "orm"))]
pub(crate) trait NaiveDecodable {}

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
