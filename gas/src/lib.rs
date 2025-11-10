pub use gas_macros::*;

use crate::builder::SelectBuilder;
use crate::condition::EqExpression;
use crate::pg_type::PgType;
use rust_decimal::Decimal;
use std::marker::PhantomData;
use std::ops::Deref;

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

#[derive(Debug)]
pub enum Relationship {
    OneToOne,
    OneToMany,
    ManyToMany,
}

#[repr(u8)]
pub enum FieldFlags {
    PrimaryKey = 1 << 0,
    Serial = 1 << 1,
    Nullable = 1 << 2,
}

#[derive(Debug)]
pub struct FieldMeta {
    pub name: &'static str,
    pub pg_type: PgType,
    pub flags: u8,
    pub relationship: Option<Relationship>,
}

#[derive(Debug)]
pub struct Field<T> {
    pub meta: FieldMeta,
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
            meta: FieldMeta {
                name,
                pg_type,
                flags,
                relationship,
            },
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Field<T> {
    type Target = FieldMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
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
