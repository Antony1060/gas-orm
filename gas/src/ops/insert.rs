use crate::connection::PgExecutor;
use crate::model::ModelMeta;
use crate::GasResult;

pub(crate) struct InsertOp<'a, T: ModelMeta> {
    // object will be replaced with the inserted one
    objects: &'a mut [T],
}

impl<'a, T: ModelMeta> InsertOp<'a, T> {
    pub(crate) fn new(object: &'a mut [T]) -> Self {
        Self { objects: object }
    }

    pub(crate) async fn run<E: PgExecutor>(self, ctx: E) -> GasResult<()> {
        let (insert, returning) = T::gen_insert_parts_sql();

        let mut full_query = insert;
        let mut full_params = Vec::new();

        for (index, object) in self.objects.iter().enumerate() {
            let (sql, params) = object.gen_insert_values_sql();
            full_query.append_query(sql);

            if index < self.objects.len() - 1 {
                full_query.append_str(",");
            }

            full_params.extend(params);
        }

        full_query.append_query(returning);

        let rows = ctx.execute_parsed::<T>(full_query, &full_params).await?;

        // NOTE: ordering should be same in practice, this might break sometime in the future
        for (object, row) in self.objects.iter_mut().zip(rows) {
            *object = row;
        }

        Ok(())
    }
}
