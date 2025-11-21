pub mod def_model;
pub mod numeric;
pub mod pg_param;
pub mod pg_type;
pub mod sql_query;

use crate::FieldMeta;
pub use def_model::*;
pub(crate) use numeric::*;
pub use pg_param::*;
pub use pg_type::*;
pub use sql_query::*;
use std::any::TypeId;

pub fn generate_update_set_fields(fields: &[&FieldMeta]) -> String {
    fields
        .iter()
        .map(|field| format!("{}=?", field.name))
        .reduce(|acc, curr| format!("{}, {}", acc, curr))
        .unwrap_or_else(String::new)
}

pub fn type_id_of_value<T: 'static>(_: &T) -> TypeId {
    TypeId::of::<T>()
}
