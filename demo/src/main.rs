use crate::models::person;
use crate::tracing_util::setup_tracing;
use gas::connection::PgConnection;
use gas::eq::PgEq;
use gas::{GasResult, ModelOps};
use rust_decimal::Decimal;
use std::env;

mod models;
mod tracing_util;

#[tokio::main]
async fn main() -> GasResult<()> {
    setup_tracing(env::var("TRACE_ORM").map(|_| true).unwrap_or(false));

    let conn =
        PgConnection::new_connection_pool("postgres://postgres:strong_password@localhost/postgres")
            .await?;

    main2(&conn).await?;

    person::Model::create_table(&conn, true).await?;

    person::Def! {
        phone_number: Some(format!("+385{}", rand::random_range(100000000u64..999999999u64))),
    }
    .update_by_key(&conn, 2)
    .await?;

    person::Model::delete_by_key(&conn, 4).await?;

    let id_ten = person::Model::find_by_key(&conn, 4).await?;
    tracing_dbg!(id_ten);

    Ok(())
}

async fn main2(conn: &PgConnection) -> GasResult<()> {
    person::Model::create_table(conn, true).await?;

    let mut new_person = person::Def! {
        first_name: String::from("Test"),
        last_name: String::from("Test"),
        email: String::from("nonce"),
    }
    .as_model();

    tracing_dbg!("before insert", new_person);
    tracing_dbg!(
        person::Model::query()
            .filter(|| person::email.eq("nonce"))
            .find_one(conn)
            .await?
    );

    new_person.insert(conn).await?;

    tracing_dbg!("after insert", new_person);
    tracing_dbg!(
        person::Model::query()
            .filter(|| person::email.eq("nonce") & person::id.eq(new_person.id))
            .find_one(conn)
            .await?
    );

    new_person.last_name = String::from("Doe");
    new_person.bank_account_balance = Decimal::from(2000);
    new_person.update(conn).await?;

    tracing_dbg!(
        "after update",
        person::Model::query()
            .filter(|| person::email.eq("nonce") & person::id.eq(new_person.id))
            .find_one(conn)
            .await?
    );

    let persons = person::Model::query()
        .filter(|| person::email.eq("nonce"))
        .find_all(conn)
        .await?;

    tracing_dbg!(persons);
    new_person.delete(conn).await?;

    Ok(())
}
