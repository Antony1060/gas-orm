use crate::connection::PgConnection;
use crate::error::GasError;
use http::{Request, Response};
use std::fmt::{Display, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

pub fn layer(connection: &PgConnection) -> GasLayer {
    GasLayer {
        connection: connection.clone(),
        rollback_on_non_success: true,
    }
}

#[derive(Clone)]
pub struct GasLayer {
    connection: PgConnection,
    rollback_on_non_success: bool,
}

impl GasLayer {
    pub fn rollback_on_non_success(mut self, value: bool) -> Self {
        self.rollback_on_non_success = value;

        self
    }
}

impl<S> Layer<S> for GasLayer {
    type Service = GasTowerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GasTowerService {
            inner,
            connection: self.connection.clone(),
            rollback_on_non_success: self.rollback_on_non_success,
        }
    }
}

#[derive(Clone)]
pub struct GasTowerService<S> {
    inner: S,
    connection: PgConnection,
    rollback_on_non_success: bool,
}

#[derive(thiserror::Error)]
pub enum GasServiceError<E> {
    GasError(#[from] GasError),
    Other(E),
}

impl<E> Display for GasServiceError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GasServiceError::GasError(err) => write!(f, "{}", err),
            GasServiceError::Other(_) => write!(f, "unknown"),
        }
    }
}

async fn handle_call<S, ReqBody, ResBody>(
    mut inner: S,
    mut req: Request<ReqBody>,
    connection: PgConnection,
    rollback_on_non_success: bool,
) -> Result<S::Response, GasServiceError<S::Error>>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    let tx = connection
        .transaction()
        .await
        .map_err(GasServiceError::GasError)?;

    req.extensions_mut().insert(tx.clone());
    req.extensions_mut().insert(connection.clone());

    let response = inner.call(req).await.map_err(GasServiceError::Other)?;

    if rollback_on_non_success && !response.status().is_success() {
        tx.discard().await.map_err(GasServiceError::GasError)?;
    } else {
        tx.save().await.map_err(GasServiceError::GasError)?;
    }

    Ok(response)
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for GasTowerService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Response: Send,
    S::Error: Send,
    S::Future: Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(ctx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let inner = self.inner.clone();

        let rollback_on_non_success = self.rollback_on_non_success;
        let connection = self.connection.clone();

        Box::pin(async move {
            let response = handle_call(inner, req, connection, rollback_on_non_success).await;

            match response {
                Ok(val) => Ok(val),
                // handling errors here seems to be an unsolved computer science problem
                //  internet contains no resources on this (or maybe google is just ass)
                Err(GasServiceError::GasError(err)) => panic!("unexpected orm error: {}", err),
                Err(GasServiceError::Other(err)) => Err(err),
            }
        })
    }
}
