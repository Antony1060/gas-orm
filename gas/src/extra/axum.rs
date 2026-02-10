use crate::connection::{PgConnection, PgTransaction};
use crate::error::GasError;
use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};
use http::request::Parts;
use http::StatusCode;

pub struct Connection(pub PgConnection);

pub struct Transaction(pub PgTransaction);

impl<S> FromRequestParts<S> for Connection
where
    S: Send + Sync,
{
    type Rejection = GasError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let connection = parts
            .extensions
            .get::<PgConnection>()
            .ok_or_else(|| GasError::AxumMissingExtension)?;

        Ok(Connection(connection.clone()))
    }
}

impl<S> FromRequestParts<S> for Transaction
where
    S: Send + Sync,
{
    type Rejection = GasError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tx = parts
            .extensions
            .get::<PgTransaction>()
            .ok_or_else(|| GasError::AxumMissingExtension)?;

        Ok(Transaction(tx.clone()))
    }
}

impl IntoResponse for GasError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
