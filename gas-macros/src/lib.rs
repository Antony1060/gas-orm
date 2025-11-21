mod ops;
mod text_util;

use crate::ops::delete::gen_delete_sql_fn_tokens;
use crate::ops::insert::gen_insert_sql_fn_tokens;
use crate::ops::update::gen_update_sql_fn_tokens;
use crate::ops::update_with_fields::gen_update_with_fields_sql_fn_tokens;
use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Field, Fields, Index};

// this file is a mess lol

#[derive(Debug)]
struct FieldNames {
    column_name: String, // username
    full_name: String,   // users.username
    // alias is used in select queries so distinguishing columns on joined tables is easier
    alias_name: String, // users_username
}

struct ModelCtx<'a> {
    table_name: &'a str,
    primary_keys: &'a [Ident],
    serials: &'a [Ident],
    uniques: &'a [Ident],

    // field.ident -> names
    field_columns: &'a [(String, FieldNames)],
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

#[derive(Debug, FromMeta)]
struct DefaultArgs {
    #[darling(rename = "fn")]
    expression: syn::Expr,
}

#[inline(always)]
fn proc_type_to_pg_type(ty: &syn::Type) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Type::Path(path) = ty else {
        Err(syn::Error::new(ty.span(), "type must be a path type"))?
    };

    Ok(quote! { <#path as gas::internals::AsPgType>::PG_TYPE })
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

    // I love O(n)
    let field_names = ctx
        .field_columns
        .iter()
        .find(|(it, _)| it == &ident.to_string())
        .map(|(_, v)| v)?;

    let pg_type_tokens = proc_type_to_pg_type(&ty);
    let pg_type_tokens = match pg_type_tokens {
        Ok(tokens) => tokens,
        Err(err) => return Some(Err(err)),
    };

    let mut flags: Vec<proc_macro2::TokenStream> = Vec::new();

    flags.push(
        quote! { ((gas::FieldFlag::Nullable as u8) * <#ty as gas::internals::IsOptional>::FACTOR) },
    );

    if ctx.primary_keys.contains(ident) {
        flags.push(quote! { (gas::FieldFlag::PrimaryKey as u8) })
    }

    if ctx.serials.contains(ident) {
        flags.push(quote! { (gas::FieldFlag::Serial as u8) })
    }

    if ctx.uniques.contains(ident) {
        flags.push(quote! { (gas::FieldFlag::Unique as u8) })
    }

    let full_name = &field_names.full_name;
    let name = &field_names.column_name;
    let alias_name = &field_names.alias_name;

    Some(Ok(quote! {
        pub const #ident: gas::Field<#ty, Model> = gas::Field::new(gas::FieldMeta {
            full_name: #full_name,
            name: #name,
            alias_name: #alias_name,
            struct_name: stringify!(#ident),
            pg_type: #pg_type_tokens,
            flags: gas::FieldFlags(#(#flags)|*),
            relationship: None
        });
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
) -> Result<Vec<(String, FieldNames)>, syn::Error> {
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
        .collect::<Result<Vec<_>, _>>()
}

fn gen_key_tokens(ctx: &ModelCtx, fields: &Fields) -> proc_macro2::TokenStream {
    let primary_key_fields = fields.iter().filter_map(|field| {
        let ident = field.ident.as_ref()?;
        ctx.primary_keys
            .iter()
            .find(|pk| *pk == ident)
            .map(|_| field)
    });

    let primary_key_field_types = primary_key_fields.clone().map(|field| field.ty.clone());
    let primary_key_field_idents = primary_key_fields.clone().map(|field| field.ident.as_ref());

    let pk_count = primary_key_fields.count();

    let (apply_fn, condition_fn) = if pk_count <= 1 {
        let field_idents = primary_key_field_idents.clone();

        (
            quote! {
                #(self.#field_idents = key;)*
            },
            quote! {
                #(#primary_key_field_idents.eq(key))*
            },
        )
    } else {
        let mut counter = 0..;

        let (assignments, eqs): (Vec<_>, Vec<_>) = primary_key_field_idents
            .clone()
            .map(|ident| {
                let index = Index::from(counter.next().unwrap_or(0));

                (
                    quote! {
                        self.#ident = key.#index;
                    },
                    quote! {
                        #ident.eq(key.#index)
                    },
                )
            })
            .collect();

        (
            quote! {
                #(#assignments)*
            },
            quote! {
                #(#eqs)&*
            },
        )
    };

    quote! {
        type Key = (#(#primary_key_field_types),*);

        fn apply_key(&mut self, key: Self::Key) {
            #apply_fn
        }

        fn filter_with_key(key: Self::Key) -> gas::condition::EqExpression {
            use gas::eq::PgEq;

            #condition_fn
        }
    }
}

#[inline(always)]
fn derive_model_impl(_input: TokenStream) -> Result<TokenStream, syn::Error> {
    let derive_input = syn::parse::<syn::DeriveInput>(_input.clone())?;
    let input = syn::parse::<syn::ItemStruct>(_input)?;

    let meta: ModelArgs = FromDeriveInput::from_derive_input(&derive_input)?;

    let primary_keys = find_fields_with_attr(&input.fields, "primary_key");
    let serials = find_fields_with_attr(&input.fields, "serial");
    let uniques = find_fields_with_attr(&input.fields, "unique");

    let table_name = meta.table_name;

    let ctx = ModelCtx {
        table_name: &table_name,
        primary_keys: &primary_keys,
        serials: &serials,
        uniques: &uniques,
        field_columns: &parse_col_names(&table_name, &input.fields)?,
    };

    let field_consts = input
        .fields
        .iter()
        .filter_map(|field| process_field(&ctx, field))
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let field_list = input.fields.iter().filter_map(|field| field.ident.as_ref());
    let filed_list_get_by_field = field_list.clone();

    let key_tokens = gen_key_tokens(&ctx, &input.fields);

    let insert_fn = gen_insert_sql_fn_tokens(&ctx)?;
    let update_fn = gen_update_sql_fn_tokens(&ctx)?;
    let update_with_fields_fn = gen_update_with_fields_sql_fn_tokens(&ctx)?;
    let delete_fn = gen_delete_sql_fn_tokens(&ctx)?;

    let from_row_impl = generate_from_row(&ctx)?;

    Ok(quote! {
        #(#field_consts)*

        impl gas::ModelMeta for Model {
            const TABLE_NAME: &'static str = #table_name;
            const FIELDS: &'static [&'static gas::FieldMeta] = &[#(&#field_list.meta),*];

            #key_tokens

            fn gen_insert_sql(&self) -> gas::internals::SqlStatement {
                #insert_fn
            }

            fn gen_update_sql(&self) -> gas::internals::SqlStatement {
                #update_fn
            }

            fn gen_update_with_fields_sql(&self, fields: &[&gas::FieldMeta]) -> gas::internals::SqlStatement {
                #update_with_fields_fn
            }

            fn gen_delete_sql(&self) -> gas::internals::SqlStatement {
                #delete_fn
            }

            fn get_by_field<T: gas::internals::AsPgType + 'static>(&self, field: &gas::FieldMeta) -> Option<T> {
                let struct_name = field.struct_name;
                let type_id_t = std::any::TypeId::of::<T>();
                match struct_name {
                    #(stringify!(#filed_list_get_by_field) => {
                        let value = &self.#filed_list_get_by_field;
                        if type_id_t != gas::internals::type_id_of_value(value) {
                            return None;
                        }
                        Some(unsafe { (*((value as *const _) as *const T)).clone() })
                    },)*
                    _ => None
                }
            }
        }

        const _: () = {
            assert!(<Model as gas::ModelMeta>::FIELDS.len() > 0, "struct must not be empty");
        };

        #from_row_impl
    }
        .into())
}

fn gen_default_impl(fields: &Fields) -> Result<proc_macro2::TokenStream, syn::Error> {
    let fields = fields
        .iter()
        .filter_map(|field| {
            let ident = field.ident.as_ref()?;
            let ty = field.ty.clone();

            let attribute = field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("default"))
                .map(|attr| <DefaultArgs as FromMeta>::from_meta(&attr.meta));

            let Some(attribute) = attribute else {
                return Some(Ok(quote! {
                    #ident: <#ty as Default>::default()
                }));
            };

            let expr = match attribute {
                Ok(DefaultArgs { expression }) => expression,
                Err(err) => return Some(Err(err.into())),
            };

            Some(Ok::<proc_macro2::TokenStream, syn::Error>(quote! {
                #ident: #expr
            }))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        impl Default for Model {
            fn default() -> Self {
                Self {
                    #(#fields),*
                }
            }
        }
    })
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

    let default_impl_tokens = gen_default_impl(&input.fields)?;

    Ok(quote! {
        pub mod #mod_identifier {
            #![allow(non_upper_case_globals, dead_code)]
            use super::*;

            #[derive(gas::__model)]
            #[__gas_meta(#args_tokens)]
            #original_struct

            #default_impl_tokens

            pub fn default() -> Model {
                Default::default()
            }

            #[allow(unused_macros)]
            macro_rules! Def {
                ($($field:ident: $value:expr,)*) => {
                    gas::internals::DefModel::new(
                        #mod_identifier::Model {
                            $($field: $value,)*
                            ..#mod_identifier::Model::default()
                        },
                        Box::new([$(stringify!($field)),*])
                    )
                }
            }

            pub(crate) use Def;
        }
    }
    .into())
}

#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream {
    model_impl(args, input).unwrap_or_else(|err| err.to_compile_error().into())
}

#[proc_macro_derive(
    __model,
    attributes(primary_key, serial, unique, default, column, __gas_meta)
)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    derive_model_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}
