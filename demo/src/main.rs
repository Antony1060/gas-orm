use crate::models::user;
use gas::{eq::PgEq, AsSql, ModelOps};

mod models;

fn main() {
    let builder = user::Model::filter(|| {
        user::username.eq("John")
            & (user::email.eq("john.user.email") | user::password.eq("john.user.password"))
    });

    dbg!(&builder);

    println!("sql:\n{}", builder.as_sql());
}
