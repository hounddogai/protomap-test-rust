//! Ports are injected at composition time, so a workflow can use a concrete adapter or a trait object.

use std::{future::Future, pin::Pin, sync::Arc};
use tonic::Status;

pub type RpcFuture<T> = Pin<Box<dyn Future<Output = Result<T, Status>> + Send + 'static>>;
pub type Callback<Q, R> = Arc<dyn Fn(Q) -> RpcFuture<R> + Send + Sync>;

pub trait RpcPort<Q, R>: Send + Sync {
    fn send(&self, request: Q) -> RpcFuture<R>;
}

pub struct Adapter<Q, R> {
    callback: Callback<Q, R>,
}

impl<Q, R> Adapter<Q, R> {
    pub fn new(callback: Callback<Q, R>) -> Self {
        Self { callback }
    }
}

impl<Q, R> RpcPort<Q, R> for Adapter<Q, R> {
    fn send(&self, request: Q) -> RpcFuture<R> {
        (self.callback)(request)
    }
}

pub async fn execute<Q, R, P: RpcPort<Q, R> + ?Sized>(port: &P, request: Q) -> Result<R, Status> {
    port.send(request).await
}

/// Hide the adapter's concrete type from workflows that only need its port.
pub fn opaque<Q, R>(callback: Callback<Q, R>) -> impl RpcPort<Q, R> {
    Adapter::new(callback)
}

pub struct RequestBuilder<Q> {
    request: Q,
    timeout: std::time::Duration,
}

impl<Q: Default> Default for RequestBuilder<Q> {
    fn default() -> Self {
        Self {
            request: Q::default(),
            timeout: std::time::Duration::from_secs(10),
        }
    }
}

impl<Q> RequestBuilder<Q> {
    pub fn map(mut self, update: impl FnOnce(&mut Q)) -> Self {
        update(&mut self.request);
        self
    }

    pub fn deadline(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> tonic::Request<Q> {
        let mut request = tonic::Request::new(self.request);
        request.set_timeout(self.timeout);
        request
    }
}
