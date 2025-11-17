use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::model::ModelMeta;
use crate::GasResult;

const UPDATE_NO_ROW_ERR: GasError = {
    GasError::UnexpectedState(
        "no returned row on update (this could be because a primary key was modified which is not allowed)",
    )
};

pub(crate) struct UpdateOp<'a, T: ModelMeta> {
    // object will be replaced with the updated one
    object: &'a mut T,
}

impl<'a, T: ModelMeta> UpdateOp<'a, T> {
    pub(crate) fn new(object: &'a mut T) -> Self {
        Self { object }
    }

    pub(crate) async fn run<E: PgExecutionContext>(self, ctx: &E) -> GasResult<()> {
        let (sql, params) = self.object.gen_update_sql();

        let mut rows = ctx.execute_parsed::<T>(sql, &params).await?;
        let updated = rows.pop().ok_or(UPDATE_NO_ROW_ERR)?;

        *self.object = updated;

        Ok(())
    }

    pub(crate) async fn run_with_fields<E: PgExecutionContext>(
        self,
        ctx: &E,
        fields: &[&'static str],
    ) -> GasResult<()> {
        // mmmm O(n^2)
        let fields = fields
            .iter()
            .map(|field| {
                T::FIELDS
                    .iter()
                    .find(|it| it.struct_name == *field)
                    .expect("field mismatch")
            })
            .collect::<Vec<_>>();
        let (sql, params) = self.object.gen_update_with_fields_sql(&fields);

        let mut rows = ctx.execute_parsed::<T>(sql, &params).await?;
        let updated = rows.pop().ok_or(UPDATE_NO_ROW_ERR)?;

        *self.object = updated;

        Ok(())
    }
}
