use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::internals::PgType::FOREIGN_KEY;
use crate::internals::{AsPgType, IsOptional, NaiveDecodable, PgParam, PgType};
use crate::row::{FromRowNamed, ResponseCtx, Row};
use crate::{Field, GasResult, ModelMeta, ModelOps};
use std::marker::PhantomData;

pub struct Relation<Fk: AsPgType + 'static, Model: ModelMeta> {
    _fk_marker: PhantomData<Fk>,
    _model_marker: PhantomData<Model>,
}

pub trait RelationTypeOps {
    type ToFull<const FIELD_INDEX: usize>;
    type ToField;
    type ToNaive;
}

impl<Fk: AsPgType + 'static, Model: ModelMeta> RelationTypeOps for Relation<Fk, Model> {
    type ToFull<const FIELD_INDEX: usize> = FullRelation<Fk, Model, FIELD_INDEX>;
    type ToField = Field<Fk, Model::Id>;
    type ToNaive = Fk;
}

impl<Fk: AsPgType + 'static, Model: ModelMeta> RelationTypeOps for Option<Relation<Fk, Model>> {
    type ToFull<const FIELD_INDEX: usize> = Option<FullRelation<Fk, Model, FIELD_INDEX>>;
    type ToField = Field<Fk, Model::Id>;
    type ToNaive = Fk;
}

// NOTE: a foreign key must have uniqueness, so it must have a unique constraint or
//  be a primary key unless it's part of a composite primary key (i.e. there's only one)
#[derive(Debug, Clone)]
pub enum FullRelation<Fk: AsPgType + 'static, Model: ModelMeta, const FIELD_INDEX: usize> {
    // this is cursed
    ForeignKey(Fk),
    Loaded(Model),
}

// the idea is that the macro will generate the signature of this type and correctly put in the index
//  this shit is cursed
impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> FullRelation<Fk, Model, FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    async fn load_by_key<E: PgExecutionContext>(ctx: E, key: Fk) -> GasResult<Option<Model>> {
        let field = Model::FIELDS
            .get(FIELD_INDEX)
            .ok_or_else(|| GasError::InvalidRelation)?;

        let mut select = Model::query();
        unsafe {
            select = select.raw_filter(format!("{}=?", field.full_name), &[PgParam::from(key)]);
        }

        select.find_one(ctx).await
    }

    // NOTE: can panic
    pub fn get_foreign_key(&self) -> Fk {
        match self {
            FullRelation::Loaded(model) => model
                .get_by_field(
                    Model::FIELDS
                        .get(FIELD_INDEX)
                        .expect("field relation is not correctly defined"),
                )
                .expect("foreign key should be accessible by field"),
            FullRelation::ForeignKey(key) => key.clone(),
        }
    }
}

pub trait RelationOps<M: ModelMeta> {
    // will try to lazy load
    fn load<'a, E: PgExecutionContext>(
        &'a mut self,
        ctx: E,
    ) -> impl Future<Output = GasResult<Option<&'a M>>>
    where
        M: 'a;

    // explicit method to get the eagerly loaded variant
    fn model(&mut self) -> Option<&M>;
}

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> RelationOps<Model>
    for FullRelation<Fk, Model, FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    async fn load<'a, E: PgExecutionContext>(&'a mut self, ctx: E) -> GasResult<Option<&'a Model>>
    where
        Model: 'a,
    {
        match self {
            FullRelation::Loaded(model) => Ok(Some(model)),
            FullRelation::ForeignKey(key) => {
                let Some(model) = Self::load_by_key(ctx, key.clone()).await? else {
                    return Ok(None);
                };

                *self = FullRelation::Loaded(model);
                let FullRelation::Loaded(model) = self else {
                    unreachable!("relation must be loaded after being assigned a loaded value")
                };

                Ok(Some(model))
            }
        }
    }

    fn model(&mut self) -> Option<&Model> {
        match self {
            FullRelation::Loaded(model) => Some(model),
            _ => None,
        }
    }
}

// allow load() and model() on Option<FullRelation<...>> to improve ergonomics
impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> RelationOps<Model>
    for Option<FullRelation<Fk, Model, FIELD_INDEX>>
where
    PgParam: From<Fk>,
{
    async fn load<'a, E: PgExecutionContext>(&'a mut self, ctx: E) -> GasResult<Option<&'a Model>>
    where
        Model: 'a,
    {
        match self {
            Some(relation) => relation.load(ctx).await,
            None => Ok(None),
        }
    }

    fn model(&mut self) -> Option<&Model> {
        match self {
            Some(relation) => relation.model(),
            None => None,
        }
    }
}

// things required for the FullRelation type compatible with gas::model macro

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> Default
    for FullRelation<Fk, Model, FIELD_INDEX>
{
    fn default() -> Self {
        Self::ForeignKey(<Fk as Default>::default())
    }
}

impl<Fk: AsPgType + NaiveDecodable, Model: ModelMeta, const FIELD_INDEX: usize> AsPgType
    for FullRelation<Fk, Model, FIELD_INDEX>
{
    // NOTE: resolved in compile time, array access should fail on time
    //  this also has a nice side effect of failing before other places that
    //  do the same array access even get to run
    const PG_TYPE: PgType = FOREIGN_KEY {
        key_type: &Fk::PG_TYPE,
        target_field: Model::FIELDS[FIELD_INDEX],
    };
}

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> IsOptional
    for FullRelation<Fk, Model, FIELD_INDEX>
{
    const FACTOR: u8 = 0;
}

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize>
    From<FullRelation<Fk, Model, FIELD_INDEX>> for PgParam
where
    PgParam: From<Fk>,
{
    fn from(value: FullRelation<Fk, Model, FIELD_INDEX>) -> Self {
        PgParam::from(value.get_foreign_key())
    }
}

impl<Fk: AsPgType + NaiveDecodable, Model: ModelMeta, const FIELD_INDEX: usize> FromRowNamed
    for FullRelation<Fk, Model, FIELD_INDEX>
{
    fn from_row_named(ctx: &ResponseCtx, row: &Row, name: &str) -> GasResult<Self> {
        Model::from_row(ctx, row)
            .map(|model| FullRelation::Loaded(model))
            .or_else(|_| {
                Ok(FullRelation::ForeignKey(Fk::from_row_named(
                    ctx, row, name,
                )?))
            })
    }
}

// optional
impl<Fk: AsPgType + NaiveDecodable, Model: ModelMeta, const FIELD_INDEX: usize> AsPgType
    for Option<FullRelation<Fk, Model, FIELD_INDEX>>
where
    Option<Fk>: AsPgType,
{
    // NOTE: const time, array access should fail on time
    const PG_TYPE: PgType = FOREIGN_KEY {
        key_type: &Fk::PG_TYPE,
        target_field: Model::FIELDS[FIELD_INDEX],
    };
}

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize>
    From<Option<FullRelation<Fk, Model, FIELD_INDEX>>> for PgParam
where
    PgParam: From<FullRelation<Fk, Model, FIELD_INDEX>>,
{
    fn from(value: Option<FullRelation<Fk, Model, FIELD_INDEX>>) -> Self {
        match value {
            Some(value) => PgParam::from(value),
            // TODO:
            None => unreachable!(), //PgParam::NULL(PgType::TEXT),
        }
    }
}

impl<Fk: AsPgType + NaiveDecodable, Model: ModelMeta, const FIELD_INDEX: usize> FromRowNamed
    for Option<FullRelation<Fk, Model, FIELD_INDEX>>
where
    Option<Fk>: AsPgType,
{
    fn from_row_named(ctx: &ResponseCtx, row: &Row, name: &str) -> GasResult<Self> {
        Ok(Option::<Fk>::from_row_named(ctx, row, name)?.map(|fk| {
            Model::from_row(ctx, row)
                .map(|model| FullRelation::Loaded(model))
                .unwrap_or_else(|_| FullRelation::ForeignKey(fk))
        }))
    }
}
