use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse::Parse;
use syn::spanned::Spanned;

fn proc_type_to_pg_type(ty: &syn::Type) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Type::Path(path) = ty else {
        Err(syn::Error::new(ty.span(), "type must be a path type"))?
    };

    Ok(quote! {|| #ty::as_pg_type()})
}

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ModelArgs {
    table_name: String,
}

#[inline(always)]
fn model_impl(args: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let args = syn::parse::<ModelArgs>(args)?;
    let input = syn::parse::<syn::ItemStruct>(input)?;

    let mod_identifier = Ident::new(&input.ident.to_string().to_lowercase(), Span::call_site());

    let fields = input
        .fields
        .iter()
        .filter_map(|field| Some((field.ident.clone()?, field.ty.clone())))
        .map(|(ident, ty)| {
            let pg_type_tokens = proc_type_to_pg_type(&ty)?;
            Ok(quote!(pub const #ident: Field<#ty> = Field::new(stringify!(#ident), #pg_type_tokens);))
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let mut original_struct = input.clone();
    original_struct.ident = Ident::new("Model", Span::call_site());

    let table_name = args.table_name;

    Ok(quote! {
        pub mod #mod_identifier {
            #![allow(non_upper_case_globals)]
            use gas::{Field, ModelOps, pg_type::PgType, pg_type::AsPgType};

            #(#fields)*

            #original_struct

            impl ModelOps for Model {
                fn table_name() -> &'static str {
                    #table_name
                }
            }
        }
    }
    .into())
}

#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream {
    model_impl(args, input).unwrap_or_else(|err| err.to_compile_error().into())
}
