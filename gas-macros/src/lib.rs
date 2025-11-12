use darling::FromDeriveInput;
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
    Ok(quote! { PgType::__to_pg_type::<#path>() })
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(__gas_meta))]
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

    let inner = move || {
        let pg_type_tokens = proc_type_to_pg_type(&ty)?;

        let mut flags: Vec<proc_macro2::TokenStream> = Vec::new();

        flags.push(quote! { ((FieldFlags::Nullable as u8) * <#ty as IsOptional>::FACTOR) });

        if ctx.primary_keys.contains(&ident) {
            flags.push(quote! { (FieldFlags::PrimaryKey as u8) })
        }

        if ctx.serials.contains(&ident) {
            flags.push(quote! { (FieldFlags::Serial as u8) })
        }

        let table_name = ctx.table_name;

        Ok(quote! {
            pub const #ident: Field<#ty> = Field::new(concat!(#table_name, ".", stringify!(#ident)), #pg_type_tokens, #(#flags)|*, None);
        })
    };

    Some(inner())
}

#[inline(always)]
fn model_impl(args: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let args: proc_macro2::TokenStream = args.into();
    let input = syn::parse::<syn::ItemStruct>(input)?;
    let mod_identifier = Ident::new(&input.ident.to_string().to_lowercase(), Span::call_site());

    let mut original_struct = input.clone();
    original_struct.ident = Ident::new("Model", Span::call_site());

    Ok(quote! {
        pub mod #mod_identifier {
            #![allow(non_upper_case_globals, dead_code)]
            use super::*;
            use gas::{Field, FieldMeta, FieldFlags, ModelMeta, pg_type::*};

            #[derive(gas::__model)]
            #[__gas_meta(#args)]
            #original_struct
        }
    }
    .into())
}

#[inline(always)]
fn derive_model_impl(_input: TokenStream) -> Result<TokenStream, syn::Error> {
    // TODO: there's probably a better way to do this without double parse
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

    Ok(quote! {
        #(#field_consts)*

        const __FIELDS: &'static [FieldMeta] = &[#(#field_list.meta),*];

        impl ModelMeta for Model {
            #[inline(always)]
            fn table_name() -> &'static str {
                #table_name
            }
        }
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
