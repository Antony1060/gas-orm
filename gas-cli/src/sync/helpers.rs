use crate::error::GasCliResult;
use crate::sync::{FieldDependency, ModelChangeActor};
use crate::util::sql_query::SqlQuery;

pub mod diff {
    use super::*;

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

    // a "creative" operation's opposite for should be destructive
    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        self.source.provides_backwards()
    }

    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        self.source.depends_on_backwards()
    }

    fn provides_backwards(&self) -> Box<[FieldDependency<'_>]> {
        self.source.provides()
    }

    fn depends_on_backwards(&self) -> Box<[FieldDependency<'_>]> {
        self.source.depends_on()
    }
}
