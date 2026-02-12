use crate::error::{DemoResult, HttpError};
use crate::models::{author, book};
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use gas::eq::PgEq;
use gas::extra::axum::Transaction;
use gas::{FullRelation, ModelOps};

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreateBookRequest {
    pub title: String,
    pub isbn: String,
    pub published_year: i32,
    pub page_count: i32,
    pub author_id: i64,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateBookRequest {
    pub title: Option<String>,
    pub isbn: Option<String>,
    pub published_year: Option<i32>,
    pub page_count: Option<i32>,
    pub author_id: Option<i64>,
}

pub fn router() -> Router {
    Router::new()
        .route("/api/books", get(list).post(create))
        .route("/api/books/{id}", get(get_one).put(update).delete(delete))
        .route("/api/books/{id}/detail", get(get_detail))
}

#[utoipa::path(
    get,
    path = "/api/books",
    tag = "Books",
    operation_id = "list_books",
    responses(
        // (status = 200, description = "List all books", body = Vec<book::Model>)
    )
)]
async fn list(Transaction(tx): Transaction) -> DemoResult<Json<Vec<book::Model>>> {
    let books = book::Model::query()
        .sort(book::id.asc())
        .find_all(&tx)
        .await?;

    Ok(Json(books))
}

#[utoipa::path(
    get,
    path = "/api/books/{id}",
    tag = "Books",
    operation_id = "get_book",
    params(("id" = i64, Path, description = "Book ID")),
    responses(
        // (status = 200, description = "Book found", body = book::Model),
        (status = 404, description = "Book not found")
    )
)]
async fn get_one(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<Json<book::Model>> {
    let model = book::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    Ok(Json(model))
}

#[utoipa::path(
    get,
    path = "/api/books/{id}/detail",
    tag = "Books",
    operation_id = "get_book_detail",
    params(("id" = i64, Path, description = "Book ID")),
    responses(
        // (status = 200, description = "Book with author detail", body = book::Model),
        (status = 404, description = "Book not found")
    )
)]
async fn get_detail(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<Json<book::Model>> {
    let book = book::Model::query()
        .include(book::author)
        .include(author::address)
        .filter(|| book::id.eq(id))
        .find_one(&tx)
        .await?
        .ok_or(HttpError::NotFound)?;

    Ok(Json(book))
}

#[utoipa::path(
    post,
    path = "/api/books",
    tag = "Books",
    operation_id = "create_book",
    request_body = CreateBookRequest,
    responses(
        // (status = 201, description = "Book created", body = book::Model)
    )
)]
async fn create(
    Transaction(tx): Transaction,
    Json(req): Json<CreateBookRequest>,
) -> DemoResult<(axum::http::StatusCode, Json<book::Model>)> {
    author::Model::find_by_key(&tx, req.author_id)
        .await?
        .ok_or(HttpError::NotFound)?;

    let mut model = book::Def! {
        title: req.title,
        isbn: req.isbn,
        published_year: req.published_year,
        page_count: req.page_count,
        author: FullRelation::ForeignKey(req.author_id),
    };

    model.insert(&tx).await?;

    Ok((axum::http::StatusCode::CREATED, Json(model.into_model())))
}

#[utoipa::path(
    put,
    path = "/api/books/{id}",
    tag = "Books",
    operation_id = "update_book",
    params(("id" = i64, Path, description = "Book ID")),
    request_body = UpdateBookRequest,
    responses(
        // (status = 200, description = "Book updated", body = book::Model),
        (status = 404, description = "Book not found")
    )
)]
async fn update(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
    Json(req): Json<UpdateBookRequest>,
) -> DemoResult<Json<book::Model>> {
    let mut model = book::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    if let Some(title) = req.title {
        model.title = title;
    }

    if let Some(isbn) = req.isbn {
        model.isbn = isbn;
    }

    if let Some(year) = req.published_year {
        model.published_year = year;
    }

    if let Some(page_count) = req.page_count {
        model.page_count = page_count;
    }

    if let Some(author_id) = req.author_id {
        let author = author::Model::find_by_key(&tx, author_id)
            .await?
            .ok_or(HttpError::NotFound)?;

        model.author = FullRelation::Loaded(author);
    }

    model.update(&tx).await?;

    Ok(Json(model))
}

#[utoipa::path(
    delete,
    path = "/api/books/{id}",
    tag = "Books",
    operation_id = "delete_book",
    params(("id" = i64, Path, description = "Book ID")),
    responses(
        (status = 204, description = "Book deleted"),
        (status = 404, description = "Book not found")
    )
)]
async fn delete(
    Transaction(tx): Transaction,
    Path(id): Path<i64>,
) -> DemoResult<axum::http::StatusCode> {
    let model = book::Model::find_by_key(&tx, id)
        .await?
        .ok_or(HttpError::NotFound)?;

    model.delete(&tx).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
