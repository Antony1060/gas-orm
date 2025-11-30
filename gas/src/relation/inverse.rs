use crate::ModelMeta;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct InverseRelation<Ret, const FORWARD_FIELD_INDEX: usize> {
    items: Ret,
}

pub trait InverseRelationConverter {
    type ToInverseRelation<const FORWARD_FIELD_INDEX: usize>;
    type ToModel: ModelMeta;
}

impl<M: ModelMeta> InverseRelationConverter for Vec<M> {
    type ToInverseRelation<const FORWARD_FIELD_INDEX: usize> =
        InverseRelation<Vec<Arc<M>>, FORWARD_FIELD_INDEX>;
    type ToModel = M;
}

impl<M: ModelMeta> InverseRelationConverter for M {
    type ToInverseRelation<const FORWARD_FIELD_INDEX: usize> =
        InverseRelation<Option<Arc<M>>, FORWARD_FIELD_INDEX>;
    type ToModel = M;
}
