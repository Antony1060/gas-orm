use crate::ModelMeta;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct InverseRelation<Ret, const FORWARD_FIELD_INDEX: usize> {
    items: Ret,
}

pub trait InverseInnerConverter {
    type ToInner;
    type ToModel;
}

impl<M: ModelMeta> InverseInnerConverter for Vec<M> {
    type ToInner = Vec<Arc<M>>;
    type ToModel = M;
}

impl<M: ModelMeta> InverseInnerConverter for M {
    type ToInner = Option<Arc<M>>;
    type ToModel = M;
}
