use crate::error::GasError;
use crate::link::portable_pg_type::PortablePgType;
use crate::link::FixedStr;
use crate::{FieldFlags, FieldMeta};

#[derive(Debug)]
pub struct PortableFieldMeta {
    pub table_name: FixedStr,
    pub name: FixedStr,
    pub pg_type: PortablePgType,
    pub flags: FieldFlags,
}

impl TryFrom<FieldMeta> for PortableFieldMeta {
    type Error = GasError;

    fn try_from(meta: FieldMeta) -> Result<Self, Self::Error> {
        Ok(Self {
            table_name: FixedStr::try_from(meta.table_name)?,
            name: FixedStr::try_from(meta.name)?,
            pg_type: PortablePgType::try_from(meta.pg_type)?,
            flags: meta.flags,
        })
    }
}

impl PortableFieldMeta {
    #[allow(clippy::missing_safety_doc)]
    pub const unsafe fn from_unchecked(meta: FieldMeta) -> Self {
        unsafe {
            Self {
                table_name: FixedStr::from_unchecked(meta.table_name),
                name: FixedStr::from_unchecked(meta.name),
                pg_type: PortablePgType::from_unchecked(meta.pg_type),
                flags: meta.flags,
            }
        }
    }
}
