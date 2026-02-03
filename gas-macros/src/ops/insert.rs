use crate::ops::{make_all_returning, make_params};
use crate::{FieldNames, ModelCtx};
use quote::quote;

// NOTE: structs with only serial fields will error here
//  might as well pull a DHH and say it's opinionated
pub(crate) fn gen_insert_sql_fn_tokens(
    ctx: &ModelCtx,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let fields = ctx.field_columns.iter().filter(|(field_name, _)| {
        !ctx.serials
            .iter()
            .map(|it| it.to_string())
            .any(|it| *field_name == it)
    });

    let field_full_list: Option<String> = fields
        .clone()
        .map(|(_, FieldNames { column_name, .. })| column_name.to_string())
        .reduce(|acc, curr| format!("{}, {}", acc, curr));

    let field_qs: Option<String> = fields
        .clone()
        .map(|_| "?".to_string())
        .reduce(|acc, curr| format!("{}, {}", acc, curr));

    let all_returning = make_all_returning(ctx);

    let field_params = make_params(&fields.collect::<Vec<_>>());

    let table_name = ctx.table_name;

    Ok(quote! {
        use gas::internals::SqlQuery;
        use gas::internals::PgParam;

        let mut sql = SqlQuery::from(concat!(
            "INSERT INTO ", #table_name, "(", #field_full_list, ")",
            " VALUES ", "(", #field_qs, ")",
            " RETURNING ", #all_returning
        ));

        (sql, std::boxed::Box::new([#(#field_params),*]))
    })
}
