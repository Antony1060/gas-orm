use crate::FieldNames;
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) mod delete;
pub(crate) mod insert;

fn make_params_insert(
    vec_ident: Ident,
    fields: &[&(String, FieldNames)],
) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|(field_path, _)| {
            let ident = Ident::new(field_path, Span::call_site());

            quote! {
                #vec_ident.push(PgParam::from(self.#ident.clone()));
            }
        })
        .collect()
}
