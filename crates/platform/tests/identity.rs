//! Verify that the identity layer resolves callers as the identity service answers its lookups.

use acme_platform::auth::IdentityLayer;
use acme_proto::acme::common::v1::OperationResult;
use acme_proto::acme::identity::v1::{LookupUserRequest, LookupUserResponse, User};
use prost::Message;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};
use tonic::{
    Status,
    body::Body,
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
};
use tower::{Layer, ServiceExt};

/// The identity service as the Go monolith serves it to lookups: they bear the workload credential and the
/// tenant of their request, and name a service account. It answers the record of the id it was asked about,
/// and rejects a revoked token.
#[derive(Clone, Default)]
struct Identity {
    lookups: Arc<Mutex<Vec<(String, String)>>>,
}

impl tonic::server::UnaryService<LookupUserRequest> for Identity {
    type Response = LookupUserResponse;
    type Future = Pin<Box<dyn Future<Output = Result<tonic::Response<LookupUserResponse>, Status>> + Send>>;

    fn call(&mut self, request: tonic::Request<LookupUserRequest>) -> Self::Future {
        let lookups = self.lookups.clone();
        Box::pin(async move {
            let header = |name: &str| {
                request
                    .metadata()
                    .get(name)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_owned()
            };
            let (authorization, tenant) = (header("authorization"), header("x-tenant-id"));
            let lookup = request.into_inner();
            let context = lookup.context.unwrap_or_default();
            if authorization != "Bearer workload-token" || tenant.is_empty() || tenant != context.tenant_id {
                return Err(Status::unauthenticated("missing bearer token or tenant"));
            }
            if !context.actor.is_some_and(|actor| actor.service_account) {
                return Err(Status::unauthenticated("context.actor is required"));
            }
            lookups
                .lock()
                .map_err(|_| Status::internal("poisoned"))?
                .push((lookup.user_id.clone(), tenant));
            let status = if lookup.user_id == "revoked-token" {
                "revoked"
            } else {
                "ok"
            };
            Ok(tonic::Response::new(LookupUserResponse {
                user: Some(User {
                    user_id: lookup.user_id,
                    ..Default::default()
                }),
                result: Some(OperationResult {
                    status: status.into(),
                    ..Default::default()
                }),
                ..Default::default()
            }))
        })
    }
}

impl tonic::server::NamedService for Identity {
    const NAME: &'static str = "acme.identity.v1.UserQueryService";
}

impl tower::Service<http::Request<Body>> for Identity {
    type Response = http::Response<Body>;
    type Error = std::convert::Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: http::Request<Body>) -> Self::Future {
        let identity = self.clone();
        Box::pin(async move { Ok(tonic::server::Grpc::new(Wire).unary(identity, request).await) })
    }
}

/// The protobuf encoding of the lookup's messages, as the generated servers have it.
struct Wire;

impl Codec for Wire {
    type Encode = LookupUserResponse;
    type Decode = LookupUserRequest;
    type Encoder = Wire;
    type Decoder = Wire;
    fn encoder(&mut self) -> Self::Encoder {
        Wire
    }
    fn decoder(&mut self) -> Self::Decoder {
        Wire
    }
}

impl Encoder for Wire {
    type Item = LookupUserResponse;
    type Error = Status;
    fn encode(&mut self, item: Self::Item, output: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        item.encode(output).map_err(|error| Status::internal(error.to_string()))
    }
}

impl Decoder for Wire {
    type Item = LookupUserRequest;
    type Error = Status;
    fn decode(&mut self, input: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        LookupUserRequest::decode(input)
            .map(Some)
            .map_err(|error| Status::internal(error.to_string()))
    }
}

#[tokio::test]
async fn test_identity_layer_resolves_callers_with_the_workload_credential() -> Result<(), Box<dyn std::error::Error>> {
    // SAFETY: no other test of this binary reads or writes the environment.
    unsafe { std::env::set_var("ACME_SERVICE_TOKEN", "workload-token") };
    let identity = Identity::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let _server = tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(identity.clone())
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener)),
    );
    let channel = tonic::transport::Channel::from_shared(format!("http://{address}"))?.connect_lazy();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let inner = tower::service_fn(move |_: http::Request<Body>| {
        observed.fetch_add(1, Ordering::SeqCst);
        async { Ok::<_, std::convert::Infallible>(http::Response::new(Body::empty())) }
    });
    let service = IdentityLayer::new(channel).layer(inner);
    for (token, status) in [("caller-token", None), ("revoked-token", Some("7"))] {
        let request = http::Request::builder()
            .header("authorization", format!("Bearer {token}"))
            .header("x-tenant-id", "tenant-a")
            .body(Body::empty())?;
        let response = service.clone().oneshot(request).await?;
        let found = response
            .headers()
            .get("grpc-status")
            .and_then(|value| value.to_str().ok());
        assert_eq!(found, status, "{token}");
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let lookups = identity.lookups.lock().map_err(|_| "poisoned")?.clone();
    assert_eq!(
        lookups,
        vec![
            ("caller-token".to_owned(), "tenant-a".to_owned()),
            ("revoked-token".to_owned(), "tenant-a".to_owned())
        ]
    );
    Ok(())
}
