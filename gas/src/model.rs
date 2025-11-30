use crate::condition::EqExpression;
use crate::connection::PgExecutionContext;
use crate::field::FieldMeta;
use crate::internals::{AsPgType, SqlStatement};
use crate::ops::create::CreateOp;
use crate::ops::delete::DeleteOp;
use crate::ops::insert::InsertOp;
use crate::ops::select::SelectBuilder;
use crate::ops::update::UpdateOp;
use crate::row::FromRow;
use crate::GasResult;

pub trait ModelSidecar {}

pub trait ModelMeta: Sized + Default + Clone + FromRow {
    type Id: ModelSidecar;

    const TABLE_NAME: &'static str;
    const FIELDS: &'static [&'static FieldMeta];

    type Key;

    fn apply_key(&mut self, key: Self::Key);

    fn filter_with_key(key: Self::Key) -> EqExpression;

    fn gen_insert_sql(&self) -> SqlStatement<'_>;

    fn gen_update_sql(&self) -> SqlStatement<'_>;

    fn gen_update_with_fields_sql(&self, fields: &[&FieldMeta]) -> SqlStatement<'_>;

    fn gen_delete_sql(&self) -> SqlStatement<'_>;

    // will be implemented by a macro with some unsafe magic
    //  used with relations
    fn get_by_field<T: AsPgType + 'static>(&self, field: &FieldMeta) -> Option<T>;
}

pub trait ModelOps: ModelMeta {
    fn query() -> SelectBuilder<Self> {
        SelectBuilder::new()
    }

    // some trait bounds cannot be enforced if I just do `async fn` here
    fn create_table<E: PgExecutionContext>(
        ctx: E,
        ignore_existing: bool,
    ) -> impl Future<Output = GasResult<()>> {
        CreateOp::<Self>::new(ignore_existing).run(ctx)
    }

    // consume self and return an entry that is inserted
    fn insert<E: PgExecutionContext>(&mut self, ctx: E) -> impl Future<Output = GasResult<()>> {
        InsertOp::<Self>::new(self).run(ctx)
    }

    fn update<E: PgExecutionContext>(&mut self, ctx: E) -> impl Future<Output = GasResult<()>> {
        UpdateOp::<Self>::new(self).run(ctx)
    }

    fn delete<E: PgExecutionContext>(self, ctx: E) -> impl Future<Output = GasResult<()>> {
        DeleteOp::<Self>::new(self).run(ctx)
    }

    fn find_by_key<E: PgExecutionContext>(
        ctx: E,
        key: Self::Key,
    ) -> impl Future<Output = GasResult<Option<Self>>> {
        Self::query()
            .filter(|| Self::filter_with_key(key))
            .find_one(ctx)
    }

    fn delete_by_key<E: PgExecutionContext>(
        ctx: E,
        key: Self::Key,
    ) -> impl Future<Output = GasResult<()>> {
        let mut im = Self::default();
        im.apply_key(key);
        DeleteOp::<Self>::new(im).run(ctx)
    }
}

impl<T: ModelMeta> ModelOps for T {}
