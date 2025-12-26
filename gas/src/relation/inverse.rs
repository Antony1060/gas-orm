use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::internals::{AsPgType, IsOptional, PgParam, PgType};
use crate::ops::select::SelectBuilder;
use crate::row::{FromRowNamed, ResponseCtx, Row};
use crate::{FieldMeta, GasResult, ModelMeta, ModelOps};
use std::marker::PhantomData;
use std::ops::Deref;
use tokio::runtime::Handle;

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InverseRelation<
    SelfModel: ModelMeta,
    Fk: AsPgType,
    Ret: Clone + Default,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> where
    PgParam: From<Fk>,
{
    parent_fk: Fk,
    loaded: bool,
    items: Ret,
    _marker: PhantomData<SelfModel>,
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

// 📦, maybe replace with Arc
type ToManyContainer<M> = Box<[M]>;
type ToOneContainer<M> = Option<Box<M>>;

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

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType,
    Ret: Clone + Default,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> AsPgType for InverseRelation<SelfModel, Fk, Ret, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>
where
    InverseRelation<SelfModel, Fk, Ret, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>: FromRowNamed,
    PgParam: From<Fk>,
{
    const PG_TYPE: PgType = PgType::IGNORED;
}

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType,
    Ret: Clone + Default,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> IsOptional for InverseRelation<SelfModel, Fk, Ret, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    const FACTOR: u8 = 0;
}

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType,
    Ret: Clone + Default,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> From<InverseRelation<SelfModel, Fk, Ret, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>> for PgParam
where
    PgParam: From<Fk>,
{
    fn from(
        _value: InverseRelation<SelfModel, Fk, Ret, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>,
    ) -> Self {
        PgParam::IGNORED
    }
}

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType + 'static,
    M: ModelMeta,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> FromRowNamed
    for InverseRelation<SelfModel, Fk, ToManyContainer<M>, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    fn from_row_named(ctx: &ResponseCtx, row: &Row, _name: &str) -> GasResult<Self> {
        Self::new_from_row_slow(ctx, row)
    }
}

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType + 'static,
    M: ModelMeta,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> FromRowNamed
    for InverseRelation<SelfModel, Fk, ToOneContainer<M>, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    fn from_row_named(ctx: &ResponseCtx, row: &Row, _name: &str) -> GasResult<Self> {
        Self::new_from_row_slow(ctx, row)
    }
}

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType,
    Ret: Clone + Default,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> Deref for InverseRelation<SelfModel, Fk, Ret, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    type Target = Ret;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType + 'static,
    Ret: Clone + Default,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> InverseRelation<SelfModel, Fk, Ret, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>
where
    InverseRelation<SelfModel, Fk, Ret, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>:
        InverseRelationOps<Ret>,
    PgParam: From<Fk>,
{
    fn new_from_row_slow(ctx: &ResponseCtx, row: &Row) -> GasResult<Self> {
        let mut instance = Self {
            parent_fk: FromRowNamed::from_row_named(ctx, row, Self::get_fk_own_field().alias_name)?,
            loaded: false,
            items: Ret::default(),
            _marker: PhantomData,
        };

        // crimes
        Handle::current().block_on(async move {
            instance.reload(&ctx.connection).await?;

            Ok(instance)
        })
    }

    pub async fn load<E: PgExecutionContext>(&mut self, ctx: E) -> GasResult<&Ret> {
        if self.loaded {
            return Ok(&self.items);
        }

        self.reload(ctx).await
    }

    fn get_fk_own_field() -> &'static FieldMeta {
        SelfModel::FIELDS
            .get(OWN_FIELD_INDEX)
            .expect("invalid relation")
    }

    pub fn update_parent(&mut self, model: &SelfModel) {
        self.parent_fk = model
            .get_by_field(Self::get_fk_own_field())
            .expect("invalid relation");
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

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType,
    M: ModelMeta,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> InverseRelationOps<ToManyContainer<M>>
    for InverseRelation<SelfModel, Fk, ToManyContainer<M>, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    async fn reload<'a, E: PgExecutionContext>(
        &'a mut self,
        ctx: E,
    ) -> GasResult<&'a ToManyContainer<M>>
    where
        M: 'a,
    {
        let select = make_lazy_inverse_query::<Fk, M, FORWARD_FIELD_INDEX>(self.parent_fk.clone())?;

        let resp = select.find_all(ctx).await?;
        self.loaded = true;
        self.items = resp.into_boxed_slice();
        Ok(&self.items)
    }
}

impl<
    SelfModel: ModelMeta,
    Fk: AsPgType,
    M: ModelMeta,
    const FORWARD_FIELD_INDEX: usize,
    const OWN_FIELD_INDEX: usize,
> InverseRelationOps<ToOneContainer<M>>
    for InverseRelation<SelfModel, Fk, ToOneContainer<M>, FORWARD_FIELD_INDEX, OWN_FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    async fn reload<'a, E: PgExecutionContext>(
        &'a mut self,
        ctx: E,
    ) -> GasResult<&'a ToOneContainer<M>>
    where
        M: 'a,
    {
        let select = make_lazy_inverse_query::<Fk, M, FORWARD_FIELD_INDEX>(self.parent_fk.clone())?;

        let resp = select.find_one(ctx).await?;
        self.loaded = true;
        self.items = resp.map(Box::from);
        Ok(&self.items)
    }
}

fn make_lazy_inverse_query<Fk, M: ModelMeta, const FIELD_INDEX: usize>(
    parent_fk: Fk,
) -> GasResult<SelectBuilder<M>>
where
    PgParam: From<Fk>,
{
    let field = M::FIELDS
        .get(FIELD_INDEX)
        .ok_or_else(|| GasError::InvalidRelation)?;

    let mut select = M::query();
    unsafe {
        select = select.raw_filter(
            format!("{}=?", field.full_name),
            &[PgParam::from(parent_fk)],
        );
    }

    Ok(select)
}
