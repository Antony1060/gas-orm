use crate::ops::make_all_returning;
use crate::{FieldNames, ModelCtx};
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) fn gen_update_with_fields_sql_fn_tokens(
    ctx: &ModelCtx,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let (pk_fields, normal_fields): (Vec<_>, Vec<_>) =
        ctx.field_columns.iter().partition(|(field_name, _)| {
            ctx.primary_keys
                .iter()
                .map(|it| it.to_string())
                .any(|it| *field_name == it)
        });

    let where_statement: Option<String> = pk_fields
        .iter()
        .map(|(_, FieldNames { column_name, .. })| format!("{}=?", column_name))
        .reduce(|acc, curr| format!("{} AND {}", acc, curr));

    let set_statement = quote! {
        gas::internals::generate_update_set_fields(fields)
    };

    let all_returning = make_all_returning(ctx);

    let field_params = normal_fields.into_iter().map(|(field_path, _)| {
        let ident = Ident::new(field_path, Span::call_site());

        quote! {
            if (fields.iter().any(|field| field.struct_name == #field_path)) {
                params.push(PgParam::from(self.#ident.clone()));
            }
        }
    });

    let pk_count = pk_fields.len();

    let pk_params = pk_fields.into_iter().map(|(field_path, _)| {
        let ident = Ident::new(field_path, Span::call_site());

        quote! {
            params.push(PgParam::from(self.#ident.clone()));
        }
    });

    let table_name = ctx.table_name;

    Ok(quote! {
        use gas::internals::SqlQuery;
        use gas::internals::PgParam;

        let mut sql = SqlQuery::from(concat!(
            "UPDATE ", #table_name, " SET "
        ));

        sql.append_str(&#set_statement);
        sql.append_str(concat!(
            " WHERE ", #where_statement,
            " RETURNING ", #all_returning
        ));

        let mut params: Vec<PgParam> = Vec::with_capacity(fields.len() + #pk_count);
        #(#field_params)*;
        #(#pk_params)*;

        (sql, params.into_boxed_slice())
    })
}
