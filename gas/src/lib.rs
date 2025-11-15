#![allow(private_bounds)]
#![allow(private_interfaces)]

pub use gas_macros::*;

use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::ops::create::CreateOp;
use crate::ops::delete::DeleteOp;
use crate::ops::insert::InsertOp;
use crate::ops::select::SelectBuilder;
use crate::ops::update::UpdateOp;
use crate::pg_type::PgType;
use crate::row::FromRow;
use crate::sql_query::{SqlQuery, SqlStatement};
use std::marker::PhantomData;
use std::ops::Deref;

pub mod condition;
pub mod connection;
pub mod eq;
pub mod error;
mod ops;
pub mod pg_param;
pub mod pg_type;
pub mod row;
pub mod sql_query;
pub mod types;

pub type GasResult<T> = Result<T, GasError>;

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

pub trait ModelMeta: Sized + FromRow {
    const TABLE_NAME: &'static str;
    const FIELDS: &'static [FieldMeta];

    fn gen_insert_sql(&self) -> SqlStatement;

    fn gen_update_sql(&self) -> SqlStatement;

    fn gen_delete_sql(&self) -> SqlStatement;
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
pub trait ModelOps: ModelMeta {
    fn query() -> SelectBuilder<Self> {
        SelectBuilder::new()
    }

    // some trait bounds cannot be enforced if I just do `async fn` here, idk
    fn create_table<E: PgExecutionContext>(
        ctx: &E,
        ignore_existing: bool,
    ) -> impl Future<Output = GasResult<()>> {
        CreateOp::<Self>::new(ignore_existing).run(ctx)
    }

    // consume self and return an entry that is inserted
    fn insert<E: PgExecutionContext>(&mut self, ctx: &E) -> impl Future<Output = GasResult<()>> {
        InsertOp::<Self>::new(self).run(ctx)
    }

    fn update<E: PgExecutionContext>(&mut self, ctx: &E) -> impl Future<Output = GasResult<()>> {
        UpdateOp::<Self>::new(self).run(ctx)
    }

    fn delete<E: PgExecutionContext>(self, ctx: &E) -> impl Future<Output = GasResult<()>> {
        DeleteOp::<Self>::new(self).run(ctx)
    }
}

impl<T: ModelMeta> ModelOps for T {}
