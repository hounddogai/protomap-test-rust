//! `backfill_quota_buckets_status` reads `acme.ratelimiting.v1.QuotaBucketQueryService/BrowseQuotaBuckets`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::inventory::v1 as inventory;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::quota_bucket_query_service_client::QuotaBucketQueryServiceClient::new(channel.clone());
    let found = source
        .browse_quota_buckets(ratelimiting::BrowseQuotaBucketsRequest {
            context: Some(context.clone()),
            order_by: "live-backfill-quota-buckets-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let record = QuotaBucketsStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn QuotaBucketsStatusSink>> = vec![
        Box::new(EnvironmentApprovalSink {
            client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(ConfigChangeWorkflowSink {
            client: configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(StockItemQuerySink {
            client: inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[0].forward(record).await
}

/// What `backfill_quota_buckets_status` keeps of each BrowseQuotaBuckets response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketsStatusRecord {
    pub status: String,
    pub correlation_id: String,
}

/// Where `backfill_quota_buckets_status` hands each record.
trait QuotaBucketsStatusSink {
    fn forward(&self, record: QuotaBucketsStatusRecord) -> Delivery;
}

/// Writes records to `acme.configuration.v1.EnvironmentApprovalService`.
struct EnvironmentApprovalSink {
    client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient<Channel>,
}

impl QuotaBucketsStatusSink for EnvironmentApprovalSink {
    fn forward(&self, record: QuotaBucketsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .comment_environment_approval(configuration::CommentEnvironmentApprovalRequest {
                    environment_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.ConfigChangeWorkflowService`.
struct ConfigChangeWorkflowSink {
    client: configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient<Channel>,
}

impl QuotaBucketsStatusSink for ConfigChangeWorkflowSink {
    fn forward(&self, record: QuotaBucketsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_config_change_workflow(configuration::AdvanceConfigChangeWorkflowRequest {
                    details: Some(configuration::advance_config_change_workflow_request::Details {
                        initiated_by: record.correlation_id,
                        ..Default::default()
                    }),
                    config_change_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.inventory.v1.StockItemQueryService`.
struct StockItemQuerySink {
    client: inventory::stock_item_query_service_client::StockItemQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl QuotaBucketsStatusSink for StockItemQuerySink {
    fn forward(&self, record: QuotaBucketsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(inventory::SummarizeStockItemsRequest {
                    details: Some(inventory::summarize_stock_items_request::Details {
                        format: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_stock_items(request).await?;
            Ok(())
        })
    }
}
