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

    person::Model::create_table(&conn, true).await?;

    normal_ops(&conn).await?;
    tracing::info!("----------------");
    by_key_ops(&conn).await?;
    tracing::info!("----------------");
    transaction_ops(&conn).await?;

    Ok(())
}

async fn transaction_ops(conn: &PgConnection) -> GasResult<()> {
    let tx = conn.transaction().await?;

    let mut person = person::Def! {
        first_name: String::from("Some"),
        last_name: String::from("Person"),
        email: String::from("some@person.com"),
    };

    person.phone_number = Some(String::from("091"));
    person.bank_account_balance = Decimal::from(2000);
    person.update(&tx).await?;

    tracing_dbg!(person);

    Ok(())
}

async fn by_key_ops(conn: &PgConnection) -> GasResult<()> {
    person::Def! {
        phone_number: Some(format!("+385{}", rand::random_range(100000000u64..999999999u64))),
        bank_account_balance: Decimal::from(rand::random_range(100u64..100000u64)),
    }
    .update_by_key(conn, 20)
    .await?;

    // person::Model::delete_by_key(&conn, 4).await?;

    let id_ten = person::Model::find_by_key(conn, 20).await?;
    tracing_dbg!(id_ten);

    Ok(())
}

async fn normal_ops(conn: &PgConnection) -> GasResult<()> {
    let mut new_person = person::Def! {
        first_name: String::from("Test"),
        last_name: String::from("Test"),
        email: String::from("nonce"),
    }
    .into_model();

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
