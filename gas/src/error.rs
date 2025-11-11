#[derive(thiserror::Error, Debug)]
pub enum GasError {
    #[error("driver failed: {0}")]
    DriverError(#[from] sqlx::Error),

    #[error("invalid query format")]
    QueryFormatError,
}
