pub mod authors;
pub mod books;
pub mod categories;
pub mod reviews;

use axum::Router;

pub fn api() -> Router {
    Router::new()
        .merge(authors::router())
        .merge(books::router())
        .merge(categories::router())
        .merge(reviews::router())
}

