use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_project_model::{CargoConfig, RustLibSource};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1);
    let args = args.collect::<Vec<_>>();

    if args.is_empty() {
        return Err(anyhow::anyhow!("empty argument"));
    }

    let project_path = &args[0];
    let project_path = Path::new(project_path);
    let cargo_toml_path = project_path.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!(
            "Cargo.toml not found in {}",
            project_path.display()
        ));
    }

    dbg!(&cargo_toml_path);

    // let abs_path = AbsPathBuf::try_from(absolute_utf8(cargo_toml_path)?);
    //
    // dbg!(&abs_path);

    // let manifest = ProjectManifest::from_manifest_file(abs_path.unwrap())?;
    //
    // dbg!(&manifest);

    let cargo_config = CargoConfig {
        sysroot: Some(RustLibSource::Discover),
        ..CargoConfig::default()
    };

    let load_cargo_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::Sysroot,
        prefill_caches: true,
    };

    let workspace = load_workspace_at(project_path, &cargo_config, &load_cargo_config, &|it| {
        println!("loading: {}", it)
    })?;

    println!("done loading");

    dbg!(&workspace);

    Ok(())
}
