use crate::internals::{AsPgType, IsOptional, PgParam, PgType};
use crate::row::{FromRowNamed, Row};
use crate::{GasResult, ModelMeta};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct InverseRelation<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize> {
    #[allow(dead_code)]
    items: Ret,
}

pub enum InverseRelationType {
    ToOne,
    ToMany,
}

pub trait InverseRelationTypeOps {
    type Inner: Clone + Default;
    type Model;
    const TYPE: InverseRelationType;
}

impl<M: ModelMeta> InverseRelationTypeOps for Vec<M> {
    type Inner = Vec<Arc<M>>;
    type Model = M;
    const TYPE: InverseRelationType = InverseRelationType::ToMany;
}

impl<M: ModelMeta> InverseRelationTypeOps for Option<M> {
    type Inner = Option<Arc<M>>;
    type Model = M;
    const TYPE: InverseRelationType = InverseRelationType::ToOne;
}

impl<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize> AsPgType
    for InverseRelation<Ret, FORWARD_FIELD_INDEX>
{
    const PG_TYPE: PgType = PgType::IGNORED;
}

impl<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize> IsOptional
    for InverseRelation<Ret, FORWARD_FIELD_INDEX>
{
    const FACTOR: u8 = 0;
}

impl<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize>
    From<InverseRelation<Ret, FORWARD_FIELD_INDEX>> for PgParam
{
    fn from(_value: InverseRelation<Ret, FORWARD_FIELD_INDEX>) -> Self {
        PgParam::IGNORED
    }
}

impl<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize> FromRowNamed
    for InverseRelation<Ret, FORWARD_FIELD_INDEX>
{
    fn from_row_named(_row: &Row, _name: &str) -> GasResult<Self> {
        // TODO:
        Ok(Self {
            items: Ret::default(),
        })
    }
}
