use crate::binary::FieldEntry;
use crate::error::GasCliResult;
use crate::sync::diff::ModelChangeActor;
use crate::sync::variants::create_table::CreateTableModelActor;
use crate::util::sql_query::SqlQuery;

pub struct DropTableModelActor<'a> {
    create_table_actor: CreateTableModelActor<'a>,
}

impl<'a> DropTableModelActor<'a> {
    pub fn new_boxed(entry: FieldEntry<'a>) -> Box<dyn ModelChangeActor + '_> {
        Box::from(Self {
            create_table_actor: CreateTableModelActor { entry },
        })
    }
}

impl<'a> ModelChangeActor for DropTableModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        self.create_table_actor.backward_sql()
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        self.create_table_actor.forward_sql()
    }
}
