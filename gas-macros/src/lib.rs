mod attribute;
mod derive;
mod ops;
mod text_util;

use proc_macro::TokenStream;
use proc_macro2::Ident;

#[derive(Debug)]
struct FieldNames {
    column_name: String, // username
    full_name: String,   // users.username
    // alias is used in select queries so distinguishing columns on joined tables is easier
    alias_name: String, // users_username
}

struct ModelCtx<'a> {
    virtuals: &'a [Ident],

    // all the other fields assume that they're derived from non-virtual fields
    table_name: &'a str,
    primary_keys: &'a [Ident],
    serials: &'a [Ident],
    uniques: &'a [Ident],

    // field.ident -> names
    field_columns: &'a [(String, FieldNames)],
}

#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream {
    attribute::model_impl(args, input).unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_derive(
    __model,
    attributes(
        primary_key,
        serial,
        unique,
        default,
        column,
        relation,
        __gas_virtual,
        __gas_meta
    )
)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    derive::model_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}
