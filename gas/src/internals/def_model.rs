use crate::connection::PgExecutionContext;
use crate::ops::update::UpdateOp;
use crate::{GasResult, ModelMeta};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

pub struct DefModel<T: ModelMeta> {
    model: T,
    pub(crate) modified_fields: Box<[&'static str]>,
}

impl<T: ModelMeta> DefModel<T> {
    pub fn new(model: T, modified_fields: Box<[&'static str]>) -> Self {
        Self {
            model,
            modified_fields,
        }
    }

    pub fn update_by_key<E: PgExecutionContext>(
        mut self,
        ctx: &E,
        key: T::Key,
    ) -> impl Future<Output = GasResult<T>> {
        async {
            self.apply_key(key);

            // this way of updating is slightly more inefficient because
            //  it can't generate the sql at compile time
            UpdateOp::<T>::new(&mut self.model)
                .run_with_fields(ctx, &self.modified_fields)
                .await?;

            Ok(self.into_model())
        }
    }

    pub fn into_model(self) -> T {
        self.model
    }
}

impl<T: ModelMeta> Deref for DefModel<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl<T: ModelMeta> DerefMut for DefModel<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}

impl<T: ModelMeta + Debug> Debug for DefModel<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.model.fmt(f)
    }
}
