use crate::error::GasError;
use crate::{GasResult, ModelMeta};

trait OnlyOption<T> {}

impl<T> OnlyOption<T> for Option<T> {}
impl<T> OnlyOption<T> for Option<&T> {}

pub trait OptionHelperOps<M: ModelMeta>: OnlyOption<M> {
    type Output;

    fn res(self) -> GasResult<Self::Output>;
}

impl<M: ModelMeta> OptionHelperOps<M> for Option<M> {
    type Output = M;

    fn res(self) -> GasResult<M> {
        match self {
            Some(m) => Ok(m),
            None => Err(GasError::EntityNotFound),
        }
    }
}

impl<'a, M: ModelMeta> OptionHelperOps<M> for Option<&'a M>
where
    M: 'a,
{
    type Output = &'a M;

    fn res(self) -> GasResult<&'a M> {
        match self {
            Some(m) => Ok(m),
            None => Err(GasError::EntityNotFound),
        }
    }
}
