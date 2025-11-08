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

pub struct Field<T> {
    pub name: &'static str,
    pub pg_type: PgType,
    _marker: PhantomData<T>,
}

impl<T> Field<T> {
    pub const fn new(name: &'static str, pg_type: PgType) -> Self {
        Self {
            name,
            pg_type,
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
