pub use gas_macros::*;
use std::fmt::{Display, Formatter};

use crate::builder::SelectBuilder;
use crate::error::GasError;
use crate::pg_type::PgType;
use crate::row::FromRow;
use crate::sql_query::SqlQuery;
use rust_decimal::Decimal;
use std::marker::PhantomData;
use std::ops::Deref;

pub mod builder;
pub mod condition;
pub mod connection;
pub mod eq;
pub mod error;
pub mod pg_type;
pub mod row;
mod sql_query;
pub mod types;

pub type GasResult<T> = Result<T, GasError>;

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

#[macro_export]
macro_rules! pg_param_all {
    ($param:ident, $ex:expr) => {
        match $param {
            PgParams::TEXT(value) => $ex("TEXT", value),
            PgParams::SMALLINT(value) => $ex("SMALLINT", value),
            PgParams::INTEGER(value) => $ex("INTEGER", value),
            PgParams::BIGINT(value) => $ex("BIGINT", value),
            PgParams::REAL(value) => $ex("REAL", value),
            PgParams::DOUBLE(value) => $ex("DOUBLE", value),
            PgParams::DECIMAL(value) => $ex("DECIMAL", value),
        }
    };
}

impl Display for PgParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        pg_param_all!(self, |variant, value| {
            write!(f, "PgParams::{}({})", variant, value)
        })
    }
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

pub(crate) trait AsSql {
    fn as_sql(&self) -> SqlQuery;
}

pub trait ModelMeta: FromRow {
    fn table_name() -> &'static str;
}

pub trait ModelOps<T: ModelMeta> {
    fn query() -> SelectBuilder<T> {
        SelectBuilder::new()
    }
}

impl<T: ModelMeta> ModelOps<T> for T {}
