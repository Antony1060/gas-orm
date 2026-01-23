use proc_macro::TokenStream;
use quote::quote;
use syn::LitStr;

pub fn load_migrations_impl(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let migrations_dir: LitStr = syn::parse(input)?;

    Ok(quote! {
        gas::migrations::Migrator::from_raw(&[#migrations_dir])
    }
    .into())
}
