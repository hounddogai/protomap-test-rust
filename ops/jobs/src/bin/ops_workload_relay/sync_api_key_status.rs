//! `sync_api_key_status` reads `acme.edgegateway.v1.ApiKeyService/CreateApiKey` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::api_key_service_client::ApiKeyServiceClient::new(channel.clone());
    let reply = source
        .create_api_key(edgegateway::CreateApiKeyRequest {
            context: Some(context.clone()),
            request_id: "default-sync-api-key-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let api_key = reply.api_key.unwrap_or_default();
    let record = ApiKeyStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
        message: result.message,
        currency_code: api_key.currency_code,
    };
    let sinks: Vec<(&str, Box<dyn ApiKeyStatusDestination>)> = vec![
        (
            "broadcast",
            Box::new(BroadcastSink {
                client: broadcasts::broadcast_service_client::BroadcastServiceClient::new(channel.clone()),
            }),
        ),
        (
            "config-change-approval",
            Box::new(ConfigChangeApprovalSink {
                client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient::new(
                    channel,
                ),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "broadcast" {
            continue;
        }
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `sync_api_key_status` keeps of each CreateApiKey response.
#[derive(Clone, Debug, Default)]
pub struct ApiKeyStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub message: String,
    pub currency_code: String,
}

/// Where `sync_api_key_status` hands each record.
trait ApiKeyStatusDestination {
    fn forward(&self, record: ApiKeyStatusRecord) -> Delivery;
}

/// Passes records to `acme.broadcasts.v1.BroadcastService`.
struct BroadcastSink {
    client: broadcasts::broadcast_service_client::BroadcastServiceClient<Channel>,
}

impl ApiKeyStatusDestination for BroadcastSink {
    fn forward(&self, record: ApiKeyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_broadcasts(broadcasts::SearchBroadcastsRequest {
                    order_by: record.status,
                    filter: Some(broadcasts::search_broadcasts_request::Filter {
                        query: record.currency_code,
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigChangeApprovalService`.
struct ConfigChangeApprovalSink {
    client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl ApiKeyStatusDestination for ConfigChangeApprovalSink {
    fn forward(&self, record: ApiKeyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::SubmitConfigChangeRequest {
                    config_change: Some(configuration::ConfigChange {
                        kind: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.submit_config_change(request).await?;
            Ok(())
        })
    }
}
