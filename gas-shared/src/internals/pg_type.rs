use crate::field::FieldMeta;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum PgType {
    TEXT,
    SMALLINT,
    INTEGER,
    BIGINT,
    REAL,
    DOUBLE,
    DECIMAL,
    TIMESTAMP,
    #[allow(nonstandard_style)]
    TIMESTAMP_TZ,
    DATE,
    TIME,
    #[allow(nonstandard_style)]
    FOREIGN_KEY {
        key_type: &'static PgType,
        target_field: &'static FieldMeta,
    },
    IGNORED,
}

impl PgType {
    pub fn as_sql_type(&self, is_serial: bool) -> Cow<'static, str> {
        match self {
            PgType::FOREIGN_KEY {
                key_type,
                target_field,
            } => format!(
                "{} REFERENCES {}({})",
                key_type.as_sql_type(false),
                target_field.table_name,
                target_field.name
            )
            .into(),
            _ => self.as_sql_type_const(is_serial).into(),
        }
    }

    // NOTE: panics
    pub const fn as_sql_type_const(&self, is_serial: bool) -> &'static str {
        match self {
            PgType::FOREIGN_KEY { .. } => panic!("can not evaluate foreign key at const time"),

            PgType::TEXT => "TEXT",
            PgType::SMALLINT if is_serial => "SMALLSERIAL",
            PgType::SMALLINT => "SMALLINT",
            PgType::INTEGER if is_serial => "SERIAL",
            PgType::INTEGER => "INTEGER",
            PgType::BIGINT if is_serial => "BIGSERIAL",
            PgType::BIGINT => "BIGINT",
            PgType::REAL => "REAL",
            PgType::DOUBLE => "DOUBLE",
            PgType::DECIMAL => "DECIMAL",
            PgType::TIMESTAMP => "TIMESTAMP",
            PgType::TIMESTAMP_TZ => "TIMESTAMP WITH TIME ZONE",
            PgType::DATE => "DATE",
            PgType::TIME => "TIME",
            PgType::IGNORED => "",
        }
    }
}
