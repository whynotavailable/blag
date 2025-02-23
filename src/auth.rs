use std::sync::{Arc, RwLock};

use axum::{extract::Request, response::Response};
use futures_util::future::BoxFuture;
use tower::{Layer, Service};
use tracing::info;

#[derive(Clone)]
pub struct AuthLayer;

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            l: Arc::new(RwLock::new(0)),
        }
    }
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    l: Arc<RwLock<u32>>,
}

impl<S> Service<Request> for AuthMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let future = self.inner.call(request);
        let mutex = self.l.clone();
        Box::pin(async move {
            if let Ok(mut i) = mutex.write() {
                *i += 1;

                info!("{}", i);
            }
            info!("hi");
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
