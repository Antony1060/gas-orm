use crate::models::person;
use crate::tracing_util::setup_tracing;
use gas::connection::PgConnection;
use gas::eq::PgEq;
use gas::types::dec;
use gas::{GasResult, ModelOps};

mod models;
mod tracing_util;

#[tokio::main]
async fn main() -> GasResult<()> {
    setup_tracing();

    let conn =
        PgConnection::new_connection_pool("postgres://postgres:strong_password@localhost/postgres")
            .await?;

    person::Model::create_table(&conn, true).await?;

    let mut new = person::Model {
        first_name: "Test".to_string(),
        last_name: "Test".to_string(),
        email: "test@test.com".to_string(),
        phone_number: None,
        bank_account_balance: dec!(0),
        ..person::default()
    };

    tracing_dbg!("before insert", new);

    new.insert(&conn).await?;

    tracing_dbg!("after insert", new);

    let persons = person::Model::query()
        // .filter(|| {
        //     (person::bank_account_balance.gte(6000) & person::phone_number.is_not_null())
        //         | (person::id.gte(18) & person::phone_number.is_null())
        // })
        .filter(|| person::first_name.eq("Test") & person::last_name.eq("Test"))
        .find_all(&conn)
        .await?;

    tracing_dbg!(persons);

    Ok(())
}
