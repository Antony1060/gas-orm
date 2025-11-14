use crate::models::person;
use gas::connection::PgConnection;
use gas::eq::PgEq;
use gas::types::dec;
use gas::{GasResult, ModelOps};

mod models;

#[tokio::main]
async fn main() -> GasResult<()> {
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

    dbg!(&new);

    new.insert(&conn).await?;

    dbg!(&new);

    let persons = person::Model::query()
        // .filter(|| {
        //     (person::bank_account_balance.gte(6000) & person::phone_number.is_not_null())
        //         | (person::id.gte(18) & person::phone_number.is_null())
        // })
        .filter(|| person::first_name.eq("Test") & person::last_name.eq("Test"))
        .find_all(&conn)
        .await?;

    dbg!(&persons);

    Ok(())
}
