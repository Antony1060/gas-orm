use crate::connection::PgExecutor;
use crate::internals::{PgParam, SqlQuery};
use crate::model::ModelMeta;
use crate::GasResult;
use tracing::info;

pub(crate) struct InsertOp<'a, T: ModelMeta> {
    // object will be replaced with the inserted one
    objects: &'a mut [T],
}

// i16::MAX - a little bit
const MAX_POSITIONAL_ARGS_LIMIT: usize = 30000;

impl<'a, T: ModelMeta> InsertOp<'a, T> {
    pub(crate) fn new(object: &'a mut [T]) -> Self {
        Self { objects: object }
    }

    pub(crate) async fn run<E: PgExecutor>(self, ctx: E) -> GasResult<()> {
        if self.objects.is_empty() {
            return Ok(());
        }

        let (insert, returning) = T::gen_insert_parts_sql();

        let mut values_chunks: Vec<(SqlQuery, Vec<PgParam>)> = vec![(SqlQuery::new(), vec![])];

        for object in self.objects.iter() {
            let (last_sql, last_params) = values_chunks.last_mut().expect("chunks are not empty");

            let (sql, params) = object.gen_insert_values_sql();

            if last_params.len() + params.len() > MAX_POSITIONAL_ARGS_LIMIT {
                values_chunks.push((sql, params.to_vec()));
                continue;
            }

            last_sql.append_str(",");
            last_sql.append_query(&sql);
            last_params.extend(params);
        }

        info!(chunks = values_chunks.len(), "prepared insert chunks");

        let mut index = 0;
        for (sql, params) in values_chunks {
            let mut full_query = insert.clone();
            full_query.append_query(&sql);
            full_query.append_query(&returning);

            let rows = ctx.execute_parsed::<T>(full_query, &params).await?;
            let rows_len = rows.len();

            // NOTE: ordering should be same in practice, this might break sometime in the future
            for (object, row) in self.objects[index..rows.len()].iter_mut().zip(rows) {
                *object = row;
            }

            index += rows_len;
        }

        Ok(())
    }
}
