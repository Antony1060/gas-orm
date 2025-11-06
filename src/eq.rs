use crate::condition::{Condition, EqExpression};
use crate::sql::{Field, PgEq, PgParams};

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
