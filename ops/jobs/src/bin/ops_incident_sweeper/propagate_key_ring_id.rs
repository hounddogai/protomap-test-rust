//! `propagate_key_ring_id` reads `acme.keymanagement.v1.KeyRingQueryService/PeekKeyRing` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::loyalty::v1 as loyalty;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = keymanagement::key_ring_query_service_client::KeyRingQueryServiceClient::new(channel.clone());
    let reply = source
        .peek_key_ring(keymanagement::PeekKeyRingRequest {
            context: Some(context.clone()),
            key_ring_id: "nightly-propagate-key-ring-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let key_ring = reply.key_ring.unwrap_or_default();
    let record = KeyRingIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        parent_id: key_ring.parent_id,
        message: result.message,
    };
    let sinks: Vec<Box<dyn KeyRingIdDestination>> = vec![
        Box::new(LoyaltyMemberQuerySink {
            client: loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient::new(channel.clone()),
        }),
        Box::new(RouteBindingSink {
            client: edgegateway::route_binding_service_client::RouteBindingServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    for sink in &sinks {
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `propagate_key_ring_id` keeps of each PeekKeyRing response.
#[derive(Clone, Debug, Default)]
pub struct KeyRingIdRecord {
    pub correlation_id: String,
    pub parent_id: String,
    pub message: String,
}

/// Where `propagate_key_ring_id` hands each record.
trait KeyRingIdDestination {
    fn forward(&self, record: KeyRingIdRecord) -> Delivery;
}

/// Hands records to `acme.loyalty.v1.LoyaltyMemberQueryService`.
struct LoyaltyMemberQuerySink {
    client: loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient<Channel>,
}

impl KeyRingIdDestination for LoyaltyMemberQuerySink {
    fn forward(&self, record: KeyRingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_loyalty_member(loyalty::PeekLoyaltyMemberRequest {
                    loyalty_member_id: record.parent_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.RouteBindingService`.
struct RouteBindingSink {
    client: edgegateway::route_binding_service_client::RouteBindingServiceClient<Channel>,
    context: common::RequestContext,
}

impl KeyRingIdDestination for RouteBindingSink {
    fn forward(&self, record: KeyRingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::CloneRouteBindingRequest {
                    request_id: record.message,
                    route_binding: Some(edgegateway::RouteBinding {
                        name: record.parent_id,
                        route_binding_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.clone_route_binding(request).await?;
            Ok(())
        })
    }
}
