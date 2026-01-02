use crate::internals::AsPgType;
use crate::sort::{SortDefinition, SortDirection, SortOp};
use crate::ModelSidecar;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

pub use gas_shared::field::*;

#[derive(Debug)]
pub struct Field<T: AsPgType, M: ModelSidecar> {
    pub meta: FieldMeta,
    _marker: PhantomData<T>,
    _model_marker: PhantomData<M>,
}

pub enum VirtualFieldType {
    InverseRelation,
}

pub struct VirtualField<M: ModelSidecar> {
    pub field_type: VirtualFieldType,
    pub meta: FieldMeta,
    _model_marker: PhantomData<M>,
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

impl<M: ModelSidecar> VirtualField<M> {
    pub const fn new(field_type: VirtualFieldType, meta: FieldMeta) -> Self {
        Self {
            field_type,
            meta,
            _model_marker: PhantomData,
        }
    }
}

impl<M: ModelSidecar> Deref for VirtualField<M> {
    type Target = FieldMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}
