use crate::error::GasError;
use crate::{GasResult, ModelMeta};

trait OnlyOption<T> {
    type Value;
}

impl<T> OnlyOption<T> for Option<T> {
    type Value = T;
}

pub trait OptionHelperOps<M: ModelMeta>: OnlyOption<M> {
    fn res(self) -> GasResult<M>;
}

impl<M: ModelMeta> OptionHelperOps<M> for Option<M> {
    fn res(self) -> GasResult<M> {
        match self {
            Some(m) => Ok(m),
            None => Err(GasError::EntityNotFound),
        }
    }
}
