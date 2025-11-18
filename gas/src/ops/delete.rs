use crate::connection::PgExecutionContext;
use crate::model::ModelMeta;
use crate::GasResult;

pub(crate) struct DeleteOp<T: ModelMeta> {
    object: T,
}

impl<T: ModelMeta> DeleteOp<T> {
    pub(crate) fn new(object: T) -> Self {
        Self { object }
    }

    pub(crate) async fn run<E: PgExecutionContext>(self, ctx: E) -> GasResult<()> {
        let (sql, params) = self.object.gen_delete_sql();

        ctx.execute(sql, &params).await?;

        Ok(())
    }
}
