#![allow(private_bounds)]

use crate::condition::{Condition, EqExpression};
use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::group::Group;
use crate::internals::{AsPgType, Numeric, PgParam, SqlQuery, SqlStatement};
use crate::model::ModelMeta;
use crate::sort::SortDefinition;
use crate::{Field, FieldMeta, GasResult, NaiveDecodable};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Default)]
pub struct SelectBuilder<T: ModelMeta> {
    pub(crate) filter: Option<EqExpression>,
    sort: Option<SortDefinition>,
    limit: Option<NonZeroUsize>,
    includes: Vec<(String, &'static [&'static FieldMeta])>,
    _marker: PhantomData<T>,
}

impl<M: ModelMeta> SelectBuilder<M> {
    pub fn new() -> Self {
        Self {
            filter: None,
            sort: None,
            limit: None,
            includes: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn filter<F: FnOnce() -> EqExpression>(mut self, cond_fn: F) -> Self {
        self.filter = Some(cond_fn());
        self
    }

    // bad, very bad
    pub(crate) unsafe fn raw_filter(mut self, where_statement: String, params: &[PgParam]) -> Self {
        // very good, very nice, much ORM
        self.filter = Some(EqExpression::new(
            Condition::Basic(where_statement),
            params.to_vec(),
        ));
        self
    }

    pub fn raw_include(mut self, join: &str, fields: &'static [&FieldMeta]) -> Self {
        self.includes.push((join.to_string(), fields));
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

    pub fn group<Ty: AsPgType + NaiveDecodable>(self, field: Field<Ty, M>) -> Group<M, Ty> {
        Group::new(field, self)
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

    pub async fn sum<E: PgExecutionContext, FM: ModelMeta, N: Numeric>(
        self,
        ctx: E,
        field: Field<N, FM>,
    ) -> GasResult<N::SumType> {
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

    pub async fn count<E: PgExecutionContext, FM: ModelMeta, T: AsPgType>(
        self,
        ctx: E,
        field: Field<T, FM>,
    ) -> GasResult<i64> {
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
        let tmp = self.includes.first().map(|it| it.1).unwrap_or(&[]);

        // sql
        let fields = M::FIELDS
            .iter()
            .chain(tmp.iter())
            .map(|f| format!("{} AS {}", f.full_name, f.alias_name))
            .reduce(|acc, cur| format!("{}, {}", acc, cur))
            .expect("no fields");

        let mut sql = SqlQuery::from(format!("SELECT {} FROM {}", fields, M::TABLE_NAME));

        for include in self.includes {
            sql.append_str(include.0.as_str());
        }

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

        for include in self.includes {
            sql.append_str(include.0.as_str());
        }

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
