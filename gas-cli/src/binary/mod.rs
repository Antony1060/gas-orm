use crate::error::{GasCliError, GasCliResult};
use gas_shared::link::PortableFieldMeta;
use object::{Object, ObjectSection};
use std::collections::BTreeMap;
use std::mem::MaybeUninit;
use std::path::PathBuf;
use tokio::fs;

// TODO: move to shared
const LINK_SECTION_PREFIX: &str = ".__gas_internals";

pub type BinaryFields = BTreeMap<String, Vec<PortableFieldMeta>>;

pub struct ProjectModelState {
    pub fields: BinaryFields,
}

pub struct FieldEntry<'a> {
    pub table: &'a str,
    pub fields: &'a [PortableFieldMeta],
}

impl ProjectModelState {
    pub async fn from_binary(path: &PathBuf) -> GasCliResult<ProjectModelState> {
        let binary_contents = fs::read(path).await?;
        let file = object::File::parse(&*binary_contents)?;

        let fields = Self::parse_fields(&file)?;

        Ok(ProjectModelState {
            fields: Self::get_organized(fields),
        })
    }

    fn get_organized(fields: Box<[PortableFieldMeta]>) -> BinaryFields {
        let mut map = BinaryFields::new();

        for meta in fields {
            let fields = map.entry(String::from(&meta.table_name)).or_default();
            fields.push(meta);
        }

        map
    }

    fn parse_fields(file: &object::File) -> GasCliResult<Box<[PortableFieldMeta]>> {
        let mut fields = Vec::new();

        let meta_size = size_of::<PortableFieldMeta>();

        for section in file.sections() {
            let data = section.data()?;

            if !matches!(section.segment_name()?, Some(segment) if segment == LINK_SECTION_PREFIX)
                && !section.name()?.starts_with(LINK_SECTION_PREFIX)
            {
                continue;
            }

            if data.len() % meta_size != 0 {
                return Err(GasCliError::BinaryParseError("invalid section size"));
            }

            for i in 0..(data.len() / meta_size) {
                let field_bytes = &data[(i * meta_size)..((i + 1) * meta_size)];

                assert_eq!(field_bytes.len(), meta_size);

                let mut meta = MaybeUninit::<PortableFieldMeta>::zeroed();
                let meta = unsafe {
                    // SAFETY:
                    //  - field_bytes should be valid for reads of meta_size
                    //  - meta should is valid for reads of it's size
                    //  - field_bytes and meta don't overlap
                    std::ptr::copy_nonoverlapping(
                        field_bytes.as_ptr(),
                        meta.as_mut_ptr() as *mut u8,
                        meta_size,
                    );

                    // SAFETY: meta should be initialized here, lol, IDK
                    meta.assume_init()
                };

                fields.push(meta);
            }
        }

        Ok(fields.into_boxed_slice())
    }
}

impl<'a> From<(&'a String, &'a Vec<PortableFieldMeta>)> for FieldEntry<'a> {
    fn from(value: (&'a String, &'a Vec<PortableFieldMeta>)) -> Self {
        FieldEntry {
            table: value.0,
            fields: value.1,
        }
    }
}
