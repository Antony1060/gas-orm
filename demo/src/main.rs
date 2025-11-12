use crate::models::thing;
use gas::connection::{GasResult, PgConnection};
use gas::eq::PgEq;
use gas::ModelOps;

mod models;

#[tokio::main]
async fn main() -> GasResult<()> {
    let conn =
        PgConnection::new_connection_pool("postgres://postgres:strong_password@localhost/postgres")
            .await?;

    let things = thing::Model::query()
        .filter(|| thing::id.eq(10))
        .find_all(&conn)
        .await?;

    dbg!(&things);

    Ok(())
}
