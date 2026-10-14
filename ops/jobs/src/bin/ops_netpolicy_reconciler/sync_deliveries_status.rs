//! `sync_deliveries_status` reads `acme.deliveries.v1.DeliveryQueryService/BrowseDeliveries` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel.clone());
    let reply = source
        .browse_deliveries(deliveries::BrowseDeliveriesRequest {
            context: Some(context.clone()),
            order_by: "main-sync-deliveries-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = DeliveriesStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        next_page_token: page_info.next_page_token,
    };
    let sinks: Vec<(&str, Box<dyn DeliveriesStatusSink>)> = vec![
        (
            "broadcast",
            Box::new(BroadcastSink {
                client: broadcasts::broadcast_service_client::BroadcastServiceClient::new(channel.clone()),
            }),
        ),
        (
            "api-key-workflow",
            Box::new(ApiKeyWorkflowSink {
                client: edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "api-key-workflow" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `sync_deliveries_status` keeps of each BrowseDeliveries response.
#[derive(Clone, Debug, Default)]
pub struct DeliveriesStatusRecord {
    pub status: String,
    pub next_page_token: String,
}

/// Where `sync_deliveries_status` hands each record.
trait DeliveriesStatusSink {
    fn deliver(&self, record: DeliveriesStatusRecord) -> Delivery;
}

/// Writes records to `acme.broadcasts.v1.BroadcastService`.
struct BroadcastSink {
    client: broadcasts::broadcast_service_client::BroadcastServiceClient<Channel>,
}

impl DeliveriesStatusSink for BroadcastSink {
    fn deliver(&self, record: DeliveriesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_broadcast(broadcasts::CreateBroadcastRequest {
                    broadcast: Some(broadcasts::Broadcast {
                        title: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.edgegateway.v1.ApiKeyWorkflowService`.
struct ApiKeyWorkflowSink {
    client: edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient<Channel>,
}

impl DeliveriesStatusSink for ApiKeyWorkflowSink {
    fn deliver(&self, record: DeliveriesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_api_key_workflows(edgegateway::ListApiKeyWorkflowsRequest {
                    filter: Some(edgegateway::list_api_key_workflows_request::Filter {
                        region: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
