mod utils;

use std::fs;
use std::path::Path;

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

    let metas = utils::binary::parse_fields(&file)?;
    let fields = utils::binary::organize_fields(&metas)?;

    dbg!(&fields);

    Ok(())
}
