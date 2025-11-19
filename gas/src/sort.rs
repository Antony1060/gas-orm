use crate::internals::SqlQuery;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Display for SortDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SortDirection::Ascending => write!(f, "ASC"),
            SortDirection::Descending => write!(f, "DESC"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SortOp {
    pub(crate) field_full_name: &'static str,
    pub(crate) direction: SortDirection,
}

#[derive(Debug, Clone, Default)]
pub struct SortDefinition {
    sorts: Vec<SortOp>,
}

impl SortDefinition {
    pub(crate) fn new() -> SortDefinition {
        SortDefinition { sorts: vec![] }
    }

    pub fn then(mut self, mut definition: SortDefinition) -> SortDefinition {
        self.sorts.append(&mut definition.sorts);
        self
    }

    pub fn as_sql(&self) -> Option<SqlQuery<'_>> {
        let mut sql = SqlQuery::new();

        let ops = self
            .sorts
            .iter()
            .map(|op| format!("{} {}", op.field_full_name, op.direction))
            .reduce(|acc, curr| format!("{}, {}", acc, curr));

        ops.map(|ops| {
            sql.append_str(&ops);
            sql
        })
    }
}

impl From<SortOp> for SortDefinition {
    fn from(value: SortOp) -> Self {
        let mut sort = Self::new();
        sort.sorts.push(value);

        sort
    }
}
