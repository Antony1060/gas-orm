use crate::error::{DemoResult, HttpError};
use crate::models::{book, review};
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use gas::eq::PgEq;
use gas::extra::axum::Transaction;
use gas::{FullRelation, ModelOps};

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateReviewRequest {
    pub rating: i32,
    pub content: String,
    pub book_id: i64,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateReviewRequest {
    pub rating: Option<i32>,
    pub content: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/reviews", get(list).post(create))
        .route("/api/reviews/{id}", get(get_one).put(update).delete(delete))
        .route("/api/books/{book_id}/reviews", get(list_by_book))
}

#[utoipa::path(
    get,
    path = "/api/reviews",
    tag = "Reviews",
    responses(
        (status = 200, description = "List all reviews", body = Vec<review::Model>)
    )
)]
async fn list(Transaction(tx): Transaction) -> DemoResult<Json<Vec<review::Model>>> {
    let reviews = review::Model::query()
        .sort(review::id.asc())
        .find_all(&tx)
        .await?;

    Ok(Json(reviews))
}

/// List all reviews for a specific book.
#[utoipa::path(
    get,
    path = "/api/books/{book_id}/reviews",
    tag = "Reviews",
    params(("book_id" = i64, Path, description = "Book ID")),
    responses(
        (status = 200, description = "List reviews for book", body = Vec<review::Model>)
    )
)]
async fn list_by_book(
    Transaction(tx): Transaction,
    Path(book_id): Path<i64>,
) -> DemoResult<Json<Vec<review::Model>>> {
    let reviews = review::Model::query()
        .include(review::book)
        .filter(|| book::id.eq(book_id))
        .sort(review::id.asc())
        .find_all(&tx)
        .await?;

    Ok(Json(reviews))
}

#[utoipa::path(
    get,
    path = "/api/reviews/{id}",
    tag = "Reviews",
    params(("id" = i64, Path, description = "Review ID")),
    responses(
        (status = 200, description = "Review found", body = review::Model),
        (status = 404, description = "Review not found")
    )
)]
async fn get_one(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<Json<review::Model>> {
    let model = review::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    Ok(Json(model))
}

#[utoipa::path(
    post,
    path = "/api/reviews",
    tag = "Reviews",
    request_body = CreateReviewRequest,
    responses(
        (status = 201, description = "Review created", body = review::Model)
    )
)]
async fn create(
    Transaction(tx): Transaction,
    Json(req): Json<CreateReviewRequest>,
) -> DemoResult<(axum::http::StatusCode, Json<review::Model>)> {
    let book = book::Model::find_by_key(&tx, req.book_id)
        .await?
        .ok_or(HttpError::NotFound)?;

    let mut model = review::Def! {
        rating: req.rating,
        content: req.content,
        book: FullRelation::Loaded(book),
    };

    model.insert(&tx).await?;

    Ok((axum::http::StatusCode::CREATED, Json(model.into_model())))
}

#[utoipa::path(
    put,
    path = "/api/reviews/{id}",
    tag = "Reviews",
    params(("id" = i64, Path, description = "Review ID")),
    request_body = UpdateReviewRequest,
    responses(
        (status = 200, description = "Review updated", body = review::Model),
        (status = 404, description = "Review not found")
    )
)]
async fn update(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
    Json(req): Json<UpdateReviewRequest>,
) -> DemoResult<Json<review::Model>> {
    let mut model = review::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    if let Some(rating) = req.rating {
        model.rating = rating;
    }

    if let Some(content) = req.content {
        model.content = content;
    }

    model.update(&tx).await?;

    Ok(Json(model))
}

#[utoipa::path(
    delete,
    path = "/api/reviews/{id}",
    tag = "Reviews",
    params(("id" = i64, Path, description = "Review ID")),
    responses(
        (status = 204, description = "Review deleted"),
        (status = 404, description = "Review not found")
    )
)]
async fn delete(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<axum::http::StatusCode> {
    let model = review::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    model.delete(&tx).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
