use crate::internals::{AsPgType, PgType};
use crate::sort::{SortDefinition, SortDirection, SortOp};
use crate::ModelMeta;
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
    pub table_name: &'static str,
    pub full_name: &'static str,   // table.column
    pub name: &'static str,        // column
    pub alias_name: &'static str,  // table_column
    pub struct_name: &'static str, // table_column
    pub pg_type: PgType,
    pub flags: FieldFlags,
    pub relationship: Option<FieldRelationship>,
}

#[derive(Debug)]
pub struct Field<T: AsPgType, M: ModelMeta> {
    pub meta: FieldMeta,
    pub index: usize,
    _marker: PhantomData<T>,
    _model_marker: PhantomData<M>,
}

impl<T: AsPgType, M: ModelMeta> Field<T, M> {
    pub const fn new(meta: FieldMeta, index: usize) -> Self {
        Self {
            meta,
            index,
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

impl<T: AsPgType, M: ModelMeta> Deref for Field<T, M> {
    type Target = FieldMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

pub trait FieldTypeAccessor {
    type VALUE: AsPgType;
    type MODEL: ModelMeta;
}

impl<T: AsPgType, M: ModelMeta> FieldTypeAccessor for Field<T, M> {
    type VALUE = T;
    type MODEL = M;
}
