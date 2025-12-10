use crate::connection::PgExecutionContext;
use crate::internals::{AsPgType, IsOptional, PgParam, PgType};
use crate::row::{FromRowNamed, ResponseCtx, Row};
use crate::{GasResult, ModelMeta};
use std::ops::Deref;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InverseRelation<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize> {
    loaded: bool,
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

type ToManyContainer<M> = Box<[Arc<M>]>;
type ToOneContainer<M> = Option<Arc<M>>;

impl<M: ModelMeta> InverseRelationTypeOps for Vec<M> {
    type Inner = ToManyContainer<M>;
    type Model = M;
    const TYPE: InverseRelationType = InverseRelationType::ToMany;
}

impl<M: ModelMeta> InverseRelationTypeOps for Option<M> {
    type Inner = ToOneContainer<M>;
    type Model = M;
    const TYPE: InverseRelationType = InverseRelationType::ToOne;
}

impl<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize> AsPgType
    for InverseRelation<Ret, FORWARD_FIELD_INDEX>
where
    InverseRelation<Ret, FORWARD_FIELD_INDEX>: FromRowNamed,
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

impl<M: ModelMeta, const FORWARD_FIELD_INDEX: usize> FromRowNamed
    for InverseRelation<ToManyContainer<M>, FORWARD_FIELD_INDEX>
{
    fn from_row_named(_ctx: &ResponseCtx, _row: &Row, _name: &str) -> GasResult<Self> {
        // TODO:
        Ok(Self {
            loaded: false,
            items: ToManyContainer::<M>::default(),
        })
    }
}

impl<M: ModelMeta, const FORWARD_FIELD_INDEX: usize> FromRowNamed
    for InverseRelation<ToOneContainer<M>, FORWARD_FIELD_INDEX>
{
    fn from_row_named(_ctx: &ResponseCtx, _row: &Row, _name: &str) -> GasResult<Self> {
        // TODO:
        Ok(Self {
            loaded: false,
            items: ToOneContainer::<M>::default(),
        })
    }
}

impl<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize> Deref
    for InverseRelation<Ret, FORWARD_FIELD_INDEX>
{
    type Target = Ret;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<Ret: Clone + Default, const FORWARD_FIELD_INDEX: usize>
    InverseRelation<Ret, FORWARD_FIELD_INDEX>
where
    InverseRelation<Ret, FORWARD_FIELD_INDEX>: InverseRelationOps<Ret>,
{
    pub async fn load<E: PgExecutionContext>(&mut self, ctx: E) -> GasResult<&Ret> {
        if self.loaded {
            return Ok(&self.items);
        }

        self.reload(ctx).await
    }
}

pub trait InverseRelationOps<Ret> {
    fn reload<'a, E: PgExecutionContext>(
        &'a mut self,
        ctx: E,
    ) -> impl Future<Output = GasResult<&'a Ret>>
    where
        Ret: 'a;
}

impl<M: ModelMeta, const FORWARD_FIELD_INDEX: usize> InverseRelationOps<ToManyContainer<M>>
    for InverseRelation<ToManyContainer<M>, FORWARD_FIELD_INDEX>
{
    async fn reload<'a, E: PgExecutionContext>(
        &'a mut self,
        _ctx: E,
    ) -> GasResult<&'a ToManyContainer<M>>
    where
        M: 'a,
    {
        todo!()
    }
}

impl<M: ModelMeta, const FORWARD_FIELD_INDEX: usize> InverseRelationOps<ToOneContainer<M>>
    for InverseRelation<ToOneContainer<M>, FORWARD_FIELD_INDEX>
{
    async fn reload<'a, E: PgExecutionContext>(
        &'a mut self,
        _ctx: E,
    ) -> GasResult<&'a ToOneContainer<M>>
    where
        M: 'a,
    {
        todo!()
    }
}
