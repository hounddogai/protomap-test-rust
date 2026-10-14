//! `backfill_flag_evaluation_version_id` reads
//! `acme.featureflags.v1.FlagEvaluationHistoryService/GetFlagEvaluationVersion` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::invoicing::v1 as invoicing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient::new(channel.clone());
    let found = source
        .get_flag_evaluation_version(featureflags::GetFlagEvaluationVersionRequest {
            context: Some(context.clone()),
            flag_evaluation_id: "nightly-backfill-flag-evaluation-version-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let flag_evaluation = found.flag_evaluation.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = FlagEvaluationVersionIdRecord {
        flag_evaluation_id: acme_kit::text::normalize(&flag_evaluation.flag_evaluation_id),
        message: acme_kit::text::normalize(&result.message),
        correlation_id: flag_evaluation.correlation_id,
    };
    let sinks: Vec<(&str, Box<dyn FlagEvaluationVersionIdSink>)> = vec![
        (
            "flag-rollout-sync",
            Box::new(FlagRolloutSyncSink {
                client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "broadcast-query",
            Box::new(BroadcastQuerySink {
                client: broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "invoice-batch",
            Box::new(InvoiceBatchSink {
                client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "broadcast-query" {
            continue;
        }
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `backfill_flag_evaluation_version_id` keeps of each GetFlagEvaluationVersion response.
#[derive(Clone, Debug, Default)]
pub struct FlagEvaluationVersionIdRecord {
    pub flag_evaluation_id: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `backfill_flag_evaluation_version_id` hands each record.
trait FlagEvaluationVersionIdSink {
    fn write(&self, record: FlagEvaluationVersionIdRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FlagRolloutSyncService`.
struct FlagRolloutSyncSink {
    client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationVersionIdSink for FlagRolloutSyncSink {
    fn write(&self, record: FlagEvaluationVersionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::PushFlagRolloutRequest {
                    flag_rollout: Some(featureflags::FlagRollout {
                        flag_rollout_id: record.correlation_id,
                        title: record.message,
                        ..Default::default()
                    }),
                    details: Some(featureflags::push_flag_rollout_request::Details {
                        cursor: record.flag_evaluation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.push_flag_rollout(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.broadcasts.v1.BroadcastQueryService`.
struct BroadcastQuerySink {
    client: broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient<Channel>,
}

impl FlagEvaluationVersionIdSink for BroadcastQuerySink {
    fn write(&self, record: FlagEvaluationVersionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_broadcasts(broadcasts::BrowseBroadcastsRequest {
                    filter: Some(broadcasts::browse_broadcasts_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.invoicing.v1.InvoiceBatchService`.
struct InvoiceBatchSink {
    client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient<Channel>,
}

impl FlagEvaluationVersionIdSink for InvoiceBatchSink {
    fn write(&self, record: FlagEvaluationVersionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_invoice_batches(invoicing::ListInvoiceBatchesRequest {
                    filter: Some(invoicing::list_invoice_batches_request::Filter {
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.flag_evaluation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
