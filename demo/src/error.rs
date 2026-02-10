use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use gas::error::GasError;
use std::borrow::Cow;

pub type DemoResult<T> = Result<T, DemoError>;

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("404: not found")]
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum DemoError {
    #[error("http: {0}")]
    Http(#[from] HttpError),

    #[error("Database error: {0}")]
    Database(#[from] GasError),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(serde::Serialize)]
struct ErrorMessage {
    error: Cow<'static, str>,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            HttpError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorMessage {
                    error: Cow::Borrowed("Not found"),
                }),
            ),
        };

        (status, message).into_response()
    }
}

impl IntoResponse for DemoError {
    fn into_response(self) -> Response {
        match self {
            DemoError::Http(err) => err.into_response(),
            err => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage {
                    error: Cow::from(format!("Internal server error: {err}")),
                }),
            )
                .into_response(),
        }
    }
}
