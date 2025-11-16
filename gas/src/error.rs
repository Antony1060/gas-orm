use crate::internals::PgParam;

#[derive(thiserror::Error, Debug)]
pub enum GasError {
    #[error("driver failed: {0}")]
    DriverError(#[from] sqlx::Error),

    #[error("failed to convert parameter: {0}")]
    TypeError(PgParam),

    #[error("invalid query format")]
    QueryFormatError,

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("unexpected state {0}")]
    UnexpectedState(&'static str),
}
