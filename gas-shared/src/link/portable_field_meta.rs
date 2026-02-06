use crate::error::GasSharedError;
use crate::link::portable_pg_type::PortablePgType;
use crate::link::FixedStr;
use crate::{FieldFlags, FieldMeta};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PortableFieldMeta {
    pub table_name: FixedStr,
    pub name: FixedStr,
    pub pg_type: PortablePgType,
    pub default_sql: Option<FixedStr>,
    pub flags: FieldFlags,
    pub index: usize,
}

impl TryFrom<FieldMeta> for PortableFieldMeta {
    type Error = GasSharedError;

    fn try_from(meta: FieldMeta) -> Result<Self, Self::Error> {
        Ok(Self {
            table_name: FixedStr::try_from(meta.table_name)?,
            name: FixedStr::try_from(meta.name)?,
            pg_type: PortablePgType::try_from(meta.pg_type)?,
            default_sql: meta.default_sql.map(FixedStr::try_from).transpose()?,
            flags: meta.flags,
            index: meta.index,
        })
    }
}

impl PortableFieldMeta {
    pub const fn from_unchecked(meta: FieldMeta) -> Self {
        Self {
            table_name: FixedStr::from_panicking(meta.table_name),
            name: FixedStr::from_panicking(meta.name),
            pg_type: PortablePgType::from_unchecked(meta.pg_type),
            default_sql: {
                if let Some(sql) = meta.default_sql {
                    Some(FixedStr::from_panicking(sql))
                } else {
                    None
                }
            },
            flags: meta.flags,
            index: meta.index,
        }
    }
}
