use crate::ops::make_params_insert;
use crate::{FieldNames, ModelCtx};
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) fn gen_insert_sql_fn_tokens(
    ctx: &ModelCtx,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let fields = ctx.field_columns.iter().filter(|(field_name, _)| {
        !ctx.serials
            .iter()
            .map(|it| it.to_string())
            .any(|it| *field_name == it)
    });

    let field_count = fields.clone().count();

    let field_full_list: Option<String> = fields
        .clone()
        .map(|(_, FieldNames { column_name, .. })| column_name.to_string())
        .reduce(|acc, curr| format!("{}, {}", acc, curr));

    let field_qs: Option<String> = fields
        .clone()
        .map(|_| "?".to_string())
        .reduce(|acc, curr| format!("{}, {}", acc, curr));

    let all_returning = ctx
        .field_columns
        .iter()
        .map(
            |(
                _,
                FieldNames {
                    full_name,
                    alias_name,
                    ..
                },
            )| format!("{} AS {}", full_name, alias_name),
        )
        .reduce(|acc, curr| format!("{}, {}", acc, curr));

    let field_params = make_params_insert(
        Ident::new("params", Span::call_site()),
        &fields.collect::<Vec<_>>(),
    );

    Ok(quote! {
        fn gen_insert_sql(&self) -> gas::sql_query::SqlStatement {
            use gas::sql_query::SqlQuery;
            use gas::pg_param::PgParam;

            let mut sql = SqlQuery::new("INSERT INTO ");
            sql.append_str(Self::TABLE_NAME);
            sql.append_str(concat!("(", #field_full_list, ")"));
            sql.append_str(" VALUES ");
            sql.append_str(concat!("(", #field_qs, ")"));
            sql.append_str(concat!(" RETURNING ", #all_returning));

            let mut params: Vec<PgParam> = Vec::with_capacity(#field_count);
            #(#field_params)*

            (sql, std::sync::Arc::from(params.as_ref()))
        }
    })
}
