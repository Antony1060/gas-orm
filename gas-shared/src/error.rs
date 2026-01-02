use std::borrow::Cow;

// eh, this makes api a little worse
#[derive(thiserror::Error, Debug)]
pub enum GasSharedError {
    #[error("internal ORM error: {0}")]
    InternalError(Cow<'static, str>),
}
