use crate::connection::PgExecutionContext;
use crate::internals::{AsPgType, Numeric, SqlQuery, SqlStatement};
use crate::ops::select::SelectBuilder;
use crate::row::{FromRow, Row};
use crate::{Field, GasResult, ModelMeta};

pub struct Group<M: ModelMeta + 'static, G: AsPgType + 'static> {
    pub(crate) field: Field<G, M>,
    pub(crate) select: SelectBuilder<M>,
}

impl<M: ModelMeta, G: AsPgType + 'static> Group<M, G> {
    pub fn new(field: Field<G, M>, select: SelectBuilder<M>) -> Self {
        Self { field, select }
    }

    pub async fn sum<E: PgExecutionContext, N: Numeric>(
        self,
        ctx: E,
        field: Field<N, M>,
    ) -> GasResult<Vec<Summed<G, N>>> {
        let aggregate_call = format!("SUM({})", field.full_name);
        let (sql, params) = self.build_aggregate_query(&aggregate_call);

        ctx.execute_parsed(sql, &params).await
    }

    pub async fn count<E: PgExecutionContext, T>(
        self,
        ctx: E,
        field: Field<T, M>,
    ) -> GasResult<Vec<Counted<G>>> {
        let aggregate_call = format!("COUNT({})", field.full_name);
        let (sql, params) = self.build_aggregate_query(&aggregate_call);

        ctx.execute_parsed(sql, &params).await
    }

    pub fn build_aggregate_query(self, aggregate_call: &str) -> SqlStatement<'_> {
        // sql
        let mut sql = SqlQuery::from(format!(
            "SELECT {} as key, {} as aggregate FROM {}",
            self.field.full_name,
            aggregate_call,
            M::TABLE_NAME
        ));

        if let Some(ref filter) = self.select.filter {
            sql.append_str(" WHERE ");
            sql.append_query(filter.condition.as_sql());
        }

        sql.append_str(&format!(" GROUP BY {}", self.field.full_name));

        // params
        let params = self
            .select
            .filter
            .map(|it| it.params.into_boxed_slice())
            .unwrap_or_else(|| Box::new([]));

        (sql, params)
    }
}

#[derive(Debug)]
pub struct Counted<G: AsPgType> {
    pub key: G,
    pub count: i64,
}

impl<G: AsPgType> FromRow for Counted<G> {
    fn from_row(row: &Row) -> GasResult<Self> {
        Ok(Self {
            key: row.try_get("key")?,
            count: row.try_get("aggregate")?,
        })
    }
}

#[derive(Debug)]
pub struct Summed<G: AsPgType, N: Numeric> {
    pub key: G,
    pub sum: N,
}

impl<G: AsPgType, N: Numeric> FromRow for Summed<G, N> {
    fn from_row(row: &Row) -> GasResult<Self> {
        Ok(Self {
            key: row.try_get("key")?,
            sum: row.try_get("aggregate")?,
        })
    }
}
