mod text_util;

use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Field, Fields};

#[inline(always)]
fn proc_type_to_pg_type(ty: &syn::Type) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Type::Path(path) = ty else {
        Err(syn::Error::new(ty.span(), "type must be a path type"))?
    };

    // going through a generic function gives better errors compared to just `#path::PG_TYPE`
    Ok(quote! { gas::pg_type::PgType::__to_pg_type::<#path>() })
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(__gas_meta))]
struct ModelArgs {
    table_name: String,

    #[allow(dead_code)]
    mod_name: Option<String>,
}

// both derive and attribute macro arguments can't be derived from the same struct (I think)
#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ModelArgsAttrib {
    #[allow(dead_code)]
    table_name: String,
    mod_name: Option<String>,
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
    table_name: &'a str,
    primary_keys: &'a [Ident],
    serials: &'a [Ident],
}

fn process_field(
    ctx: &ModelCtx<'_>,
    field: &Field,
) -> Option<Result<proc_macro2::TokenStream, syn::Error>> {
    let ident = field.ident.clone()?;
    let ty = field.ty.clone();

    let pg_type_tokens = proc_type_to_pg_type(&ty);
    let pg_type_tokens = match pg_type_tokens {
        Ok(tokens) => tokens,
        Err(err) => return Some(Err(err)),
    };

    let mut flags: Vec<proc_macro2::TokenStream> = Vec::new();

    flags.push(
        quote! { ((gas::FieldFlags::Nullable as u8) * <#ty as gas::pg_type::IsOptional>::FACTOR) },
    );

    if ctx.primary_keys.contains(&ident) {
        flags.push(quote! { (gas::FieldFlags::PrimaryKey as u8) })
    }

    if ctx.serials.contains(&ident) {
        flags.push(quote! { (gas::FieldFlags::Serial as u8) })
    }

    let table_name = ctx.table_name;

    Some(Ok(quote! {
        pub const #ident: gas::Field<#ty> = gas::Field::new(concat!(#table_name, ".", stringify!(#ident)), #pg_type_tokens, #(#flags)|*, None);
    }))
}

#[inline(always)]
fn model_impl(args: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let args_tokens: proc_macro2::TokenStream = args.clone().into();
    let input = syn::parse::<syn::ItemStruct>(input)?;

    let args: ModelArgsAttrib = syn::parse::<ModelArgsAttrib>(args)?;

    let mod_identifier_name = match args.mod_name {
        Some(ident) => ident,
        None => text_util::pascal_to_snake_case(&input.ident.to_string()),
    };

    let mod_identifier = Ident::new(&mod_identifier_name, Span::call_site());

    let mut original_struct = input.clone();
    original_struct.ident = Ident::new("Model", Span::call_site());

    Ok(quote! {
        pub mod #mod_identifier {
            #![allow(non_upper_case_globals, dead_code)]
            use super::*;

            #[derive(gas::__model)]
            #[__gas_meta(#args_tokens)]
            #original_struct
        }
    }
    .into())
}

fn generate_from_row(fields: &Fields) -> Result<proc_macro2::TokenStream, syn::Error> {
    // TODO: move to column name
    let mut idx = 0usize;

    let field_defs = fields
        .iter()
        .filter_map(|field| {
            let ident = field.ident.clone()?;

            idx += 1;

            Some(quote! {
                #ident: row.try_get(#idx - 1)?,
            })
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        impl gas::row::FromRow for Model {
            fn from_row(row: &gas::row::Row) -> gas::GasResult<Model> {
                Ok(Self {
                    #(#field_defs)*
                })
            }
        }
    })
}

#[inline(always)]
fn derive_model_impl(_input: TokenStream) -> Result<TokenStream, syn::Error> {
    // TODO(low priority): there's probably a better way to do this without double parse
    let derive_input = syn::parse::<syn::DeriveInput>(_input.clone())?;
    let input = syn::parse::<syn::ItemStruct>(_input)?;

    let meta: ModelArgs = FromDeriveInput::from_derive_input(&derive_input)?;

    let primary_keys = find_fields_with_attr(&input.fields, "primary_key");
    let serials = find_fields_with_attr(&input.fields, "serial");

    let table_name = meta.table_name;

    let ctx = ModelCtx {
        table_name: &table_name,
        primary_keys: &primary_keys,
        serials: &serials,
    };

    let field_consts = input
        .fields
        .iter()
        .filter_map(|field| process_field(&ctx, field))
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let field_list = input.fields.iter().filter_map(|field| field.ident.clone());

    let from_row_impl = generate_from_row(&input.fields)?;

    Ok(quote! {
        #(#field_consts)*

        const __FIELDS: &'static [gas::FieldMeta] = &[#(#field_list.meta),*];

        impl gas::ModelMeta for Model {
            #[inline(always)]
            fn table_name() -> &'static str {
                #table_name
            }
        }

        #from_row_impl
    }
    .into())
}

#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream {
    model_impl(args, input).unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_derive(__model, attributes(primary_key, serial, __gas_meta))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    derive_model_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}
