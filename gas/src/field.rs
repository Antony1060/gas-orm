use crate::internals::{AsPgType, PgType};
use crate::sort::{SortDefinition, SortDirection, SortOp};
use crate::ModelSidecar;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;

#[repr(u8)]
pub enum FieldFlag {
    Nullable = 1 << 0,
    PrimaryKey = 1 << 1,
    CompositePrimaryKey = 1 << 2,
    Unique = 1 << 3,
    Serial = 1 << 4,
}

pub struct FieldFlags(pub u8);

impl FieldFlags {
    pub const fn has_flag(&self, flag: FieldFlag) -> bool {
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
    pub table_name: &'static str,  // table
    pub full_name: &'static str,   // table.column
    pub name: &'static str,        // column
    pub alias_name: &'static str,  // table_column
    pub struct_name: &'static str, // table_column
    pub pg_type: PgType,
    pub flags: FieldFlags,
    pub index: usize,
}

#[derive(Debug)]
pub struct Field<T: AsPgType, M: ModelSidecar> {
    pub meta: FieldMeta,
    _marker: PhantomData<T>,
    _model_marker: PhantomData<M>,
}

pub enum VirtualFieldType {
    InverseRelation,
}

pub struct VirtualField {
    pub field_type: VirtualFieldType,
    pub meta: FieldMeta,
}

impl<T: AsPgType, M: ModelSidecar> Field<T, M> {
    pub const fn new(meta: FieldMeta) -> Self {
        Self {
            meta,
            _marker: PhantomData,
            _model_marker: PhantomData,
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

impl<T: AsPgType, M: ModelSidecar> Deref for Field<T, M> {
    type Target = FieldMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

impl Deref for VirtualField {
    type Target = FieldMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}
