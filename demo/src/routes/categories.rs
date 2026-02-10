use crate::error::{DemoResult, HttpError};
use crate::models::category;
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use gas::extra::axum::Transaction;
use gas::ModelOps;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: String,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/categories", get(list).post(create))
        .route(
            "/api/categories/{id}",
            get(get_one).put(update).delete(delete),
        )
}

#[utoipa::path(
    get,
    path = "/api/categories",
    tag = "Categories",
    responses(
        (status = 200, description = "List all categories", body = Vec<category::Model>)
    )
)]
async fn list(Transaction(tx): Transaction) -> DemoResult<Json<Vec<category::Model>>> {
    let categories = category::Model::query()
        .sort(category::id.asc())
        .find_all(&tx)
        .await?;

    Ok(Json(categories))
}

#[utoipa::path(
    get,
    path = "/api/categories/{id}",
    tag = "Categories",
    params(("id" = i64, Path, description = "Category ID")),
    responses(
        (status = 200, description = "Category found", body = category::Model),
        (status = 404, description = "Category not found")
    )
)]
async fn get_one(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<Json<category::Model>> {
    let category = category::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    Ok(Json(category))
}

#[utoipa::path(
    post,
    path = "/api/categories",
    tag = "Categories",
    request_body = CreateCategoryRequest,
    responses(
        (status = 201, description = "Category created", body = category::Model)
    )
)]
async fn create(
    Transaction(tx): Transaction,
    Json(req): Json<CreateCategoryRequest>,
) -> DemoResult<(axum::http::StatusCode, Json<category::Model>)> {
    let mut model = category::Def! {
        name: req.name,
        description: req.description,
    };

    model.insert(&tx).await?;

    Ok((axum::http::StatusCode::CREATED, Json(model.into_model())))
}

#[utoipa::path(
    put,
    path = "/api/categories/{id}",
    tag = "Categories",
    params(("id" = i64, Path, description = "Category ID")),
    request_body = UpdateCategoryRequest,
    responses(
        (status = 200, description = "Category updated", body = category::Model),
        (status = 404, description = "Category not found")
    )
)]
async fn update(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
    Json(req): Json<UpdateCategoryRequest>,
) -> DemoResult<Json<category::Model>> {
    let mut model = category::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    if let Some(name) = req.name {
        model.name = name;
    }

    if let Some(desc) = req.description {
        model.description = desc;
    }

    model.update(&tx).await?;

    Ok(Json(model))
}

#[utoipa::path(
    delete,
    path = "/api/categories/{id}",
    tag = "Categories",
    params(("id" = i64, Path, description = "Category ID")),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 404, description = "Category not found")
    )
)]
async fn delete(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<axum::http::StatusCode> {
    let model = category::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    model.delete(&tx).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
