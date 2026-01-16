use crate::error::GasCliResult;
use crate::util::sql_query::SqlQuery;

pub trait ModelChangeActor {
    fn forward_sql(&self) -> GasCliResult<SqlQuery>;

    fn backward_sql(&self) -> GasCliResult<SqlQuery>;
}

// TODO
pub struct SampleModelActor {}

impl ModelChangeActor for SampleModelActor {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok("ALTER TABLE foo ADD id BIGINT;".to_string())
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        Ok("ALTER TABLE foo DROP COLUMN id;".to_string())
    }
}
