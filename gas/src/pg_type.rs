use crate::types::Decimal;

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
}

pub trait AsPgType {
    const PG_TYPE: PgType;
}

pub trait IsOptional {
    const FACTOR: u8;
}

impl<T: AsPgType> IsOptional for Option<T> {
    const FACTOR: u8 = 1;
}

macro_rules! pg_type_impl {
    ($field_type:ty as $pg_type:expr) => {
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
    };
}

pg_type_impl!(String as PgType::TEXT);
pg_type_impl!(i16 as PgType::SMALLINT);
pg_type_impl!(i32 as PgType::INTEGER);
pg_type_impl!(i64 as PgType::BIGINT);
pg_type_impl!(f32 as PgType::REAL);
pg_type_impl!(f64 as PgType::DOUBLE);
pg_type_impl!(Decimal as PgType::DECIMAL);
