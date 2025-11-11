use crate::condition::EqExpression;
use crate::sql_query::SqlQuery;
use crate::AsSql;

#[derive(Debug, Clone)]
pub struct SelectBuilder {
    pub(crate) table: &'static str,
    pub(crate) filter: Option<EqExpression>,
}

impl SelectBuilder {}

impl AsSql for SelectBuilder {
    fn as_sql(&self) -> SqlQuery {
        let mut sql = SqlQuery::from(format!("SELECT * FROM {}", self.table));

        if let Some(ref filter) = self.filter {
            sql.append_str("\n\tWHERE ");
            sql.append_query(filter.condition.as_sql());
        }

        sql.append_str(";");

        sql
    }
}
