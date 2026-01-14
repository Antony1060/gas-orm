use crate::internals::PgType;
use std::fmt::{Debug, Formatter};

#[repr(u8)]
pub enum FieldFlag {
    Nullable = 1 << 0,
    PrimaryKey = 1 << 1,
    CompositePrimaryKey = 1 << 2,
    Unique = 1 << 3,
    Serial = 1 << 4,
    ForeignKey = 1 << 5,

    // ORM specific
    Virtual = 1 << 6,
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldFlags(pub u8);

impl FieldFlags {
    pub const fn has_flag(&self, flag: FieldFlag) -> bool {
        (self.0 & (flag as u8)) != 0
    }
}

impl Debug for FieldFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0b{:b}", self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FieldMeta {
    // a lot of names
    pub table_name: &'static str,  // table
    pub full_name: &'static str,   // table.column
    pub name: &'static str,        // column
    pub alias_name: &'static str,  // table_column
    pub struct_name: &'static str, // table_column
    pub pg_type: PgType,
    pub flags: FieldFlags,
    pub index: usize,
}
