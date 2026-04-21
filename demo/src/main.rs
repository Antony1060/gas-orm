use crate::error::DemoResult;
use crate::tracing_util::setup_tracing;
use gas::connection::PgConnection;
use gas::load_migrations;
use gas::migrations::{MigrateCount, MigrateDirection};
use std::env;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod error;
mod models;
mod routes;
mod tracing_util;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::authors::list,
        routes::authors::get_one,
        routes::authors::create,
        routes::authors::update,
        routes::authors::delete,
        routes::books::list,
        routes::books::get_one,
        routes::books::get_detail,
        routes::books::create,
        routes::books::update,
        routes::books::delete,
        routes::categories::list,
        routes::categories::get_one,
        routes::categories::create,
        routes::categories::update,
        routes::categories::delete,
        routes::reviews::list,
        routes::reviews::list_by_book,
        routes::reviews::get_one,
        routes::reviews::create,
        routes::reviews::update,
        routes::reviews::delete,
    ),
    components(schemas(
        routes::authors::CreateAuthorRequest,
        routes::authors::UpdateAuthorRequest,
        routes::books::CreateBookRequest,
        routes::books::UpdateBookRequest,
        routes::categories::CreateCategoryRequest,
        routes::categories::UpdateCategoryRequest,
        routes::reviews::CreateReviewRequest,
        routes::reviews::UpdateReviewRequest,
    )),
    info(
        title = "Bookstore API",
        description = "A demo REST API built with Axum and the Gas ORM",
        version = "1.0.0"
    )
)]
struct ApiDoc;

async fn init_db() -> DemoResult<PgConnection> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:strong_password@localhost/orm_test".to_string());

    let conn = PgConnection::new_connection_pool(&database_url).await?;

    let migrator = load_migrations!("./migrations")?;
    migrator
        .run_migrations(&conn, MigrateDirection::Forward, MigrateCount::All)
        .await?;

    Ok(conn)
}

#[tokio::main]
async fn main() -> DemoResult<()> {
    setup_tracing(env::var("TRACE_ORM").map(|_| true).unwrap_or(false));

    let db_connection = init_db().await?;

    let app = routes::api()
        .layer(gas::extra::tower::layer(&db_connection))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http());

    let port = env::var("PORT")
        .unwrap_or_else(|_| String::from("3000"))
        .parse::<u16>()
        .expect("PORT must be a number");

    let bind_address = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    tracing::info!("Server listening on {bind_address}");
    tracing::info!("Swagger UI at http://{bind_address}/swagger-ui");

    axum::serve(listener, app).await?;

    Ok(())
}
