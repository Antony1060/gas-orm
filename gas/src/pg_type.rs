use crate::types::Decimal;

pub enum PgType {
    TEXT,
    INT,
    DECIMAL,
    FLOAT,
}

pub trait AsPgType {
    fn as_pg_type() -> PgType;
}

impl AsPgType for String {
    fn as_pg_type() -> PgType {
        PgType::TEXT
    }
}

impl AsPgType for i32 {
    fn as_pg_type() -> PgType {
        PgType::INT
    }
}

impl AsPgType for f32 {
    fn as_pg_type() -> PgType {
        PgType::FLOAT
    }
}

impl AsPgType for Decimal {
    fn as_pg_type() -> PgType {
        PgType::DECIMAL
    }
}
