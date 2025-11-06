use crate::condition::EqExpression;
use crate::sql::AsSql;

#[derive(Debug, Clone)]
pub struct SelectBuilder {
    pub(crate) table: &'static str,
    pub(crate) filter: Option<EqExpression>,
}

impl SelectBuilder {}

impl AsSql for SelectBuilder {
    fn as_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);

        if let Some(ref filter) = self.filter {
            sql.push_str("\n\tWHERE ");
            sql.push_str(&filter.condition.as_sql());
        }

        sql.push(';');

        sql
    }
}
