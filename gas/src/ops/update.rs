use crate::connection::PgExecutor;
use crate::error::GasError;
use crate::model::ModelMeta;
use crate::{FieldFlag, GasResult};

const UPDATE_NO_MODIFIED_FIELDS_ERR: GasError =
    GasError::InvalidInput("attempted to update an object with no modified fields");

const UPDATE_NO_ROW_ERR: GasError = {
    GasError::QueryNoResponse(
        "no returned row on update (this could be because of a non-existing primary key)",
    )
};

// updates can fail with GasError::QueryNoResponse, it took way too much brain power to think
//  if it would be an Option<T> or return with an error so we're left with this
pub(crate) struct UpdateOp<'a, T: ModelMeta> {
    // object will be replaced with the updated one
    object: &'a mut T,
}

impl<'a, T: ModelMeta> UpdateOp<'a, T> {
    pub(crate) fn new(object: &'a mut T) -> Self {
        Self { object }
    }

    pub(crate) async fn run<E: PgExecutor>(self, ctx: E) -> GasResult<()> {
        let (sql, params) = self.object.gen_update_sql();

        let mut rows = ctx.execute_parsed::<T>(sql, &params).await?;
        let updated = rows.pop().ok_or(UPDATE_NO_ROW_ERR)?;

        *self.object = updated;

        Ok(())
    }

    pub(crate) async fn run_with_fields<E: PgExecutor>(
        self,
        ctx: E,
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
            .filter(|field| !field.flags.has_flag(FieldFlag::PrimaryKey))
            .collect::<Vec<_>>();

        if fields.is_empty() {
            return Err(UPDATE_NO_MODIFIED_FIELDS_ERR);
        }

        let (sql, params) = self.object.gen_update_with_fields_sql(&fields);

        let mut rows = ctx.execute_parsed::<T>(sql, &params).await?;
        let updated = rows.pop().ok_or(UPDATE_NO_ROW_ERR)?;

        *self.object = updated;

        Ok(())
    }
}
