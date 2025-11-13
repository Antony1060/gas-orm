mod text_util;

use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{Field, Fields};

#[derive(Debug)]
struct FieldNames {
    column_name: String, // username
    full_name: String,   // users.username
    // alias is used in select queries so distinguishing columns on joined tables is easier
    alias_name: String, // users_username
}

struct ModelCtx<'a> {
    primary_keys: &'a [Ident],
    serials: &'a [Ident],

    // field.ident -> names
    field_columns: HashMap<String, FieldNames>,
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

#[derive(Debug, FromMeta)]
struct ColumnArgs {
    name: String,
}

#[inline(always)]
fn proc_type_to_pg_type(ty: &syn::Type) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Type::Path(path) = ty else {
        Err(syn::Error::new(ty.span(), "type must be a path type"))?
    };

    // going through a generic function gives better errors compared to just `#path::PG_TYPE`
    Ok(quote! { gas::pg_type::PgType::__to_pg_type::<#path>() })
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

fn process_field(
    ctx: &ModelCtx<'_>,
    field: &Field,
) -> Option<Result<proc_macro2::TokenStream, syn::Error>> {
    let ident = field.ident.as_ref()?;
    let ty = field.ty.clone();

    let field_names = ctx.field_columns.get(&ident.to_string())?;

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

    let full_name = &field_names.full_name;
    let name = &field_names.column_name;
    let alias_name = &field_names.alias_name;

    Some(Ok(quote! {
        pub const #ident: gas::Field<#ty> = gas::Field::new(
            #full_name, #name, #alias_name,
            #pg_type_tokens, #(#flags)|*, None
        );
    }))
}

fn generate_from_row(ctx: &ModelCtx) -> Result<proc_macro2::TokenStream, syn::Error> {
    let field_defs = ctx
        .field_columns
        .iter()
        .map(|(ident, FieldNames { alias_name, .. })| {
            let ident = Ident::new(ident, Span::call_site());

            quote! {
                #ident: row.try_get(#alias_name)?,
            }
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

fn get_col_name(field: &Field) -> Option<Result<String, syn::Error>> {
    let attribute = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("column"));

    let Some(attr) = attribute else {
        return field.ident.as_ref().map(|it| Ok(it.to_string()));
    };

    let column_args: Result<ColumnArgs, _> = FromMeta::from_meta(&attr.meta);
    match column_args {
        Ok(ColumnArgs { name }) => Some(Ok(name)),
        Err(err) => Some(Err(err.into())),
    }
}

fn parse_col_names(
    table_name: &str,
    fields: &Fields,
) -> Result<HashMap<String, FieldNames>, syn::Error> {
    fields
        .iter()
        .filter_map(|field| {
            // None if field has nor attribute nor name
            //  Result otherwise, will report errors of invalid usage of column attribute
            let col_name = get_col_name(field)?;
            let col_name = match col_name {
                Ok(name) => name,
                Err(err) => return Some(Err(err)),
            };

            Some(Ok((
                field.ident.as_ref()?.to_string(),
                FieldNames {
                    full_name: format!("{}.{}", table_name, col_name),
                    alias_name: format!("{}_{}", table_name, col_name),
                    column_name: col_name,
                },
            )))
        })
        .collect::<Result<HashMap<_, _>, _>>()
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
        primary_keys: &primary_keys,
        serials: &serials,
        field_columns: parse_col_names(&table_name, &input.fields)?,
    };

    let field_consts = input
        .fields
        .iter()
        .filter_map(|field| process_field(&ctx, field))
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let field_list = input.fields.iter().filter_map(|field| field.ident.clone());

    let from_row_impl = generate_from_row(&ctx)?;

    Ok(quote! {
        #(#field_consts)*

        impl gas::ModelMeta for Model {
            const TABLE_NAME: &'static str = #table_name;
            const FIELDS: &'static [gas::FieldMeta] = &[#(#field_list.meta),*];
        }

        const _: () = {
            assert!(<Model as gas::ModelMeta>::FIELDS.len() > 1, "struct must not be empty");
        };

        #from_row_impl
    }
    .into())
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

#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream {
    model_impl(args, input).unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_derive(__model, attributes(primary_key, serial, column, __gas_meta))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    derive_model_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}
