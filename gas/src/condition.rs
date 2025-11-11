use crate::sql_query::SqlQuery;
use crate::{AsSql, PgParams};
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
    fn as_sql(&self) -> SqlQuery {
        let mut sql = SqlQuery::new("");

        match self {
            Condition::Basic(s) => sql.append_str(s),
            Condition::And { lhs, rhs } => {
                // eeeeh
                sql.append_str("(");
                sql.append_query(lhs.as_sql());
                sql.append_str(") AND (");
                sql.append_query(rhs.as_sql());
                sql.append_str(")");
            }
            Condition::Or { lhs, rhs } => {
                sql.append_str("(");
                sql.append_query(lhs.as_sql());
                sql.append_str(") OR (");
                sql.append_query(rhs.as_sql());
                sql.append_str(")");
            }
        };

        sql
    }
}
