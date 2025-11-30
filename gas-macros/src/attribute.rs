use crate::text_util;
use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_quote, Field, Fields};

// both derive and attribute macro arguments can't be derived from the same struct (I think)
#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ModelArgs {
    #[allow(dead_code)]
    table_name: String,
    mod_name: Option<String>,
}

#[derive(Debug, FromMeta)]
struct DefaultArgs {
    #[darling(rename = "fn")]
    expression: syn::Expr,
}

#[derive(Debug, FromMeta)]
struct RelationArgs {
    field: Option<syn::Path>,
    inverse: Option<syn::Path>,
}

#[inline(always)]
pub(crate) fn model_impl(args: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    let args_tokens: proc_macro2::TokenStream = args.clone().into();
    let input = syn::parse::<syn::ItemStruct>(input)?;

    let args: ModelArgs = syn::parse::<ModelArgs>(args)?;

    let mod_identifier_name = match args.mod_name {
        Some(ident) => ident,
        None => text_util::pascal_to_snake_case(&input.ident.to_string()),
    };

    let mod_identifier = Ident::new(&mod_identifier_name, Span::call_site());

    let mut original_struct = input.clone();
    original_struct.ident = Ident::new("Model", Span::call_site());
    apply_relation_type_changes(&mut original_struct)?;

    let default_impl_tokens = gen_default_impl(&original_struct.fields)?;

    Ok(quote! {
        pub mod #mod_identifier {
            #![allow(non_upper_case_globals, dead_code)]
            use super::*;

            #[derive(gas::__model, Clone)]
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

fn apply_forward_relation(field: &mut Field, path: syn::Path) -> Result<(), syn::Error> {
    let ty = &field.ty;
    let meta_path = {
        let mut meta_path = path.clone();
        let last = meta_path.segments.last_mut();
        let Some(last) = last else {
            return Err(syn::Error::new(path.span(), "invalid path"));
        };

        last.ident = Ident::new(&format!("{}_meta", last.ident), Span::call_site());

        meta_path
    };

    // this yields some very very very ugly errors, but hey,
    //  at least it won't compile if incorrect
    field.ty = parse_quote! { <#ty as gas::RelationConverter>::ToFull<{
        // figure out a way for this without cycles
        gas::internals::assert_type::<<#ty as gas::RelationConverter>::ToField>(&#path);

        assert!(
            #meta_path.flags.has_flag(gas::FieldFlag::Unique) ||
                (#meta_path.flags.has_flag(gas::FieldFlag::PrimaryKey) &&
                    !#meta_path.flags.has_flag(gas::FieldFlag::CompositePrimaryKey)),
            "relation must point to a field that is unique or a single primary key"
        );

        #meta_path.index
    }> };
    Ok(())
}

fn apply_inverse_relation(field: &mut Field, _path: syn::Path) -> Result<(), syn::Error> {
    // TODO:
    field.ty = parse_quote! { i64 };

    Ok(())
}

fn apply_relation_type_changes(target: &mut syn::ItemStruct) -> Result<(), syn::Error> {
    let fields = target.fields.iter_mut().filter_map(|field| {
        let attr = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("relation"))?
            .clone();

        Some((field, attr))
    });

    for (field, relation_attr) in fields {
        let args: RelationArgs = FromMeta::from_meta(&relation_attr.meta)?;

        if args.field.is_some() && args.inverse.is_some() {
            Err(syn::Error::new(
                field.span(),
                "relation must be either field or inverse",
            ))?
        }

        if let Some(path) = args.field {
            apply_forward_relation(field, path)?;
            continue;
        }

        if let Some(path) = args.inverse {
            apply_inverse_relation(field, path)?;
            continue;
        }

        Err(syn::Error::new(
            field.span(),
            "missing field: `field` or `inverse`",
        ))?
    }

    Ok(())
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
