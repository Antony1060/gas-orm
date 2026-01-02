use crate::connection::PgExecutor;
use crate::error::GasError;
use crate::model::ModelMeta;
use crate::GasResult;

pub(crate) struct InsertOp<'a, T: ModelMeta> {
    // object will be replaced with the inserted one
    object: &'a mut T,
}

impl<'a, T: ModelMeta> InsertOp<'a, T> {
    pub(crate) fn new(object: &'a mut T) -> Self {
        Self { object }
    }

    pub(crate) async fn run<E: PgExecutor>(self, ctx: E) -> GasResult<()> {
        let (sql, params) = self.object.gen_insert_sql();

        let mut rows = ctx.execute_parsed::<T>(sql, &params).await?;
        let inserted = rows
            .pop()
            .ok_or_else(|| GasError::UnexpectedResponse("no returned row on insert".into()))?;

        *self.object = inserted;

        Ok(())
    }
}
