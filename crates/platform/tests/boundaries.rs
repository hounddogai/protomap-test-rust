//! Verify that authentication, serialization, and stream termination preserve their boundary contracts.

use acme_platform::{
    auth::{IdentityLayer, Principal, require_identity},
    codec,
};
use acme_proto::acme::common::v1::{Actor, RequestContext};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio_stream::StreamExt;
use tonic::{Code, body::Body, transport::Channel};
use tower::{Layer, ServiceExt};

#[test]
fn test_identity_guard_rejects_missing_and_empty_principals() {
    let missing = require_identity(tonic::Request::new(()));
    assert_eq!(missing.unwrap_err().code(), Code::Unauthenticated);
    let mut empty = tonic::Request::new(());
    empty.extensions_mut().insert(Principal { subject: String::new() });
    assert_eq!(require_identity(empty).unwrap_err().code(), Code::Unauthenticated);
    let mut authenticated = tonic::Request::new(());
    authenticated.extensions_mut().insert(Principal {
        subject: "service-a".into(),
    });
    assert!(require_identity(authenticated).is_ok());
}

#[tokio::test]
async fn test_identity_layer_rejects_before_dispatch() -> Result<(), Box<dyn std::error::Error>> {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let inner = tower::service_fn(move |_: http::Request<Body>| {
        observed.fetch_add(1, Ordering::SeqCst);
        async { Ok::<_, std::convert::Infallible>(http::Response::new(Body::empty())) }
    });
    let channel = Channel::from_static("http://127.0.0.1:9").connect_lazy();
    let service = IdentityLayer::new(channel).layer(inner);
    let response = service.oneshot(http::Request::new(Body::empty())).await?;
    assert_eq!(response.headers().get("grpc-status").unwrap(), "16");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    Ok(())
}

#[test]
fn test_wire_envelopes_preserve_nested_and_repeated_data() -> Result<(), tonic::Status> {
    let context = RequestContext {
        tenant_id: "tenant-a".into(),
        trace_id: "trace-a".into(),
        actor: Some(Actor {
            user_id: "user-a".into(),
            roles: vec!["reader".into(), "writer".into()],
            ..Default::default()
        }),
        baggage: BTreeMap::from([("region".into(), "west".into()), ("locale".into(), "日本語".into())]),
        ..Default::default()
    };
    let decoded = codec::through_any(&context, "acme.common.v1.RequestContext")?;
    assert_eq!(decoded, context);
    let bytes = codec::json_payload(codec::AuditEnvelope {
        trace_id: "trace\n\"\u{0}日本語".into(),
        tenant_id: context.tenant_id,
    })?;
    let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(payload["trace_id"], "trace\n\"\u{0}日本語");
    assert_eq!(payload["tenant_id"], "tenant-a");
    Ok(())
}

#[tokio::test]
async fn test_snapshot_stream_propagates_failure_then_closes() {
    let mut stream =
        acme_platform::transport::one_reply(async { Err::<String, _>(tonic::Status::unavailable("upstream stopped")) });
    assert_eq!(stream.next().await.unwrap().unwrap_err().code(), Code::Unavailable);
    assert!(stream.next().await.is_none());
}
