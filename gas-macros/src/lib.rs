use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Field, Fields};

fn proc_type_to_pg_type(ty: &syn::Type) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Type::Path(path) = ty else {
        Err(syn::Error::new(ty.span(), "type must be a path type"))?
    };

    // going through a generic function gives better errors compared to just `#path::PG_TYPE`
    Ok(quote! {PgType::__to_pg_type::<#path>()})
}

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ModelArgs {
    table_name: String,
}

fn find_fields_with_attr(fields: &Fields, target_attr: &'static str) -> Vec<Ident> {
    fields
        .iter()
        .cloned()
        .filter_map(|field| {
            field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident(target_attr))
                .and_then(|_| field.ident.clone())
        })
        .collect()
}

struct ModelCtx<'a> {
    primary_keys: &'a [Ident],
    serials: &'a [Ident],
}

fn process_field(
    ctx: &ModelCtx<'_>,
    field: &Field,
) -> Option<Result<proc_macro2::TokenStream, syn::Error>> {
    let ident = field.ident.clone()?;
    let ty = field.ty.clone();

    let inner = move || {
        let pg_type_tokens = proc_type_to_pg_type(&ty)?;

        let mut flags: Vec<proc_macro2::TokenStream> = Vec::new();
        if ctx.primary_keys.contains(&ident) {
            flags.push(quote! {(FieldFlags::PrimaryKey as u8)})
        }

        if ctx.serials.contains(&ident) {
            flags.push(quote! {(FieldFlags::Serial as u8)})
        }

        if flags.is_empty() {
            flags.push(quote! {0});
        }

        Ok(
            quote!(pub const #ident: Field<#ty> = Field::new(stringify!(#ident), #pg_type_tokens, #(#flags)|*, None);),
        )
    };

    Some(inner())
}

#[inline(always)]
fn model_impl(args: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let args = syn::parse::<ModelArgs>(args)?;
    let input = syn::parse::<syn::ItemStruct>(input)?;
    let mod_identifier = Ident::new(&input.ident.to_string().to_lowercase(), Span::call_site());

    let mut original_struct = input.clone();
    original_struct.ident = Ident::new("Model", Span::call_site());

    let table_name = args.table_name;

    Ok(quote! {
        pub mod #mod_identifier {
            #![allow(non_upper_case_globals, dead_code)]
            use super::*;
            use gas::{Field, FieldFlags, ModelOps, pg_type::*};

            #[derive(gas::__model)]
            #original_struct

            impl ModelOps for Model {
                #[inline(always)]
                fn table_name() -> &'static str {
                    #table_name
                }
            }
        }
    }
    .into())
}

#[inline(always)]
fn derive_model_impl(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let input = syn::parse::<syn::ItemStruct>(input)?;

    let primary_keys = find_fields_with_attr(&input.fields, "primary_key");
    let serials = find_fields_with_attr(&input.fields, "serial");

    let ctx = ModelCtx {
        primary_keys: &primary_keys,
        serials: &serials,
    };

    let fields = input
        .fields
        .iter()
        .filter_map(|field| process_field(&ctx, field))
        .collect::<Result<Vec<_>, syn::Error>>()?;

    Ok(quote! {
        #(#fields)*
    }
    .into())
}

#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream {
    model_impl(args, input).unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_derive(__model, attributes(primary_key, serial))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    derive_model_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}
