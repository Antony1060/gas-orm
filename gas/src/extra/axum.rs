use crate::connection::{PgConnection, PgTransaction};
use crate::error::GasError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;

pub struct Transaction(pub PgTransaction);

impl<S> FromRequestParts<S> for Transaction
where
    S: Send + Sync,
{
    type Rejection = GasError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let connection = parts
            .extensions
            .get::<PgConnection>()
            .ok_or_else(|| GasError::AxumNoConnectionExtensionSet)?;

        connection.transaction().await.map(Transaction)
    }
}

impl IntoResponse for GasError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

pub fn extension(connection: &PgConnection) -> Extension<PgConnection> {
    Extension(connection.clone())
}
