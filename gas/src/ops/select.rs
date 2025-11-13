#![allow(private_bounds)]

use crate::condition::EqExpression;
use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::sql_query::SqlQuery;
use crate::{AsSql, GasResult, ModelMeta, PgParam};
use std::marker::PhantomData;

#[derive(Debug, Clone, Default)]
pub struct SelectBuilder<T> {
    pub(crate) filter: Option<EqExpression>,
    _marker: PhantomData<T>,
}

impl<T: ModelMeta> SelectBuilder<T> {
    pub fn new() -> Self {
        Self {
            filter: None,
            _marker: PhantomData,
        }
    }

    pub fn filter(mut self, cond_fn: fn() -> EqExpression) -> Self {
        self.filter = Some(cond_fn());
        self
    }

    pub async fn find_all<E: PgExecutionContext>(self, ctx: &E) -> GasResult<Vec<T>> {
        let params = self.accumulate_params();
        let items = ctx.execute_parsed::<T>(self.as_sql(), params).await?;

        Ok(items)
    }

    pub async fn find_one<E: PgExecutionContext>(self, ctx: &E) -> GasResult<Option<T>> {
        let params = self.accumulate_params();

        let mut sql = self.as_sql();
        sql.append_str(" LIMIT 1");

        let mut items = ctx.execute_parsed::<T>(sql, params).await?;

        if items.len() > 1 {
            return Err(GasError::UnexpectedResponse(format!(
                "find_one: got {}, expected <= 1",
                items.len()
            )));
        }

        Ok(items.pop())
    }

    fn accumulate_params(&self) -> &[PgParam] {
        self.filter
            .as_ref()
            .map(|it| it.params.as_slice())
            .unwrap_or_else(|| &[])
    }
}

impl<T: ModelMeta> AsSql for SelectBuilder<T> {
    fn as_sql(&self) -> SqlQuery {
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

        sql.append_str(";");

        sql
    }
}
