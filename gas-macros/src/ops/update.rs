use crate::ops::{make_all_returning, make_params_insert};
use crate::{FieldNames, ModelCtx};
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) fn gen_update_sql_fn_tokens(
    ctx: &ModelCtx,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let (pk_fields, normal_fields): (Vec<_>, Vec<_>) =
        ctx.field_columns.iter().partition(|(field_name, _)| {
            ctx.primary_keys
                .iter()
                .map(|it| it.to_string())
                .any(|it| *field_name == it)
        });

    let field_count = ctx.field_columns.len();

    let where_statement: Option<String> = pk_fields
        .iter()
        .map(|(_, FieldNames { column_name, .. })| format!("{}=?", column_name))
        .reduce(|acc, curr| format!("{} AND {}", acc, curr));

    let set_statement: Option<String> = normal_fields
        .iter()
        .map(|(_, FieldNames { column_name, .. })| format!("{}=?", column_name))
        .reduce(|acc, curr| format!("{}, {}", acc, curr));

    let all_returning = make_all_returning(ctx);

    let field_params = make_params_insert(
        Ident::new("params", Span::call_site()),
        &normal_fields
            .into_iter()
            .chain(pk_fields)
            .collect::<Vec<_>>(),
    );

    Ok(quote! {
        use gas::sql_query::SqlQuery;
        use gas::pg_param::PgParam;

        let mut sql = SqlQuery::new("UPDATE ");
        sql.append_str(Self::TABLE_NAME);
        sql.append_str(" SET ");
        sql.append_str(#set_statement);
        sql.append_str(" WHERE ");
        sql.append_str(#where_statement);
        sql.append_str(concat!(" RETURNING ", #all_returning));

        let mut params: Vec<PgParam> = Vec::with_capacity(#field_count);
        #(#field_params)*

        (sql, std::sync::Arc::from(params.as_ref()))
    })
}
