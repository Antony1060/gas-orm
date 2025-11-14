use crate::connection::PgExecutionContext;
use crate::{GasResult, ModelMeta};

pub(crate) struct DeleteOp<T: ModelMeta> {
    // object will me replaced with the inserted one
    object: T,
}

impl<T: ModelMeta> DeleteOp<T> {
    pub(crate) fn new(object: T) -> Self {
        Self { object }
    }

    pub(crate) async fn run<E: PgExecutionContext>(self, ctx: &E) -> GasResult<()> {
        let (sql, params) = self.object.gen_delete_sql();

        ctx.execute(sql, params.as_ref()).await?;

        Ok(())
    }
}
