use crate::connection::PgExecutionContext;
use crate::internals::{AsPgType, PgParam};
use crate::{GasResult, ModelMeta, ModelOps};

pub enum Relation<Fk: AsPgType, Model: ModelMeta> {
    ForeignKey(Fk),
    Loaded(Model),
}

// TODO: enforce that Field<Fk, Model> matches the one provided with macro, e.g.
//  ```
//  #[foreign(key = account::id)] // account::id must be a Field<i64, account::Model>
//  // macro should replace this type and make it RelationFull<i64, account::Model, { account::id.index }> while checking if Field<i64, account::Model>
//  some_fk: Relation<i64, account::Model>
//  ```
// NOTE: a foreign key must have uniqueness, so it must have a unique constraint or
//  be a primary key unless it's part of a composite primary key (i.e. there's only one)
pub enum FullRelation<Fk: AsPgType, Model: ModelMeta, const FIELD_INDEX: usize> {
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
            FullRelation::Loaded(model) => Ok(Some(model)),
            FullRelation::ForeignKey(key) => {
                let Some(model) = Self::load_by_key(ctx, key.clone()).await? else {
                    return Ok(None);
                };

                *self = FullRelation::Loaded(model);
                let FullRelation::Loaded(model) = self else {
                    unreachable!()
                };

                Ok(Some(model))
            }
        }
    }
}
