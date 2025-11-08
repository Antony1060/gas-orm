use crate::models::{student, user};
use gas::{eq::PgEq, AsSql, ModelOps};

mod models;

fn main() {
    {
        let select = user::Model::filter(|| {
            user::username.eq("John")
                & (user::email.eq("john.user.email") | user::bank_account_balance.gt(1000000i128))
        });

        dbg!(&select);
        println!("sql:\n{}", select.as_sql());
    }

    {
        let select = student::Model::filter(|| student::id.lte(100) & student::last_name.eq("Doe"));

        dbg!(&select);
        println!("sql:\n{}", select.as_sql());
    }
}
