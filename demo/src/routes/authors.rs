use crate::error::{DemoResult, HttpError};
use crate::models::author;
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use gas::extra::axum::Transaction;
use gas::ModelOps;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateAuthorRequest {
    pub name: String,
    pub bio: String,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateAuthorRequest {
    pub name: Option<String>,
    pub bio: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/authors", get(list).post(create))
        .route("/api/authors/{id}", get(get_one).put(update).delete(delete))
}

#[utoipa::path(
    get,
    path = "/api/authors",
    tag = "Authors",
    responses(
        (status = 200, description = "List all authors", body = Vec<author::Model>)
    )
)]
async fn list(Transaction(mut tx): Transaction) -> DemoResult<Json<Vec<author::Model>>> {
    let authors = author::Model::query()
        .sort(author::id.asc())
        .find_all(&mut tx)
        .await?;

    Ok(Json(authors))
}

#[utoipa::path(
    get,
    path = "/api/authors/{id}",
    tag = "Authors",
    params(("id" = i64, Path, description = "Author ID")),
    responses(
        (status = 200, description = "Author found", body = author::Model),
        (status = 404, description = "Author not found")
    )
)]
async fn get_one(
    Transaction(mut tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<Json<author::Model>> {
    let author = author::Model::find_by_key(&mut tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    Ok(Json(author))
}

#[utoipa::path(
    post,
    path = "/api/authors",
    tag = "Authors",
    request_body = CreateAuthorRequest,
    responses(
        (status = 201, description = "Author created", body = author::Model)
    )
)]
async fn create(
    Transaction(mut tx): Transaction,
    Json(req): Json<CreateAuthorRequest>,
) -> DemoResult<(axum::http::StatusCode, Json<author::Model>)> {
    let mut new_author = author::Def! {
        name: req.name,
        bio: req.bio,
    };

    new_author.insert(&mut tx).await?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(new_author.into_model()),
    ))
}

#[utoipa::path(
    put,
    path = "/api/authors/{id}",
    tag = "Authors",
    params(("id" = i64, Path, description = "Author ID")),
    request_body = UpdateAuthorRequest,
    responses(
        (status = 200, description = "Author updated", body = author::Model),
        (status = 404, description = "Author not found")
    )
)]
async fn update(
    Transaction(mut tx): Transaction,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAuthorRequest>,
) -> DemoResult<Json<author::Model>> {
    let mut model = author::Model::find_by_key(&mut tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    if let Some(name) = req.name {
        model.name = name;
    }

    if let Some(bio) = req.bio {
        model.bio = bio;
    }

    model.update(&mut tx).await?;

    Ok(Json(model))
}

#[utoipa::path(
    delete,
    path = "/api/authors/{id}",
    tag = "Authors",
    params(("id" = i64, Path, description = "Author ID")),
    responses(
        (status = 204, description = "Author deleted"),
        (status = 404, description = "Author not found")
    )
)]
async fn delete(
    Transaction(mut tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<axum::http::StatusCode> {
    let model = author::Model::find_by_key(&mut tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    model.delete(&mut tx).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
