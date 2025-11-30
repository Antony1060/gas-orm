use crate::ModelMeta;

#[derive(Clone, Debug)]
pub struct InverseRelation<M: ModelMeta> {
    items: Vec<M>,
}
