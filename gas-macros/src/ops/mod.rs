use crate::{FieldNames, ModelCtx};
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) mod delete;
pub(crate) mod insert;
pub(crate) mod update;

fn make_params(fields: &[&(String, FieldNames)]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|(field_path, _)| {
            let ident = Ident::new(field_path, Span::call_site());

            quote! {
                PgParam::from(self.#ident.clone())
            }
        })
        .collect()
}

fn make_all_returning(ctx: &ModelCtx) -> Option<String> {
    ctx.field_columns
        .iter()
        .map(
            |(
                _,
                FieldNames {
                    full_name,
                    alias_name,
                    ..
                },
            )| format!("{} AS {}", full_name, alias_name),
        )
        .reduce(|acc, curr| format!("{}, {}", acc, curr))
}
