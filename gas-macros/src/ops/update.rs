use crate::ops::{make_all_returning, make_params};
use crate::{FieldNames, ModelCtx};
use proc_macro2::Span;
use quote::quote;

// TODO: fields with only primary keys will error here
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

    if normal_fields.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "The struct is only made from ",
        ));
    }

    let where_statement: Option<String> = pk_fields
        .iter()
        .map(|(_, FieldNames { column_name, .. })| format!("{}=?", column_name))
        .reduce(|acc, curr| format!("{} AND {}", acc, curr));

    let where_statement: String = where_statement.unwrap_or_else(|| "1=1".to_string());

    let set_statement: Option<String> = normal_fields
        .iter()
        .map(|(_, FieldNames { column_name, .. })| format!("{}=?", column_name))
        .reduce(|acc, curr| format!("{}, {}", acc, curr));

    let all_returning = make_all_returning(ctx);

    let field_params = make_params(
        &normal_fields
            .into_iter()
            .chain(pk_fields)
            .collect::<Vec<_>>(),
    );

    let table_name = ctx.table_name;

    Ok(quote! {
        use gas::internals::SqlQuery;
        use gas::internals::PgParam;

        let mut sql = SqlQuery::from(concat!(
            "UPDATE ", #table_name,
            " SET ", #set_statement,
            " WHERE ", #where_statement,
            " RETURNING ", #all_returning
        ));

        (sql, std::boxed::Box::new([#(#field_params),*]))
    })
}
