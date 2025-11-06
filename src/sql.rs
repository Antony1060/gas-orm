use crate::builder::SelectBuilder;
use crate::condition::EqExpression;
use std::marker::PhantomData;

pub enum PgType {
    TEXT,
    INT,
    // TODO: DECIMAL
    FLOAT,
}

#[derive(Debug, Clone)]
pub enum PgParams {
    T(String),
    I(i32),
    F(f64),
}

pub struct Field<T> {
    pub name: &'static str,
    pub pg_type: PgType,
    _mark: PhantomData<T>,
}

impl<T> Field<T> {
    pub const fn new(name: &'static str, pg_type: PgType) -> Self {
        Self {
            name,
            pg_type,
            _mark: PhantomData,
        }
    }
}

pub trait AsSql {
    fn as_sql(self: &Self) -> String;
}

pub trait ModelOps {
    fn table_name() -> &'static str;

    fn filter(cond_fn: fn() -> EqExpression) -> SelectBuilder {
        SelectBuilder {
            table: Self::table_name(),
            filter: Some(cond_fn()),
        }
    }
}

pub trait PgEq<T> {
    fn eq(&self, other: T) -> EqExpression;

    fn neq(&self, other: T) -> EqExpression;

    fn lt(&self, other: T) -> EqExpression;

    fn lte(&self, other: T) -> EqExpression;

    fn gt(&self, other: T) -> EqExpression;

    fn gte(&self, other: T) -> EqExpression;

    fn one_of(&self, other: &[T]) -> EqExpression;
}
