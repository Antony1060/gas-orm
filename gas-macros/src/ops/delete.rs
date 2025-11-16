use crate::ops::make_params;
use crate::{FieldNames, ModelCtx};
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

    let where_statement: Option<String> = pk_fields
        .clone()
        .map(|(_, FieldNames { column_name, .. })| format!("{}=?", column_name))
        .reduce(|acc, curr| format!("{} AND {}", acc, curr));

    let field_params = make_params(&pk_fields.collect::<Vec<_>>());

    let table_name = ctx.table_name;

    Ok(quote! {
        use gas::internals::SqlQuery;
        use gas::internals::PgParam;

        let mut sql = SqlQuery::from(concat!(
            "DELETE FROM ",
            #table_name,
            " WHERE ",
            #where_statement
        ));

        (sql, std::boxed::Box::new([#(#field_params),*]))
    })
}
