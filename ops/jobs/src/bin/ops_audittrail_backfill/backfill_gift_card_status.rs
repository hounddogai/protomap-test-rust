//! `backfill_gift_card_status` reads `acme.giftcards.v1.GiftCardService/CloneGiftCard` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = giftcards::gift_card_service_client::GiftCardServiceClient::new(channel.clone());
    let fetched = source
        .clone_gift_card(giftcards::CloneGiftCardRequest {
            context: Some(context.clone()),
            request_id: "primary-backfill-gift-card-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let gift_card = fetched.gift_card.unwrap_or_default();
    let record = GiftCardStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        gift_card_id: gift_card.gift_card_id,
        currency_code: acme_kit::text::normalize(&gift_card.currency_code),
    };
    let sinks: Vec<(&str, Box<dyn GiftCardStatusSink>)> = vec![
        (
            "gateway-policy-integration",
            Box::new(GatewayPolicyIntegrationSink {
                client:
                    edgegateway::gateway_policy_integration_service_client::GatewayPolicyIntegrationServiceClient::new(
                        channel.clone(),
                    ),
                context: context.clone(),
            }),
        ),
        (
            "key-ring-approval",
            Box::new(KeyRingApprovalSink {
                client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "gateway-policy-integration" {
            continue;
        }
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `backfill_gift_card_status` keeps of each CloneGiftCard response.
#[derive(Clone, Debug, Default)]
pub struct GiftCardStatusRecord {
    pub status: String,
    pub gift_card_id: String,
    pub currency_code: String,
}

/// Where `backfill_gift_card_status` hands each record.
trait GiftCardStatusSink {
    fn send(&self, record: GiftCardStatusRecord) -> Delivery;
}

/// Passes records to `acme.edgegateway.v1.GatewayPolicyIntegrationService`.
struct GatewayPolicyIntegrationSink {
    client: edgegateway::gateway_policy_integration_service_client::GatewayPolicyIntegrationServiceClient<Channel>,
    context: common::RequestContext,
}

impl GiftCardStatusSink for GatewayPolicyIntegrationSink {
    fn send(&self, record: GiftCardStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::AcknowledgeGatewayPolicyEventRequest {
                    details: Some(edgegateway::acknowledge_gateway_policy_event_request::Details {
                        message_id: record.gift_card_id,
                        source_system: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.acknowledge_gateway_policy_event(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRingApprovalService`.
struct KeyRingApprovalSink {
    client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient<Channel>,
}

impl GiftCardStatusSink for KeyRingApprovalSink {
    fn send(&self, record: GiftCardStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recall_key_ring(keymanagement::RecallKeyRingRequest {
                    key_ring_id: record.gift_card_id,
                    key_ring: Some(keymanagement::KeyRing {
                        parent_id: record.status,
                        name: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
