#![allow(private_bounds)]

use crate::condition::EqExpression;
use crate::connection::PgExecutionContext;
use crate::error::GasError;
use crate::internals::{SqlQuery, SqlStatement};
use crate::model::ModelMeta;
use crate::GasResult;
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

    pub fn filter<F: Fn() -> EqExpression>(mut self, cond_fn: F) -> Self {
        self.filter = Some(cond_fn());
        self
    }

    pub async fn find_all<E: PgExecutionContext>(self, ctx: &E) -> GasResult<Vec<T>> {
        let (sql, params) = self.build();

        let items = ctx.execute_parsed::<T>(sql, &params).await?;

        Ok(items)
    }

    pub async fn find_one<E: PgExecutionContext>(self, ctx: &E) -> GasResult<Option<T>> {
        let (mut sql, params) = self.build();

        sql.append_str(" LIMIT 1");

        let mut items = ctx.execute_parsed::<T>(sql, &params).await?;

        if items.len() > 1 {
            return Err(GasError::UnexpectedResponse(format!(
                "find_one: got {}, expected <= 1",
                items.len()
            )));
        }

        Ok(items.pop())
    }

    fn build(self) -> SqlStatement {
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

        // params
        let params = self
            .filter
            .map(|it| it.params.into_boxed_slice())
            .unwrap_or_else(|| Box::new([]));

        (sql, params)
    }
}
