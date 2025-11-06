use crate::sql::{AsSql, PgParams};
use std::ops::{BitAnd, BitOr};

#[derive(Debug, Clone)]
pub enum Condition {
    Basic(String),
    And {
        lhs: Box<Condition>,
        rhs: Box<Condition>,
    },
    Or {
        lhs: Box<Condition>,
        rhs: Box<Condition>,
    },
}

#[derive(Debug, Clone)]
pub struct EqExpression {
    pub condition: Condition,
    pub params: Vec<PgParams>,
}

impl EqExpression {
    pub(crate) const fn new(condition: Condition, params: Vec<PgParams>) -> EqExpression {
        EqExpression { condition, params }
    }

    // consumes other fully so mutability is not a big problem
    pub fn and(mut self, mut other: EqExpression) -> EqExpression {
        self.condition = Condition::And {
            lhs: Box::from(self.condition),
            rhs: Box::from(other.condition),
        };
        self.params.append(&mut other.params);

        self
    }

    pub fn or(mut self, mut other: EqExpression) -> EqExpression {
        self.condition = Condition::Or {
            lhs: Box::from(self.condition),
            rhs: Box::from(other.condition),
        };
        self.params.append(&mut other.params);

        self
    }
}

impl BitAnd for EqExpression {
    type Output = EqExpression;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

impl BitOr for EqExpression {
    type Output = EqExpression;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.or(rhs)
    }
}

impl AsSql for Condition {
    fn as_sql(&self) -> String {
        match self {
            Condition::Basic(s) => s.to_string(),
            Condition::And { lhs, rhs } => format!("({}) AND ({})", lhs.as_sql(), rhs.as_sql()),
            Condition::Or { lhs, rhs } => format!("({}) OR ({})", lhs.as_sql(), rhs.as_sql()),
        }
    }
}
