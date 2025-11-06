use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn model(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemStruct);

    dbg!(&input);

    let mod_identifier = Ident::new(&input.ident.to_string().to_lowercase(), Span::call_site());

    // TODO: correct PgType
    let fields = input
        .fields
        .iter()
        .filter_map(|field| Some((field.ident.clone()?, field.ty.clone())))
        .map(|(ident, ty)| quote!(pub const #ident: Field<#ty> = Field::new(stringify!(#ident), PgType::TEXT);));

    input.ident = Ident::new("Model", Span::call_site());

    // TODO: better attrs support
    quote! {
        pub mod #mod_identifier {
            #![allow(non_upper_case_globals)]
            use pg_orm::{Field, ModelOps, PgType};

            #(#fields)*

            #input

            impl ModelOps for Model {
                fn table_name() -> &'static str {
                    concat!(stringify!(#mod_identifier), "s")
                }
            }
        }
    }
    .into()
}
