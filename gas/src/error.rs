use crate::internals::PgParam;
use std::borrow::Cow;

#[derive(thiserror::Error, Debug)]
pub enum GasError {
    #[error("driver failed: {0}")]
    DriverError(#[from] sqlx::Error),

    #[error("failed to convert parameter: {0}")]
    TypeError(PgParam),

    #[error("invalid query format")]
    QueryFormatError,

    #[error("unexpected response: {0}")]
    UnexpectedResponse(Cow<'static, str>),

    // "everything is checked at compile time" they said
    //  "if it compiles, it will work", they said
    #[error("invalid input to ORM: {0}")]
    InvalidInput(&'static str),

    #[error("query yielded no responses: {0}")]
    QueryNoResponse(&'static str),

    #[error("relation wasn't defined correctly")]
    InvalidRelation,
}
