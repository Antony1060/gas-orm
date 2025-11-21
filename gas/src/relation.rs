use crate::connection::PgExecutionContext;
use crate::internals::PgType::FOREIGN_KEY;
use crate::internals::{AsPgType, IsOptional, PgParam, PgType};
use crate::row::{FromRowNamed, Row};
use crate::{GasResult, ModelMeta, ModelOps, NaiveDecodable};

// TODO: enforce that Field<Fk, Model> matches the one provided with macro, e.g.
//  ```
//  #[foreign(key = account::id)] // account::id must be a Field<i64, account::Model>
//  // macro should replace this type and make it RelationFull<i64, account::Model, { account::id.index }> while checking if Field<i64, account::Model>
//  some_fk: Relation<i64, account::Model>
//  ```
// NOTE: a foreign key must have uniqueness, so it must have a unique constraint or
//  be a primary key unless it's part of a composite primary key (i.e. there's only one)
#[derive(Debug, Clone)]
pub enum Relation<Fk: AsPgType + 'static, Model: ModelMeta, const FIELD_INDEX: usize> {
    // this is cursed
    ForeignKey(Fk),
    Loaded(Model),
}

// the idea is that the macro will generate the signature of this type and correctly put in the index
//  this shit is cursed
impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> Relation<Fk, Model, FIELD_INDEX>
where
    PgParam: From<Fk>,
{
    async fn load_by_key<E: PgExecutionContext>(ctx: E, key: Fk) -> GasResult<Option<Model>> {
        // TODO: !!!!
        // 🫠
        let field = Model::FIELDS[FIELD_INDEX];

        let mut select = Model::query();
        unsafe {
            select = select.raw_filter(format!("{}=?", field.full_name), &[PgParam::from(key)]);
        }

        select.find_one(ctx).await
    }

    pub async fn load<E: PgExecutionContext>(&mut self, ctx: E) -> GasResult<Option<&Model>> {
        match self {
            Relation::Loaded(model) => Ok(Some(model)),
            Relation::ForeignKey(key) => {
                let Some(model) = Self::load_by_key(ctx, key.clone()).await? else {
                    return Ok(None);
                };

                *self = Relation::Loaded(model);
                let Relation::Loaded(model) = self else {
                    unreachable!()
                };

                Ok(Some(model))
            }
        }
    }

    pub fn get_foreign_key(&self) -> Fk {
        match self {
            Relation::Loaded(model) => model
                .get_by_field(Model::FIELDS[FIELD_INDEX])
                .expect("foreign key should be accessible by field"),
            Relation::ForeignKey(key) => key.clone(),
        }
    }
}

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> Default
    for Relation<Fk, Model, FIELD_INDEX>
{
    fn default() -> Self {
        Self::ForeignKey(<Fk as Default>::default())
    }
}

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> AsPgType
    for Relation<Fk, Model, FIELD_INDEX>
{
    const PG_TYPE: PgType = FOREIGN_KEY {
        key_type: &Fk::PG_TYPE,
        target_field: Model::FIELDS[FIELD_INDEX],
    };
}

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> IsOptional
    for Relation<Fk, Model, FIELD_INDEX>
{
    const FACTOR: u8 = 0;
}

impl<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize>
    From<Relation<Fk, Model, FIELD_INDEX>> for PgParam
where
    PgParam: From<Fk>,
{
    fn from(value: Relation<Fk, Model, FIELD_INDEX>) -> Self {
        PgParam::from(value.get_foreign_key())
    }
}

impl<Fk: AsPgType + NaiveDecodable, Model: ModelMeta, const FIELD_INDEX: usize> FromRowNamed
    for Relation<Fk, Model, FIELD_INDEX>
{
    fn from_row_named(row: &Row, name: &str) -> GasResult<Self> {
        Model::from_row(row)
            .map(|model| Relation::Loaded(model))
            .or_else(|_| Ok(Relation::ForeignKey(Fk::from_row_named(row, name)?)))
    }
}
