//! Async identity checks live in a tower Layer because tonic's synchronous interceptor cannot await an RPC.

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_query_service_client::UserQueryServiceClient;

use common::RequestContext;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tonic::{Status, body::Body, transport::Channel};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct Principal {
    pub subject: String,
}

#[derive(Clone)]
pub struct IdentityLayer {
    channel: Channel,
}

impl IdentityLayer {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl<S> Layer<S> for IdentityLayer {
    type Service = IdentityService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        IdentityService {
            inner,
            channel: self.channel.clone(),
        }
    }
}

#[derive(Clone)]
pub struct IdentityService<S> {
    inner: S,
    channel: Channel,
}

impl<S> Service<http::Request<Body>> for IdentityService<S>
where
    S: Service<http::Request<Body>, Response = http::Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = http::Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context)
    }

    fn call(&mut self, mut request: http::Request<Body>) -> Self::Future {
        // Move the ready service into the future, leaving its clone for the next readiness poll.
        let replacement = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, replacement);
        let channel = self.channel.clone();
        Box::pin(async move {
            match check(channel, request.headers()).await {
                Ok(principal) => {
                    request.extensions_mut().insert(principal);
                }
                Err(status) => return Ok(status.into_http()),
            }
            inner.call(request).await
        })
    }
}

/// The synchronous guard makes accidentally mounting an endpoint without the identity layer fail closed.
pub fn require_identity(request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
    match request.extensions().get::<Principal>() {
        Some(principal) if !principal.subject.is_empty() => Ok(request),
        _ => Err(Status::unauthenticated("identity middleware is required")),
    }
}

/// Attach the workload credential and the tenant envelope to outbound requests, including stream headers.
pub fn service_request<T>(
    mut request: tonic::Request<T>,
    context: &RequestContext,
) -> Result<tonic::Request<T>, Status> {
    let token = std::env::var("ACME_SERVICE_TOKEN")
        .map_err(|_| Status::failed_precondition("ACME_SERVICE_TOKEN is required"))?;
    let bearer = format!("Bearer {token}")
        .parse()
        .map_err(|_| Status::failed_precondition("workload token is invalid"))?;
    request.metadata_mut().insert("authorization", bearer);
    request.metadata_mut().insert(
        "x-tenant-id",
        context
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("tenant metadata is invalid"))?,
    );
    request.metadata_mut().insert(
        "x-trace-id",
        context
            .trace_id
            .parse()
            .map_err(|_| Status::invalid_argument("trace metadata is invalid"))?,
    );
    Ok(request)
}

/// Resolve the caller through acme.identity.v1.UserQueryService/LookupUser; failures never reach an application
/// handler.
async fn check(channel: Channel, headers: &http::HeaderMap) -> Result<Principal, Status> {
    let token = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Status::unauthenticated("a bearer token is required"))?;
    let tenant = headers
        .get("x-tenant-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if tenant.is_empty() {
        return Err(Status::unauthenticated("tenant metadata is required"));
    }
    let trace = headers
        .get("x-trace-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let mut identity = UserQueryServiceClient::new(channel);
    let context = RequestContext {
        tenant_id: tenant.to_owned(),
        trace_id: trace.to_owned(),
        actor: Some(common::Actor {
            service_account: true,
            user_id: "platform-auth".into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    let query = identity::LookupUserRequest {
        user_id: token.to_owned(),
        context: Some(context.clone()),
        ..Default::default()
    };
    // The lookup is this server's own call, which bears its workload credential.
    let response: identity::LookupUserResponse = identity
        .lookup_user(service_request(tonic::Request::new(query), &context)?)
        .await
        .map_err(|error| {
            tracing::warn!(code = ?error.code(), "identity check failed");
            Status::unauthenticated("identity check failed")
        })?
        .into_inner();
    if response.result.as_ref().map(|result| result.status.as_str()) != Some("ok") {
        return Err(Status::permission_denied("identity rejected the caller"));
    }
    let record = response
        .user
        .ok_or_else(|| Status::permission_denied("identity was not resolved"))?;
    if record.user_id != token {
        return Err(Status::permission_denied("identity resolved another caller"));
    }
    Ok(Principal {
        subject: record.user_id,
    })
}
