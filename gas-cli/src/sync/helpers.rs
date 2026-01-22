use crate::error::GasCliResult;
use crate::sync::diff::ModelChangeActor;
use crate::util::sql_query::SqlQuery;

pub mod diff {
    use crate::sync::diff::ModelChangeActor;
    use crate::sync::helpers::InverseChangeActor;

    pub fn invert<'a>(other: Box<dyn ModelChangeActor + 'a>) -> Box<dyn ModelChangeActor + 'a> {
        InverseChangeActor::new_boxed(other)
    }
}

pub struct InverseChangeActor<'a> {
    source: Box<dyn ModelChangeActor + 'a>,
}

impl<'a> InverseChangeActor<'a> {
    pub fn new_boxed(source: Box<dyn ModelChangeActor + 'a>) -> Box<dyn ModelChangeActor + 'a> {
        Box::from(Self { source })
    }
}

impl<'a> ModelChangeActor for InverseChangeActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        self.source.backward_sql()
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        self.source.forward_sql()
    }
}
