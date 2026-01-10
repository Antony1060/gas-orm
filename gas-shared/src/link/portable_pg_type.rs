use crate::error::GasSharedError;
use crate::internals::PgType;
use crate::link::FixedStr;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PortablePgType {
    Raw(PgType),
    ForeignKey {
        key_sql_type: FixedStr,
        target_table_name: FixedStr,
        target_column_name: FixedStr,
    },
}

impl TryFrom<PgType> for PortablePgType {
    type Error = GasSharedError;

    fn try_from(pg_type: PgType) -> Result<Self, Self::Error> {
        Ok(match pg_type {
            PgType::FOREIGN_KEY {
                key_type,
                target_field,
                ..
            } => Self::ForeignKey {
                key_sql_type: FixedStr::try_from(key_type.as_sql_type(false).as_ref())?,
                target_table_name: FixedStr::try_from(target_field.table_name)?,
                target_column_name: FixedStr::try_from(target_field.name)?,
            },
            _ => Self::Raw(pg_type),
        })
    }
}

impl PortablePgType {
    #[allow(clippy::missing_safety_doc)]
    pub const fn from_unchecked(pg_type: PgType) -> Self {
        match pg_type {
            PgType::FOREIGN_KEY {
                key_type,
                target_field,
                ..
            } => Self::ForeignKey {
                key_sql_type: FixedStr::from_panicking(key_type.as_sql_type_const(false)),
                target_table_name: FixedStr::from_panicking(target_field.table_name),
                target_column_name: FixedStr::from_panicking(target_field.name),
            },
            _ => Self::Raw(pg_type),
        }
    }
}

impl From<PortablePgType> for PgType {
    fn from(pg_type: PortablePgType) -> Self {
        match pg_type {
            PortablePgType::Raw(pg_type) => pg_type,
            PortablePgType::ForeignKey { .. } => todo!(),
        }
    }
}
