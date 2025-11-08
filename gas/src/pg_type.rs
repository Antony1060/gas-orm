use crate::types::Decimal;

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

impl AsPgType for String {
    const PG_TYPE: PgType = PgType::TEXT;
}

impl AsPgType for i16 {
    const PG_TYPE: PgType = PgType::SMALLINT;
}

impl AsPgType for i32 {
    const PG_TYPE: PgType = PgType::INTEGER;
}

impl AsPgType for i64 {
    const PG_TYPE: PgType = PgType::BIGINT;
}

impl AsPgType for f32 {
    const PG_TYPE: PgType = PgType::REAL;
}

impl AsPgType for f64 {
    const PG_TYPE: PgType = PgType::DOUBLE;
}

impl AsPgType for Decimal {
    const PG_TYPE: PgType = PgType::DECIMAL;
}
