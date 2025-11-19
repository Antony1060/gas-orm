#![allow(private_bounds)]

use crate::condition::EqExpression;
use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::internals::{SqlQuery, SqlStatement};
use crate::model::ModelMeta;
use crate::sort::SortDefinition;
use crate::GasResult;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Default)]
pub struct SelectBuilder<T> {
    pub(crate) filter: Option<EqExpression>,
    pub(crate) sort: Option<SortDefinition>,
    pub(crate) limit: Option<NonZeroUsize>,
    _marker: PhantomData<T>,
}

impl<T: ModelMeta> SelectBuilder<T> {
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

    pub async fn find_all<E: PgExecutionContext>(self, ctx: E) -> GasResult<Vec<T>> {
        let (sql, params) = self.build(true);

        let items = ctx.execute_parsed::<T>(sql, &params).await?;

        Ok(items)
    }

    pub async fn find_one<E: PgExecutionContext>(self, ctx: E) -> GasResult<Option<T>> {
        let (mut sql, params) = self.build(false);

        sql.append_str(" LIMIT 1");

        let mut items = ctx.execute_parsed::<T>(sql, &params).await?;

        if items.len() > 1 {
            return Err(GasError::UnexpectedResponse(
                format!("find_one: got {}, expected <= 1", items.len()).into(),
            ));
        }

        Ok(items.pop())
    }

    // include_limit is important here because of find_one
    //  if limit is built into the query and then later on enforced by find_one,
    //  the query would fail; not very nice way to enforce an invariant but eh
    fn build<'a>(self, include_limit: bool) -> SqlStatement<'a> {
        // sql
        let fields = T::FIELDS
            .iter()
            .map(|f| format!("{} AS {}", f.full_name, f.alias_name))
            .reduce(|acc, cur| format!("{}, {}", acc, cur))
            .expect("no fields");

        let mut sql = SqlQuery::from(format!("SELECT {} FROM {}", fields, T::TABLE_NAME));

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
}
