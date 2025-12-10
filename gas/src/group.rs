use crate::connection::PgExecutionContext;
use crate::internals::{AsPgType, Numeric, SqlQuery, SqlStatement};
use crate::ops::select::SelectBuilder;
use crate::row::{FromRow, FromRowNamed, ResponseCtx, Row};
use crate::sort::{SortDefinition, SortDirection, SortOp};
use crate::{Field, GasResult, ModelMeta};
use std::num::NonZeroUsize;

pub enum GroupSorting {
    Key,
    Aggregate,
}

impl GroupSorting {
    fn sql_select_name(&self) -> &'static str {
        match self {
            GroupSorting::Key => "key",
            GroupSorting::Aggregate => "aggregate",
        }
    }

    pub fn asc(&self) -> SortDefinition {
        SortDefinition::from(SortOp {
            field_full_name: self.sql_select_name(),
            direction: SortDirection::Ascending,
        })
    }

    pub fn desc(&self) -> SortDefinition {
        SortDefinition::from(SortOp {
            field_full_name: self.sql_select_name(),
            direction: SortDirection::Descending,
        })
    }
}

pub struct Group<M: ModelMeta + 'static, G: AsPgType + 'static> {
    field: Field<G, M::Id>,
    select: SelectBuilder<M>,

    sort: Option<SortDefinition>,
    limit: Option<NonZeroUsize>,
}

impl<M: ModelMeta, G: AsPgType + 'static> Group<M, G> {
    pub fn new(field: Field<G, M::Id>, select: SelectBuilder<M>) -> Self {
        Self {
            field,
            select,
            sort: None,
            limit: None,
        }
    }

    pub fn sort(mut self, sort_definition: SortDefinition) -> Self {
        self.sort = Some(sort_definition);
        self
    }

    pub fn limit(mut self, items: usize) -> Self {
        self.limit = NonZeroUsize::new(items);
        self
    }

    pub async fn sum<E: PgExecutionContext, N: Numeric>(
        self,
        ctx: E,
        field: Field<N, M::Id>,
    ) -> GasResult<Vec<Summed<G, N::SumType>>> {
        let aggregate_call = format!("SUM({})", field.full_name);
        let (sql, params) = self.build_aggregate_query(&aggregate_call);

        ctx.execute_parsed(sql, &params).await
    }

    pub async fn count<E: PgExecutionContext, T: AsPgType>(
        self,
        ctx: E,
        field: Field<T, M::Id>,
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

        if let Some(ref sort) = self.sort
            && let Some(sort_sql) = sort.as_sql()
        {
            sql.append_str(" ORDER BY ");
            sql.append_query(sort_sql);
        }

        if let Some(limit) = self.limit {
            sql.append_str(&format!(" LIMIT {}", limit.get()));
        }

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
    fn from_row(ctx: &ResponseCtx, row: &Row) -> GasResult<Self> {
        Ok(Self {
            key: G::from_row_named(ctx, row, "key")?,
            count: <i64 as FromRowNamed>::from_row_named(ctx, row, "aggregate")?,
        })
    }
}

#[derive(Debug)]
pub struct Summed<G: AsPgType, N: Numeric> {
    pub key: G,
    pub sum: N,
}

impl<G: AsPgType, N: Numeric> FromRow for Summed<G, N> {
    fn from_row(ctx: &ResponseCtx, row: &Row) -> GasResult<Self> {
        Ok(Self {
            key: G::from_row_named(ctx, row, "key")?,
            sum: N::from_row_named(ctx, row, "aggregate")?,
        })
    }
}
