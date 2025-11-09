pub use gas_macros::*;

use crate::builder::SelectBuilder;
use crate::condition::EqExpression;
use crate::pg_type::PgType;
use rust_decimal::Decimal;
use std::marker::PhantomData;

pub mod builder;
pub mod condition;
pub mod eq;
pub mod pg_type;
pub mod types;

#[derive(Debug, Clone)]
pub enum PgParams {
    TEXT(String),
    SMALLINT(i16),
    INTEGER(i32),
    BIGINT(i64),
    REAL(f32),
    DOUBLE(f64),
    DECIMAL(Decimal),
}

pub enum Relationship {
    OneToOne,
    OneToMany,
    ManyToMany,
}

#[repr(u8)]
pub enum FieldFlags {
    PrimaryKey = 0,
    Serial,
    Nullable,
}

pub struct Field<T> {
    pub name: &'static str,
    pub pg_type: PgType,
    pub flags: u8,
    pub relationship: Option<Relationship>,
    _marker: PhantomData<T>,
}

impl<T> Field<T> {
    pub const fn new(
        name: &'static str,
        pg_type: PgType,
        flags: u8,
        relationship: Option<Relationship>,
    ) -> Self {
        Self {
            name,
            pg_type,
            flags,
            relationship,
            _marker: PhantomData,
        }
    }
}

pub trait AsSql {
    fn as_sql(&self) -> String;
}

pub trait ModelOps {
    fn table_name() -> &'static str;

    fn filter(cond_fn: fn() -> EqExpression) -> SelectBuilder {
        SelectBuilder {
            table: Self::table_name(),
            filter: Some(cond_fn()),
        }
    }
}
