use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::{GasResult, ModelMeta};

pub(crate) struct InsertOp<'a, T: ModelMeta> {
    // object will me replaced with the inserted one
    object: &'a mut T,
}

impl<'a, T: ModelMeta> InsertOp<'a, T> {
    pub(crate) fn new(object: &'a mut T) -> Self {
        Self { object }
    }

    pub(crate) async fn run<E: PgExecutionContext>(self, ctx: &E) -> GasResult<()> {
        let (sql, params) = self.object.gen_insert_sql();

        let mut rows = ctx.execute_parsed::<T>(sql, params.as_ref()).await?;
        let inserted = rows
            .pop()
            .ok_or_else(|| GasError::UnexpectedResponse("no returned row on insert".to_string()))?;

        *self.object = inserted;

        Ok(())
    }
}
