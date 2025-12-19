use object::{Object, ObjectSection};
use std::fs;
use std::mem::MaybeUninit;
use std::path::Path;

#[derive(Debug)]
pub struct FieldMeta {
    pub table_name: *const str,
    pub full_name: *const str,
    pub name: *const str,
    pub alias_name: *const str,
    pub struct_name: *const str,
    pub pg_type: gas::internals::PgType,
    pub flags: gas::FieldFlags,
    pub index: usize,
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1);
    let args = args.collect::<Vec<_>>();

    if args.is_empty() {
        return Err(anyhow::anyhow!("empty argument"));
    }

    let binary_path = &args[0];
    let binary_path = Path::new(binary_path);

    if !binary_path.exists() {
        return Err(anyhow::anyhow!(
            "binary path not found in {}",
            binary_path.display()
        ));
    }

    dbg!(&binary_path);

    let binding = fs::read(binary_path)?;
    let file = object::File::parse(&*binding)?;

    let sz = size_of::<FieldMeta>();

    for section in file.sections() {
        let data = section.data()?;

        if !matches!(section.segment_name()?, Some(segment) if segment == "__gas_internals")
            && !section.name()?.starts_with("__gas_internals")
        {
            continue;
        }

        if data.len() % sz != 0 {
            return Err(anyhow::anyhow!("invalid section size"));
        }

        println!("looking at: {}", section.name()?);

        for i in 0..(data.len() / sz) {
            let f = &data[(i * sz)..((i + 1) * sz)];

            assert_eq!(f.len(), sz);

            let mut meta = MaybeUninit::<FieldMeta>::uninit();
            let meta = unsafe {
                std::ptr::copy_nonoverlapping(f.as_ptr(), meta.as_mut_ptr() as *mut u8, sz);
                meta.assume_init()
            };

            dbg!(&meta);
        }
        break;
    }

    Ok(())
}
