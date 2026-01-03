use crate::const_util;
use crate::field::FieldMeta;

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
    pub const fn as_sql_type(&self, is_serial: bool) -> &'static str {
        match self {
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
            PgType::FOREIGN_KEY {
                key_type,
                target_field,
            } => {
                // NOTE: 256 should be enough
                //  64 is currently the name for table_name and name because of the FixedStr bound in gas_shared::link
                //  2 * 64 + len(" REFERENCES ()")(14) + len(largest_as_sql_type_value)(24) = 166
                const_util::join_static_str::<256>(&[
                    key_type.as_sql_type(false),
                    " REFERENCES ",
                    target_field.table_name,
                    "(",
                    target_field.name,
                    ")",
                ])
            }
        }
    }
}
