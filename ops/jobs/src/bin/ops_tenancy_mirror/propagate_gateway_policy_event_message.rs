//! `propagate_gateway_policy_event_message` reads
//! `acme.edgegateway.v1.GatewayPolicyIntegrationService/AcknowledgeGatewayPolicyEvent` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::gateway_policy_integration_service_client::GatewayPolicyIntegrationServiceClient::new(
        channel.clone(),
    );
    let reply = source
        .acknowledge_gateway_policy_event(edgegateway::AcknowledgeGatewayPolicyEventRequest {
            context: Some(context.clone()),
            reason: "default-propagate-gateway-policy-event-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let gateway_policy = reply.gateway_policy.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = GatewayPolicyEventMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        name: gateway_policy.name,
        status: outcome.status,
    };
    let sinks: Vec<(&str, Box<dyn GatewayPolicyEventMessageSink>)> = vec![
        (
            "fan-session",
            Box::new(FanSessionSink {
                client: fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone()),
            }),
        ),
        (
            "crypto-key-workflow",
            Box::new(CryptoKeyWorkflowSink {
                client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "fan-session" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `propagate_gateway_policy_event_message` keeps of each AcknowledgeGatewayPolicyEvent response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyEventMessageRecord {
    pub message: String,
    pub name: String,
    pub status: String,
}

/// Where `propagate_gateway_policy_event_message` hands each record.
trait GatewayPolicyEventMessageSink {
    fn deliver(&self, record: GatewayPolicyEventMessageRecord) -> Delivery;
}

/// Sends records to `acme.fanportal.v1.FanSessionService`.
struct FanSessionSink {
    client: fanportal::fan_session_service_client::FanSessionServiceClient<Channel>,
}

impl GatewayPolicyEventMessageSink for FanSessionSink {
    fn deliver(&self, record: GatewayPolicyEventMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_fan_sessions(fanportal::ListFanSessionsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    filter: Some(fanportal::list_fan_sessions_request::Filter {
                        region: record.name,
                        owner_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyWorkflowService`.
struct CryptoKeyWorkflowSink {
    client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl GatewayPolicyEventMessageSink for CryptoKeyWorkflowSink {
    fn deliver(&self, record: GatewayPolicyEventMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::InspectCryptoKeyWorkflowRequest {
                    crypto_key_id: record.name,
                    ..Default::default()
                }),
                &context,
            )?;
            client.inspect_crypto_key_workflow(request).await?;
            Ok(())
        })
    }
}
