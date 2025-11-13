#![allow(private_bounds)]

pub use gas_macros::*;
use std::fmt::{Display, Formatter};

use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::ops::create::CreateOp;
use crate::ops::select::SelectBuilder;
use crate::pg_type::PgType;
use crate::row::FromRow;
use crate::sql_query::SqlQuery;
use rust_decimal::Decimal;
use std::marker::PhantomData;
use std::ops::Deref;

pub mod condition;
pub mod connection;
pub mod eq;
pub mod error;
mod ops;
pub mod pg_type;
pub mod row;
mod sql_query;
pub mod types;

pub type GasResult<T> = Result<T, GasError>;

#[derive(Debug, Clone)]
pub enum PgParam {
    TEXT(String),
    SMALLINT(i16),
    INTEGER(i32),
    BIGINT(i64),
    REAL(f32),
    DOUBLE(f64),
    DECIMAL(Decimal),
}

// very good 👍
macro_rules! pg_param_all {
    ($param:ident, $ex:expr) => {
        match $param {
            PgParam::TEXT(value) => $ex("TEXT", value),
            PgParam::SMALLINT(value) => $ex("SMALLINT", value),
            PgParam::INTEGER(value) => $ex("INTEGER", value),
            PgParam::BIGINT(value) => $ex("BIGINT", value),
            PgParam::REAL(value) => $ex("REAL", value),
            PgParam::DOUBLE(value) => $ex("DOUBLE", value),
            PgParam::DECIMAL(value) => $ex("DECIMAL", value),
        }
    };
}

pub(crate) use pg_param_all;

impl Display for PgParam {
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

impl FieldFlags {
    pub fn in_bitmask(self, mask: u8) -> bool {
        (mask & (self as u8)) != 0
    }
}

#[derive(Debug)]
pub struct FieldMeta {
    // a lot of names
    pub full_name: &'static str,  // table.column
    pub name: &'static str,       // column
    pub alias_name: &'static str, // table_column
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
        // eh
        full_name: &'static str,
        name: &'static str,
        alias_name: &'static str,
        pg_type: PgType,
        flags: u8,
        relationship: Option<Relationship>,
    ) -> Self {
        Self {
            meta: FieldMeta {
                full_name,
                name,
                alias_name,
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
    const TABLE_NAME: &'static str;
    const FIELDS: &'static [FieldMeta];
}

// NOTE: maybe add ByKeyOps<T: ModelMeta, Key> that will implement find_by_key, delete_by_key and update_by_key
//  update_by_key would probably be used something like
//  ```
//  user::Model {
//      username: "user1234".to_string(),
//      ..user::default()
//  }.update_by_key(key: K) // insert would be similar
//  ```
//
// NOTE 2: maybe add aliases for all of these in the root of the namespace,
//  so it can be used like user::query() or user::create_table()
pub trait ModelOps<T: ModelMeta> {
    fn query() -> SelectBuilder<T> {
        SelectBuilder::new()
    }

    // some trait bounds cannot be enforced if I just do `async fn` here, idk
    fn create_table<E: PgExecutionContext>(
        ctx: &E,
        ignore_existing: bool,
    ) -> impl Future<Output = GasResult<()>> {
        CreateOp::<T>::new(ignore_existing).run(ctx)
    }

    fn update<E: PgExecutionContext>(&self, _ctx: &E) -> impl Future<Output = GasResult<()>> {
        async { todo!() }
    }

    fn delete<E: PgExecutionContext>(&self, _ctx: &E) -> impl Future<Output = GasResult<()>> {
        async { todo!() }
    }
}

impl<T: ModelMeta> ModelOps<T> for T {}
