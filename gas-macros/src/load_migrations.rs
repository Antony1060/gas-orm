use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use std::path::PathBuf;
use syn::LitStr;

const SCRIPT_SEPARATOR: &str = "-- GAS_ORM(forward_backward_separator)";

pub fn load_migrations_impl(input: TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let project_root =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env variable not set");

    let migrations_dir: LitStr = syn::parse(input)?;

    let scripts_path = PathBuf::from(project_root)
        .join(migrations_dir.value())
        .join("scripts");

    if !scripts_path.exists() || !scripts_path.is_dir() {
        return Ok(quote! {
            Err::<gas::migrations::Migrator<0>, gas::error::GasError>(gas::error::GasError::MigrationsNotDefined)
        });
    }

    let files: Vec<_> = std::fs::read_dir(scripts_path)
        .expect("read_dir failed")
        // error safety is my passion
        .map(Result::unwrap)
        .filter(|file| file.file_type().unwrap().is_file())
        .map(|file| file.path().display().to_string())
        .filter(|path| path.ends_with(".sql"))
        .sorted()
        .collect();

    let script_contents_raw: Vec<_> = files
        .into_iter()
        .map(|path| std::fs::read_to_string(path).expect("failed to read file"))
        .collect();

    let mut parsed_scripts: Vec<(&str, &str)> = Vec::with_capacity(script_contents_raw.len());
    for script in script_contents_raw.iter() {
        let (forward, backward) = script
            .split_once(SCRIPT_SEPARATOR)
            .expect("failed to parse migration script");

        parsed_scripts.push((forward, backward));
    }

    let struct_defs = parsed_scripts.into_iter().map(|(forwards, backwards)| {
        quote! {
            gas::migrations::MigrationScript::new(#forwards,#backwards)
        }
    });

    Ok(quote! {
        Ok::<_, gas::error::GasError>(gas::migrations::Migrator::from([#(#struct_defs),*]))
    })
}
