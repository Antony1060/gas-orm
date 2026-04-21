use std::pin::Pin;

pub unsafe fn into_static<'from, Out>(
    future: impl Future<Output = Out> + Send + 'from,
) -> Pin<Box<dyn Future<Output = Out> + Send + 'static>> {
    unsafe {
        std::mem::transmute::<
            Pin<Box<dyn Future<Output = Out> + Send + 'from>>,
            Pin<Box<dyn Future<Output = Out> + Send + 'static>>,
        >(Box::pin(future))
    }
}
