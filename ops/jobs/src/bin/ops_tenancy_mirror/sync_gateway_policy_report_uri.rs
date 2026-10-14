//! `sync_gateway_policy_report_uri` reads
//! `acme.edgegateway.v1.GatewayPolicyReportingService/ScheduleGatewayPolicyReport` and hands what it keeps to
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
        edgegateway::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient::new(channel.clone());
    let reply = source
        .schedule_gateway_policy_report(edgegateway::ScheduleGatewayPolicyReportRequest {
            context: Some(context.clone()),
            reason: "live-sync-gateway-policy-report-uri".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let period = reply.period.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = GatewayPolicyReportUriRecord {
        download_uri: reply.download_uri,
        timezone: period.timezone,
        message: acme_kit::text::normalize(&result.message),
        report_id: acme_kit::text::normalize(&reply.report_id),
    };
    ChatRoomQuerySink {
        client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient::new(channel),
    }
    .push(record)
    .await
}

/// What `sync_gateway_policy_report_uri` keeps of each ScheduleGatewayPolicyReport response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyReportUriRecord {
    pub download_uri: String,
    pub timezone: String,
    pub message: String,
    pub report_id: String,
}

/// Where `sync_gateway_policy_report_uri` hands each record.
trait GatewayPolicyReportUriTarget {
    fn push(&self, record: GatewayPolicyReportUriRecord) -> Delivery;
}

/// Writes records to `acme.chat.v1.ChatRoomQueryService`.
struct ChatRoomQuerySink {
    client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient<Channel>,
}

impl GatewayPolicyReportUriTarget for ChatRoomQuerySink {
    fn push(&self, record: GatewayPolicyReportUriRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_chat_rooms(chat::SuggestChatRoomsRequest {
                    filter: Some(chat::suggest_chat_rooms_request::Filter {
                        owner_id: record.report_id,
                        region: record.timezone,
                        query: record.download_uri,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
