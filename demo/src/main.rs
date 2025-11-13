use crate::models::person;
use gas::connection::PgConnection;
use gas::{GasResult, ModelOps};

mod models;

#[tokio::main]
async fn main() -> GasResult<()> {
    let conn =
        PgConnection::new_connection_pool("postgres://postgres:strong_password@localhost/postgres")
            .await?;

    person::Model::create_table(&conn, true).await?;

    let students = person::Model::query().find_all(&conn).await?;

    dbg!(&students);

    Ok(())
}
