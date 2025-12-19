use crate::ops::delete::gen_delete_sql_fn_tokens;
use crate::ops::insert::gen_insert_sql_fn_tokens;
use crate::ops::update::gen_update_sql_fn_tokens;
use crate::ops::update_with_fields::gen_update_with_fields_sql_fn_tokens;
use crate::{FieldNames, ModelCtx};
use darling::{FromDeriveInput, FromMeta};
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Field, Index, LitStr, Meta, MetaList};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(__gas_meta))]
struct ModelArgs {
    table_name: String,

    #[allow(dead_code)]
    mod_name: Option<String>,
}

#[derive(Debug, FromMeta)]
struct ColumnArgs {
    name: String,
}

#[inline(always)]
pub fn model_impl(_input: TokenStream) -> Result<TokenStream, syn::Error> {
    let derive_input = syn::parse::<syn::DeriveInput>(_input.clone())?;
    let input = syn::parse::<syn::ItemStruct>(_input)?;

    let meta: ModelArgs = FromDeriveInput::from_derive_input(&derive_input)?;

    // TODO (low priority): extract these magic string constants to some module
    let virtuals =
        find_fields_with_attr(&input.fields.iter().cloned().collect_vec(), "__gas_virtual");
    let real_fields = input
        .fields
        .iter()
        .filter(|&field| {
            field
                .ident
                .as_ref()
                .map(|ident| !virtuals.contains(ident))
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();

    let primary_keys = find_fields_with_attr(&real_fields, "primary_key");
    let serials = find_fields_with_attr(&real_fields, "serial");
    let uniques = find_fields_with_attr(&real_fields, "unique");

    let table_name = meta.table_name;

    let ctx = ModelCtx {
        virtuals: &virtuals,
        table_name: &table_name,
        primary_keys: &primary_keys,
        serials: &serials,
        uniques: &uniques,
        foreign_keys: &parse_foreign_keys(&real_fields),
        field_columns: &parse_col_names(&table_name, &real_fields)?,
    };

    let mut counter = 0usize..;

    let (field_consts, field_metas) = input
        .fields
        .iter()
        .filter_map(|field| process_field(&ctx, field, counter.next().unwrap()))
        .collect::<Result<(Vec<_>, Vec<_>), syn::Error>>()?;

    let field_list = real_fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .collect::<Vec<_>>();
    let field_list_len = field_list.len();

    let key_tokens = gen_key_tokens(&ctx, &real_fields);

    let insert_fn = gen_insert_sql_fn_tokens(&ctx)?;
    let update_fn = gen_update_sql_fn_tokens(&ctx)?;
    let update_with_fields_fn = gen_update_with_fields_sql_fn_tokens(&ctx)?;
    let delete_fn = gen_delete_sql_fn_tokens(&ctx)?;

    let from_row_impl = generate_from_row(&ctx)?;

    let link_section_name: LitStr = LitStr::new(
        &format!("__gas_internals,__{}", table_name),
        Span::call_site(),
    );

    Ok(quote! {
        #(#field_consts)*

        impl gas::ModelMeta for Model {
            type Id = __::Inner;

            const TABLE_NAME: &'static str = #table_name;
            const FIELDS: &'static [gas::FieldMeta] = &[#(#field_list.meta),*];
            const VIRTUAL_FIELDS: &'static [gas::FieldMeta] = &[#(#virtuals.meta),*];

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
                    #(stringify!(#field_list) => {
                        let value = &self.#field_list;
                        if type_id_t != gas::internals::type_id_of_value(value) {
                            return None;
                        }
                        Some(unsafe { (*((value as *const _) as *const T)).clone() })
                    },)*
                    _ => None
                }
            }
        }

        pub mod __ {
            use super::*;

            // helper struct that comes "together" with Model, this allows limiting things like
            //  Field to a specific model and also avoiding cyclic type checking
            #[doc(hidden)]
            pub struct Inner;

            #[unsafe(link_section = #link_section_name)]
            #[used]
            static FIELDS: [([u8; 64], gas::FieldMeta); #field_list_len] = [#(({
                let mut val = [0; 64];
                unsafe { std::ptr::copy_nonoverlapping(#field_list.meta.name.as_bytes().as_ptr(), val.as_mut_ptr(), #field_list.meta.name.as_bytes().len()); }

                val
            }, #field_list.meta)),*];

            impl gas::ModelSidecar for Inner {}

            #(#field_metas)*
        }

        const _: () = {
            assert!(<Model as gas::ModelMeta>::FIELDS.len() > 0, "struct must not be empty");
        };

        #from_row_impl
    }
        .into())
}

fn find_fields_with_attr(fields: &[Field], target_attr: &'static str) -> Vec<Ident> {
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

fn parse_foreign_keys(fields: &[Field]) -> Vec<(Ident, syn::Type)> {
    fields
        .iter()
        .cloned()
        .filter_map(|field| {
            field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("__gas_foreign_key"))
                .and_then(|attr| {
                    let Meta::List(MetaList { tokens, .. }) = &attr.meta else {
                        return None;
                    };
                    Some((field.ident.clone()?, syn::parse_quote! { #tokens }))
                })
        })
        .collect()
}

fn gen_key_tokens(ctx: &ModelCtx, fields: &[Field]) -> proc_macro2::TokenStream {
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

fn generate_from_row(ctx: &ModelCtx) -> Result<proc_macro2::TokenStream, syn::Error> {
    let field_defs = ctx
        .field_columns
        .iter()
        .map(|(ident, FieldNames { alias_name, .. })| {
            let ident = Ident::new(ident, Span::call_site());

            quote! {
                #ident: gas::row::FromRowNamed::from_row_named(ctx, row, #alias_name)?,
            }
        });

    let virtual_defs = ctx.virtuals.iter().map(|ident| {
        quote! {
            #ident: gas::row::FromRowNamed::from_row_named(ctx, row, stringify!(#ident))?,
        }
    });

    Ok(quote! {
        impl gas::row::FromRow for Model {
            fn from_row(ctx: &gas::row::ResponseCtx, row: &gas::row::Row) -> gas::GasResult<Model> {
                Ok(Self {
                    #(#field_defs)*
                    #(#virtual_defs)*
                })
            }
        }
    })
}

fn parse_col_names(
    table_name: &str,
    fields: &[Field],
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

#[inline(always)]
fn proc_type_to_pg_type(ty: &syn::Type) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Type::Path(path) = ty else {
        Err(syn::Error::new(ty.span(), "type must be a path type"))?
    };

    Ok(quote! { <#path as gas::internals::AsPgType>::PG_TYPE })
}

fn process_field(
    ctx: &ModelCtx<'_>,
    field: &Field,
    index: usize,
) -> Option<Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), syn::Error>> {
    let ident = field.ident.as_ref()?;
    let ty = field.ty.clone();

    let virtual_names = FieldNames {
        column_name: ident.to_string(),
        full_name: format!("virtual.{}", ident),
        alias_name: format!("virtual_{}", ident),
    };
    let field_names = ctx
        .field_columns
        .iter()
        .find(|(it, _)| it == &ident.to_string())
        .map(|(_, v)| v)
        .unwrap_or(&virtual_names);

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

    if ctx.primary_keys.len() > 1 {
        flags.push(quote! { (gas::FieldFlag::CompositePrimaryKey as u8) })
    }

    if ctx.serials.contains(ident) {
        flags.push(quote! { (gas::FieldFlag::Serial as u8) })
    }

    if ctx.uniques.contains(ident) {
        flags.push(quote! { (gas::FieldFlag::Unique as u8) })
    }

    let maybe_foreign_key = ctx.foreign_keys.iter().find(|(it, _)| it == ident);
    if maybe_foreign_key.is_some() {
        flags.push(quote! { (gas::FieldFlag::ForeignKey as u8) })
    }

    let is_virtual = ctx.virtuals.contains(ident);
    if is_virtual {
        flags.push(quote! { (gas::FieldFlag::Virtual as u8) })
    }

    let full_name = &field_names.full_name;
    let name = &field_names.column_name;
    let alias_name = &field_names.alias_name;

    let table_name = ctx.table_name;

    let ident_meta = Ident::new(&format!("{}_meta", ident), ident.span());
    let ident_index = Ident::new(&format!("{}_index", ident), ident.span());
    let ident_flags = Ident::new(&format!("{}_flags", ident), ident.span());
    let ident_fk_type_alias = Ident::new(&format!("{}_fk_type", ident), ident.span());
    let ident_fk_remote_index = Ident::new(&format!("{}_fk_remote_index", ident), ident.span());

    let fk_extra_vars = if let Some((_, fk_type)) = maybe_foreign_key {
        quote! {
            #[allow(non_camel_case_types)]
            pub type #ident_fk_type_alias = #fk_type;
            pub const #ident_fk_remote_index: usize = 0;
        }
    } else {
        quote! {}
    };

    Some(Ok((
        if !is_virtual {
            quote! {
                pub const #ident: gas::Field<#ty, __::Inner> = gas::Field::new(__::#ident_meta);
            }
        } else {
            let virtual_field_type = get_virtual_field_type()?;
            quote! {
                pub const #ident: gas::VirtualField<__::Inner> = gas::VirtualField::new(#virtual_field_type, __::#ident_meta);
            }
        },
        quote! {
            #fk_extra_vars
            pub const #ident_index: usize = #index;
            pub const #ident_flags: gas::FieldFlags = gas::FieldFlags(#(#flags)|*);
            pub const #ident_meta: gas::FieldMeta = gas::FieldMeta {
                table_name: #table_name,
                full_name: #full_name,
                name: #name,
                alias_name: #alias_name,
                struct_name: stringify!(#ident),
                pg_type: #pg_type_tokens,
                flags: #ident_flags,
                index: #ident_index,
            };
        },
    )))
}

fn get_virtual_field_type() -> Option<proc_macro2::TokenStream> {
    // currently the only one supported, the function is not that useful lol
    Some(quote! {
        gas::VirtualFieldType::InverseRelation
    })
}
