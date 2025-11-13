use crate::models::person;
use gas::connection::PgConnection;
use gas::eq::{PgEq, PgEqNone};
use gas::{GasResult, ModelOps};

mod models;

#[tokio::main]
async fn main() -> GasResult<()> {
    let conn =
        PgConnection::new_connection_pool("postgres://postgres:strong_password@localhost/postgres")
            .await?;

    person::Model::create_table(&conn, true).await?;

    let persons = person::Model::query()
        .filter(|| {
            (person::bank_account_balance.gte(6000) & person::phone_number.is_not_null())
                | (person::id.gte(18) & person::phone_number.is_null())
        })
        .find_all(&conn)
        .await?;

    dbg!(&persons);

    Ok(())
}
