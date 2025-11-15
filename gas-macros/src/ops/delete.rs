use crate::ops::make_params_insert;
use crate::{FieldNames, ModelCtx};
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) fn gen_delete_sql_fn_tokens(
    ctx: &ModelCtx,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let pk_fields = ctx.field_columns.iter().filter(|(field_name, _)| {
        ctx.primary_keys
            .iter()
            .map(|it| it.to_string())
            .any(|it| *field_name == it)
    });

    let field_count = pk_fields.clone().count();

    let where_statement: Option<String> = pk_fields
        .clone()
        .map(|(_, FieldNames { column_name, .. })| format!("{}=?", column_name))
        .reduce(|acc, curr| format!("{} AND {}", acc, curr));

    let field_params = make_params_insert(
        Ident::new("params", Span::call_site()),
        &pk_fields.collect::<Vec<_>>(),
    );

    let table_name = ctx.table_name;

    Ok(quote! {
        use gas::sql_query::SqlQuery;
        use gas::pg_param::PgParam;

        let mut sql = SqlQuery::new(concat!(
            "DELETE FROM ",
            #table_name,
            " WHERE ",
            #where_statement
        ));
        let mut params: Vec<PgParam> = Vec::with_capacity(#field_count);
        #(#field_params)*

        (sql, std::sync::Arc::from(params.as_ref()))
    })
}
