use gas_shared::error::GasSharedError;
use gas_shared::migrations::parse_migrations_from_dir;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::path::PathBuf;
use syn::LitStr;

pub fn load_migrations_impl(input: TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let project_root =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env variable not set");

    let migrations_dir: LitStr = syn::parse(input)?;

    let scripts =
        parse_migrations_from_dir(PathBuf::from(project_root).join(migrations_dir.value()));

    match scripts {
        Ok(scripts) => {
            let struct_defs = scripts.iter().map(|(forwards, backwards)| {
                quote! {
                    gas::migrations::MigrationScript::new(#forwards,#backwards)
                }
            });

            Ok(quote! {
                Ok::<_, gas::error::GasError>(gas::migrations::Migrator::from([#(#struct_defs),*]))
            })
        }
        Err(GasSharedError::MigrationsNotDefined) => Ok(quote! {
            Err::<gas::migrations::Migrator<0>, gas::error::GasError>(
                gas::error::GasError::SharedError(gas::error::GasSharedError::MigrationsNotDefined)
            )
        }),
        Err(err) => Err(syn::Error::new(Span::call_site(), err.to_string())),
    }
}
