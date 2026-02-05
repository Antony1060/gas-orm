use gas_shared::link::PortableFieldMeta;
use gas_shared::FieldFlag;
use std::borrow::Cow;

// not the same thing as in the ORM
pub type SqlQuery = String;

pub fn gen_column_descriptor_sql(field: &PortableFieldMeta) -> SqlQuery {
    let mut sql = SqlQuery::new();

    let sql_type: Cow<'_, str> = field
        .pg_type
        .as_sql_type(field.flags.has_flag(FieldFlag::Serial));

    sql.push_str(field.name.as_ref());
    sql.push(' ');
    sql.push_str(&sql_type);

    if !field.flags.has_flag(FieldFlag::Nullable) {
        sql.push_str(" NOT NULL");
    }

    if field.flags.has_flag(FieldFlag::Unique) {
        sql.push_str(" UNIQUE");
    }

    if let Some(ref default_sql) = field.default_sql {
        sql.push_str(" DEFAULT (");
        sql.push_str(default_sql.as_ref());
        sql.push(')');
    }

    sql
}
