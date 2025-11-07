use crate::models::{student, user};
use gas::{AsSql, ModelOps, eq::PgEq};

mod models;

fn main() {
    let builder = user::Model::filter(|| {
        user::username.eq("John")
            & (user::email.eq("john.user.email") | user::password.eq("john.user.password"))
    });

    let a: user::Model = user::Model {
        id: 0,
        username: "".to_string(),
        email: "".to_string(),
        password: "".to_string(),
        bank_account_balance: Default::default(),
    };

    dbg!(&builder);

    println!("sql:\n{}", builder.as_sql());
}
