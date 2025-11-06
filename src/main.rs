use crate::model::user;
use crate::sql::{AsSql, ModelOps, PgEq};

mod builder;
mod condition;
mod eq;
mod model;
mod sql;

fn main() {
    let builder = user::Model::filter(|| {
        user::username.eq("John")
            & (user::email.eq("john.user.email") | user::password.eq("john.user.password"))
    });

    dbg!(&builder);

    println!("sql:\n{}", builder.as_sql());
}
