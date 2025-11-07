use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;

fn proc_type_to_pg_type(ty: &syn::Type) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Type::Path(path) = ty else {
        Err(syn::Error::new(ty.span(), "type must be a path type"))?
    };

    Ok(quote! {PgType::TEXT})
}

fn model_impl(_args: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let mut input = syn::parse::<syn::ItemStruct>(input)?;

    dbg!(&input);

    let mod_identifier = Ident::new(&input.ident.to_string().to_lowercase(), Span::call_site());

    // TODO: correct PgType
    let fields = input
        .fields
        .iter()
        .filter_map(|field| Some((field.ident.clone()?, field.ty.clone())))
        .map(|(ident, ty)| {
            let pg_type_str = proc_type_to_pg_type(&ty)?;
            Ok(quote!(pub const #ident: Field<#ty> = Field::new(stringify!(#ident), #pg_type_str);))
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    input.ident = Ident::new("Model", Span::call_site());

    // TODO: better attrs support
    Ok(quote! {
        pub mod #mod_identifier {
            #![allow(non_upper_case_globals)]
            use gas::{Field, ModelOps, PgType};

            #(#fields)*

            #input

            impl ModelOps for Model {
                fn table_name() -> &'static str {
                    concat!(stringify!(#mod_identifier), "s")
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
