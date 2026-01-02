use gas_shared::link::{FixedStr, PortableFieldMeta};
use object::{Object, ObjectSection};
use std::collections::HashMap;
use std::mem::MaybeUninit;

pub fn parse_fields(file: &object::File) -> anyhow::Result<Box<[PortableFieldMeta]>> {
    let mut fields = Vec::new();

    let sz = size_of::<PortableFieldMeta>();

    for section in file.sections() {
        let data = section.data()?;

        if !matches!(section.segment_name()?, Some(segment) if segment == ".__gas_internals")
            && !section.name()?.starts_with(".__gas_internals")
        {
            continue;
        }

        if data.len() % sz != 0 {
            return Err(anyhow::anyhow!("invalid section size"));
        }

        for i in 0..(data.len() / sz) {
            let f = &data[(i * sz)..((i + 1) * sz)];

            assert_eq!(f.len(), sz);

            let mut meta = MaybeUninit::<PortableFieldMeta>::uninit();
            let meta = unsafe {
                std::ptr::copy_nonoverlapping(f.as_ptr(), meta.as_mut_ptr() as *mut u8, sz);
                meta.assume_init()
            };

            fields.push(meta);
        }
    }

    Ok(fields.into_boxed_slice())
}

pub fn organize_fields(
    metas: &[PortableFieldMeta],
) -> anyhow::Result<HashMap<&FixedStr, Vec<&PortableFieldMeta>>> {
    let mut map = HashMap::new();

    for meta in metas {
        let fields = map.entry(&meta.table_name).or_insert_with(Vec::new);
        fields.push(meta);
    }

    Ok(map)
}
