use crate::condition::{Condition, EqExpression};
use crate::{Field, PgParams};

pub trait PgEq<T> {
    fn eq(&self, other: T) -> EqExpression;

    fn neq(&self, other: T) -> EqExpression;

    fn lt(&self, other: T) -> EqExpression;

    fn lte(&self, other: T) -> EqExpression;

    fn gt(&self, other: T) -> EqExpression;

    fn gte(&self, other: T) -> EqExpression;

    fn one_of(&self, other: &[T]) -> EqExpression;
}

impl PgEq<&str> for Field<String> {
    fn eq(&self, other: &str) -> EqExpression {
        EqExpression::new(
            Condition::Basic(format!("{}=?", self.name)),
            vec![PgParams::T(other.to_string())],
        )
    }

    fn neq(&self, other: &str) -> EqExpression {
        EqExpression::new(
            Condition::Basic(format!("{}!=?", self.name)),
            vec![PgParams::T(other.to_string())],
        )
    }

    fn lt(&self, other: &str) -> EqExpression {
        EqExpression::new(
            Condition::Basic(format!("{}<?", self.name)),
            vec![PgParams::T(other.to_string())],
        )
    }

    fn lte(&self, other: &str) -> EqExpression {
        EqExpression::new(
            Condition::Basic(format!("{}<=?", self.name)),
            vec![PgParams::T(other.to_string())],
        )
    }

    fn gt(&self, other: &str) -> EqExpression {
        EqExpression::new(
            Condition::Basic(format!("{}>?", self.name)),
            vec![PgParams::T(other.to_string())],
        )
    }

    fn gte(&self, other: &str) -> EqExpression {
        EqExpression::new(
            Condition::Basic(format!("{}>=?", self.name)),
            vec![PgParams::T(other.to_string())],
        )
    }

    fn one_of(&self, other: &[&str]) -> EqExpression {
        let condition: String = format!(
            "{} IN ({})",
            self.name,
            other
                .iter()
                .map(|_| '?')
                .fold(String::new(), |acc, curr| format!("{acc}, {curr}"))
        );

        EqExpression::new(
            Condition::Basic(condition),
            other
                .into_iter()
                .map(|it| it.to_string())
                .map(PgParams::T)
                .collect(),
        )
    }
}
