use crate::error::GasCliResult;
use crate::util::sql_query::SqlQuery;
use std::fmt::Display;

pub mod diff;
mod graph;
pub mod helpers;
pub mod variants;

#[derive(Clone, Debug)]
pub struct MigrationScript {
    pub forward: String,
    pub backward: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum FieldState {
    Existing,
    InverseDropped,
}

impl FieldState {
    pub fn flip(&self) -> Self {
        match self {
            Self::Existing => Self::InverseDropped,
            Self::InverseDropped => Self::Existing,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FieldDependency<'a> {
    pub table_name: &'a str,
    pub name: &'a str,
    pub state: FieldState,
}

pub trait ModelChangeActor: Display {
    fn forward_sql(&self) -> GasCliResult<SqlQuery>;

    fn backward_sql(&self) -> GasCliResult<SqlQuery>;

    fn provides(&self) -> Box<[FieldDependency<'_>]> {
        Box::from([])
    }

    // fields that this operation's `forward_sql` depends on
    // operations that provide those fields will be executed before this one
    //  e.g. create table with a foreign key should require that the related table
    //      be created before this one
    fn depends_on(&self) -> Box<[FieldDependency<'_>]> {
        Box::from([])
    }

    fn provides_inverted(&self) -> Box<[FieldDependency<'_>]> {
        self.provides()
            .into_iter()
            .map(|mut it| {
                it.state = it.state.flip();
                it
            })
            .collect()
    }

    fn depends_on_inverted(&self) -> Box<[FieldDependency<'_>]> {
        self.depends_on()
            .into_iter()
            .map(|mut it| {
                it.state = it.state.flip();
                it
            })
            .collect()
    }
}
