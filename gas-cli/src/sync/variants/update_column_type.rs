use crate::binary::TableSpec;
use crate::error::GasCliResult;
use crate::sync::variants::add_column::AddColumnModelActor;
use crate::sync::{helpers, FieldDependency, ModelChangeActor};
use crate::util::sql_query::SqlQuery;
use gas_shared::link::PortableFieldMeta;
use std::fmt::{Display, Formatter};

pub struct UpdateColumnTypeModelActor<'a> {
    add_column_actor: Box<dyn ModelChangeActor + 'a>,
    drop_column_actor: Box<dyn ModelChangeActor + 'a>,
}

impl<'a> UpdateColumnTypeModelActor<'a> {
    pub fn new_boxed(
        old_table: TableSpec<'a>,
        old_field: &'a PortableFieldMeta,
        field: &'a PortableFieldMeta,
    ) -> Box<dyn ModelChangeActor + 'a> {
        Box::new(UpdateColumnTypeModelActor {
            add_column_actor: AddColumnModelActor::new_boxed(old_table.clone(), field),
            drop_column_actor: helpers::diff::invert(AddColumnModelActor::new_boxed(
                old_table, old_field,
            )),
        })
    }
}

impl<'a> Display for UpdateColumnTypeModelActor<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UpdateColumnType[{} then {}]",
            self.drop_column_actor, self.add_column_actor,
        )
    }
}

impl<'a> ModelChangeActor for UpdateColumnTypeModelActor<'a> {
    fn forward_sql(&self) -> GasCliResult<SqlQuery> {
        let mut sql = SqlQuery::new();

        sql.push_str(&self.drop_column_actor.forward_sql()?);
        sql.push_str(";\n");
        sql.push_str(&self.add_column_actor.forward_sql()?);

        Ok(sql)
    }

    fn backward_sql(&self) -> GasCliResult<SqlQuery> {
        let mut sql = SqlQuery::new();

        sql.push_str(&self.add_column_actor.backward_sql()?);
        sql.push_str(";\n");
        sql.push_str(&self.drop_column_actor.backward_sql()?);

        Ok(sql)
    }

    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        self.add_column_actor
            .provides()
            .into_iter()
            .chain(self.drop_column_actor.provides())
            .collect()
    }

    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        self.add_column_actor
            .depends_on()
            .into_iter()
            .chain(self.drop_column_actor.depends_on())
            .collect()
    }
}
