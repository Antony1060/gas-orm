use crate::internals::PgType;
use crate::sort::{SortDefinition, SortDirection, SortOp};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Debug)]
pub enum FieldRelationship {
    OneToOne,
    OneToMany,
    ManyToMany,
}

#[repr(u8)]
pub enum FieldFlag {
    Nullable = 1 << 0,
    PrimaryKey = 1 << 1,
    Unique = 1 << 2,
    Serial = 1 << 3,
}

pub struct FieldFlags(pub u8);

impl FieldFlags {
    pub fn has_flag(&self, flag: FieldFlag) -> bool {
        (self.0 & (flag as u8)) != 0
    }
}

impl Debug for FieldFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:b}", self.0)
    }
}

#[derive(Debug)]
pub struct FieldMeta {
    // a lot of names
    pub full_name: &'static str,   // table.column
    pub name: &'static str,        // column
    pub alias_name: &'static str,  // table_column
    pub struct_name: &'static str, // table_column
    pub pg_type: PgType,
    pub flags: FieldFlags,
    pub relationship: Option<FieldRelationship>,
}

#[derive(Debug)]
pub struct Field<T> {
    pub meta: FieldMeta,
    _marker: PhantomData<T>,
}

impl<T> Field<T> {
    pub const fn new(meta: FieldMeta) -> Self {
        Self {
            meta,
            _marker: PhantomData,
        }
    }

    pub fn asc(&self) -> SortDefinition {
        SortDefinition::from(SortOp {
            field_full_name: self.full_name,
            direction: SortDirection::Ascending,
        })
    }

    pub fn desc(&self) -> SortDefinition {
        SortDefinition::from(SortOp {
            field_full_name: self.full_name,
            direction: SortDirection::Descending,
        })
    }
}

impl<T> Deref for Field<T> {
    type Target = FieldMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}
