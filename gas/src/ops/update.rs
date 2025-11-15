use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::{GasResult, ModelMeta};

const UPDATE_NO_ROW_ERR: GasError = {
    GasError::UnexpectedState(
        "no returned row on update (this could be because a primary key was modified which is not allowed)",
    )
};

pub(crate) struct UpdateOp<'a, T: ModelMeta> {
    // object will be replaced with the inserted one
    object: &'a mut T,
}

impl<'a, T: ModelMeta> UpdateOp<'a, T> {
    pub(crate) fn new(object: &'a mut T) -> Self {
        Self { object }
    }

    pub(crate) async fn run<E: PgExecutionContext>(self, ctx: &E) -> GasResult<()> {
        let (sql, params) = self.object.gen_update_sql();

        let mut rows = ctx.execute_parsed::<T>(sql, params.as_ref()).await?;
        let updated = rows.pop().ok_or(UPDATE_NO_ROW_ERR)?;

        *self.object = updated;

        Ok(())
    }
}
