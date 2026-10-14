//! `settle_gateway_policy_workflow_name` reads
//! `acme.edgegateway.v1.GatewayPolicyWorkflowService/StartGatewayPolicyWorkflow` and hands what it keeps to
//! `acme.chat.v1.ChatRoomQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient::new(channel.clone());
    let reply = source
        .start_gateway_policy_workflow(edgegateway::StartGatewayPolicyWorkflowRequest {
            context: Some(context.clone()),
            gateway_policy_id: "primary-settle-gateway-policy-workflow-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let gateway_policy = reply.gateway_policy.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = GatewayPolicyWorkflowNameRecord {
        name: gateway_policy.name,
        status: acme_kit::text::normalize(&outcome.status),
    };
    ChatRoomQuerySink {
        client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient::new(channel),
    }
    .write(record)
    .await
}

/// What `settle_gateway_policy_workflow_name` keeps of each StartGatewayPolicyWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyWorkflowNameRecord {
    pub name: String,
    pub status: String,
}

/// Where `settle_gateway_policy_workflow_name` hands each record.
trait GatewayPolicyWorkflowNameTarget {
    fn write(&self, record: GatewayPolicyWorkflowNameRecord) -> Delivery;
}

/// Writes records to `acme.chat.v1.ChatRoomQueryService`.
struct ChatRoomQuerySink {
    client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient<Channel>,
}

impl GatewayPolicyWorkflowNameTarget for ChatRoomQuerySink {
    fn write(&self, record: GatewayPolicyWorkflowNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_chat_room(chat::PeekChatRoomRequest {
                    chat_room_id: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
