use crate::models::jaspersuser;
use gas::connection::{GasResult, PgConnection};
use gas::eq::PgEq;
use gas::ModelOps;

mod models;

#[tokio::main]
async fn main() -> GasResult<()> {
    let conn =
        PgConnection::new_connection_pool("postgres://postgres:strong_password@localhost/jaspers")
            .await?;

    let things = jaspersuser::Model::query()
        .filter(|| jaspersuser::id.lt(10))
        .find_all(&conn)
        .await?;

    dbg!(&things);

    Ok(())
}
