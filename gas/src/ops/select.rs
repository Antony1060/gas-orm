#![allow(private_bounds)]

use crate::condition::EqExpression;
use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::group::Group;
use crate::internals::{AsPgType, Numeric, SqlQuery, SqlStatement};
use crate::model::ModelMeta;
use crate::sort::SortDefinition;
use crate::{Field, GasResult};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Default)]
pub struct SelectBuilder<T: ModelMeta> {
    pub(crate) filter: Option<EqExpression>,
    pub(crate) sort: Option<SortDefinition>,
    pub(crate) limit: Option<NonZeroUsize>,
    _marker: PhantomData<T>,
}

impl<M: ModelMeta> SelectBuilder<M> {
    pub fn new() -> Self {
        Self {
            filter: None,
            sort: None,
            limit: None,
            _marker: PhantomData,
        }
    }

    pub fn filter<F: FnOnce() -> EqExpression>(mut self, cond_fn: F) -> Self {
        self.filter = Some(cond_fn());
        self
    }

    pub fn sort(mut self, sort_definition: SortDefinition) -> Self {
        self.sort = Some(sort_definition);
        self
    }

    pub fn limit(mut self, items: usize) -> Self {
        self.limit = NonZeroUsize::new(items);
        self
    }

    pub fn group<Ty: AsPgType>(self, field: Field<Ty, M>) -> Group<M, Ty> {
        Group {
            field,
            select: self,
        }
    }

    pub async fn find_all<E: PgExecutionContext>(self, ctx: E) -> GasResult<Vec<M>> {
        let (sql, params) = self.build(true);

        let items = ctx.execute_parsed::<M>(sql, &params).await?;

        Ok(items)
    }

    pub async fn find_one<E: PgExecutionContext>(self, ctx: E) -> GasResult<Option<M>> {
        let (mut sql, params) = self.build(false);

        sql.append_str(" LIMIT 1");

        let mut items = ctx.execute_parsed::<M>(sql, &params).await?;

        if items.len() > 1 {
            return Err(GasError::UnexpectedResponse(
                format!("find_one: got {}, expected <= 1", items.len()).into(),
            ));
        }

        Ok(items.pop())
    }

    pub async fn sum<E: PgExecutionContext, N: Numeric>(
        self,
        ctx: E,
        field: Field<N, M>,
    ) -> GasResult<N> {
        let aggregate_call = format!("SUM({})", field.full_name);
        let (sql, params) = self.build_aggregate_query(&aggregate_call);

        let rows = ctx.execute(sql, &params).await?;
        if rows.len() != 1 {
            return Err(GasError::UnexpectedResponse(
                format!("find_one: got {}, expected <= 1", rows.len()).into(),
            ));
        }

        rows[0].try_get("aggregate")
    }

    pub async fn count<E: PgExecutionContext, T>(
        self,
        ctx: E,
        field: Field<T, M>,
    ) -> GasResult<i64> {
        let field = field.as_ref();
        let aggregate_call = format!("COUNT({})", field.full_name);
        let (sql, params) = self.build_aggregate_query(&aggregate_call);

        let rows = ctx.execute(sql, &params).await?;
        if rows.len() != 1 {
            return Err(GasError::UnexpectedResponse(
                format!("find_one: got {}, expected <= 1", rows.len()).into(),
            ));
        }

        rows[0].try_get("aggregate")
    }

    // include_limit is important here because of find_one
    //  if limit is built into the query and then later on enforced by find_one,
    //  the query would fail; not very nice way to enforce an invariant but eh
    fn build<'a>(self, include_limit: bool) -> SqlStatement<'a> {
        // sql
        let fields = M::FIELDS
            .iter()
            .map(|f| format!("{} AS {}", f.full_name, f.alias_name))
            .reduce(|acc, cur| format!("{}, {}", acc, cur))
            .expect("no fields");

        let mut sql = SqlQuery::from(format!("SELECT {} FROM {}", fields, M::TABLE_NAME));

        if let Some(ref filter) = self.filter {
            sql.append_str(" WHERE ");
            sql.append_query(filter.condition.as_sql());
        }

        if let Some(ref sort) = self.sort
            && let Some(sort_sql) = sort.as_sql()
        {
            sql.append_str(" ORDER BY ");
            sql.append_query(sort_sql);
        }

        if include_limit && let Some(limit) = self.limit {
            sql.append_str(&format!(" LIMIT {}", limit.get()));
        }

        // params
        let params = self
            .filter
            .map(|it| it.params.into_boxed_slice())
            .unwrap_or_else(|| Box::new([]));

        (sql, params)
    }

    pub fn build_aggregate_query(self, aggregate_call: &str) -> SqlStatement<'_> {
        // sql
        let mut sql = SqlQuery::from(format!(
            "SELECT {} as aggregate FROM {}",
            aggregate_call,
            M::TABLE_NAME
        ));

        if let Some(ref filter) = self.filter {
            sql.append_str(" WHERE ");
            sql.append_query(filter.condition.as_sql());
        }

        // params
        let params = self
            .filter
            .map(|it| it.params.into_boxed_slice())
            .unwrap_or_else(|| Box::new([]));

        (sql, params)
    }
}
