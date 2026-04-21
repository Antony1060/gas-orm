use crate::connection::PgExecutor;
use crate::internals::{PgParam, SqlQuery};
use crate::model::ModelMeta;
use crate::GasResult;
use tokio::task::JoinSet;

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

        let mut values_chunks: Vec<(SqlQuery, Vec<PgParam>, usize)> =
            vec![(SqlQuery::new(), vec![], 0)];

        for (index, object) in self.objects.iter().enumerate() {
            let (last_sql, last_params, last_count) =
                values_chunks.last_mut().expect("chunks are not empty");

            let (sql, params) = object.gen_insert_values_sql();

            if last_params.len() + params.len() > MAX_POSITIONAL_ARGS_LIMIT {
                values_chunks.push((sql, params.to_vec(), 1));
                continue;
            }

            if index != 0 {
                last_sql.append_str(",");
            }

            *last_count += 1;
            last_sql.append_query(&sql);
            last_params.extend(params);
        }

        let mut join_set = JoinSet::new();

        let mut index = 0;
        for (sql, params, count) in values_chunks {
            let mut full_query = insert.clone();
            full_query.append_query(&sql);
            full_query.append_query(&returning);

            join_set.spawn(async move {
                let rows = ctx.execute_parsed::<T>(full_query, &params).await?;

                // NOTE: ordering should be same in practice, this might break sometime in the future
                for (object, row) in self.objects[index..index + rows.len()].iter_mut().zip(rows) {
                    *object = row;
                }
            });

            index += count;
        }

        join_set.join_all().await;

        Ok(())
    }
}
