use crate::connection::PgExecutionContext;
use crate::field::FieldMeta;
use crate::internals::SqlStatement;
use crate::ops::create::CreateOp;
use crate::ops::delete::DeleteOp;
use crate::ops::insert::InsertOp;
use crate::ops::select::SelectBuilder;
use crate::ops::update::UpdateOp;
use crate::row::FromRow;
use crate::GasResult;

pub trait ModelMeta: Sized + FromRow {
    const TABLE_NAME: &'static str;
    const FIELDS: &'static [FieldMeta];

    type Key;

    fn gen_insert_sql(&self) -> SqlStatement<'_>;

    fn gen_update_sql(&self) -> SqlStatement<'_>;

    fn gen_delete_sql(&self) -> SqlStatement<'_>;
}

// NOTE: maybe add ByKeyOps<T: ModelMeta, Key> that will implement find_by_key, delete_by_key and update_by_key
//  update_by_key would probably be used something like
//  ```
//  user::Model {
//      username: "user1234".to_string(),
//      ..user::default()
//  }.update_by_key(key: K) // insert would be similar
//  ```
//
// NOTE 2: maybe add aliases for all of these in the root of the namespace,
//  so it can be used like user::query() or user::create_table()
pub trait ModelOps: ModelMeta {
    fn query() -> SelectBuilder<Self> {
        SelectBuilder::new()
    }

    // some trait bounds cannot be enforced if I just do `async fn` here, idk
    fn create_table<E: PgExecutionContext>(
        ctx: &E,
        ignore_existing: bool,
    ) -> impl Future<Output = GasResult<()>> {
        CreateOp::<Self>::new(ignore_existing).run(ctx)
    }

    // consume self and return an entry that is inserted
    fn insert<E: PgExecutionContext>(&mut self, ctx: &E) -> impl Future<Output = GasResult<()>> {
        InsertOp::<Self>::new(self).run(ctx)
    }

    fn update<E: PgExecutionContext>(&mut self, ctx: &E) -> impl Future<Output = GasResult<()>> {
        UpdateOp::<Self>::new(self).run(ctx)
    }

    fn delete<E: PgExecutionContext>(self, ctx: &E) -> impl Future<Output = GasResult<()>> {
        DeleteOp::<Self>::new(self).run(ctx)
    }
}

impl<T: ModelMeta> ModelOps for T {}
