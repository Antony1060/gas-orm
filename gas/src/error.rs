use crate::PgParams;

#[derive(thiserror::Error, Debug)]
pub enum GasError {
    #[error("driver failed: {0}")]
    DriverError(#[from] sqlx::Error),

    #[error("failed to convert parameter: {0}")]
    TypeError(PgParams),

    #[error("invalid query format")]
    QueryFormatError,
}
