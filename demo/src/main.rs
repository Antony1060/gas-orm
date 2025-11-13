use crate::models::student;
use gas::connection::PgConnection;
use gas::{GasResult, ModelOps};

mod models;

#[tokio::main]
async fn main() -> GasResult<()> {
    let conn =
        PgConnection::new_connection_pool("postgres://postgres:strong_password@localhost/postgres")
            .await?;

    let students = student::Model::query().find_all(&conn).await?;

    dbg!(&students);

    Ok(())
}
