use crate::internals::PgType;
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Debug)]
pub enum FieldRelationship {
    OneToOne,
    OneToMany,
    ManyToMany,
}

#[repr(u8)]
pub enum FieldFlags {
    PrimaryKey = 1 << 0,
    Serial = 1 << 1,
    Nullable = 1 << 2,
}

impl FieldFlags {
    pub fn in_bitmask(self, mask: u8) -> bool {
        (mask & (self as u8)) != 0
    }
}

#[derive(Debug)]
pub struct FieldMeta {
    // a lot of names
    pub full_name: &'static str,  // table.column
    pub name: &'static str,       // column
    pub alias_name: &'static str, // table_column
    pub pg_type: PgType,
    pub flags: u8,
    pub relationship: Option<FieldRelationship>,
}

#[derive(Debug)]
pub struct Field<T> {
    pub meta: FieldMeta,
    _marker: PhantomData<T>,
}

impl<T> Field<T> {
    pub const fn new(meta: FieldMeta) -> Self {
        Self {
            meta,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Field<T> {
    type Target = FieldMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}
