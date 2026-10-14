//! `propagate_throttle_expiry_id` reads
//! `acme.ratelimiting.v1.ThrottleLifecycleService/ScheduleThrottleExpiry` and hands what it keeps to the sink
//! of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::procurement::v1 as procurement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient::new(channel.clone());
    let fetched = source
        .schedule_throttle_expiry(ratelimiting::ScheduleThrottleExpiryRequest {
            context: Some(context.clone()),
            throttle_id: "default-propagate-throttle-expiry-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = fetched.outcome.unwrap_or_default();
    let throttle = fetched.throttle.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = ThrottleExpiryIdRecord {
        reference_id: outcome.reference_id,
        kind: throttle.kind,
        status: outcome.status,
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ThrottleExpiryIdTarget>> = BTreeMap::new();
    sinks.insert(
        "purchase-requisition-query",
        Box::new(PurchaseRequisitionQuerySink {
            client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "flag-rollout-sync",
        Box::new(FlagRolloutSyncSink {
            client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "quota-bucket-workflow",
        Box::new(QuotaBucketWorkflowSink {
            client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient::new(channel),
        }),
    );
    match sinks.get("flag-rollout-sync") {
        Some(sink) => sink.push(record).await,
        None => anyhow::bail!("no sink registered for flag-rollout-sync"),
    }
}

/// What `propagate_throttle_expiry_id` keeps of each ScheduleThrottleExpiry response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleExpiryIdRecord {
    pub reference_id: String,
    pub kind: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `propagate_throttle_expiry_id` hands each record.
trait ThrottleExpiryIdTarget {
    fn push(&self, record: ThrottleExpiryIdRecord) -> Delivery;
}

/// Passes records to `acme.procurement.v1.PurchaseRequisitionQueryService`.
struct PurchaseRequisitionQuerySink {
    client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient<Channel>,
}

impl ThrottleExpiryIdTarget for PurchaseRequisitionQuerySink {
    fn push(&self, record: ThrottleExpiryIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_purchase_requisitions(procurement::SuggestPurchaseRequisitionsRequest {
                    filter: Some(procurement::suggest_purchase_requisitions_request::Filter {
                        owner_id: record.correlation_id,
                        region: record.status,
                        ..Default::default()
                    }),
                    order_by: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagRolloutSyncService`.
struct FlagRolloutSyncSink {
    client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient<Channel>,
}

impl ThrottleExpiryIdTarget for FlagRolloutSyncSink {
    fn push(&self, record: ThrottleExpiryIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_flag_rollout_sync_state(featureflags::GetFlagRolloutSyncStateRequest {
                    flag_rollout_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketWorkflowService`.
struct QuotaBucketWorkflowSink {
    client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient<Channel>,
}

impl ThrottleExpiryIdTarget for QuotaBucketWorkflowSink {
    fn push(&self, record: ThrottleExpiryIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pause_quota_bucket_workflow(ratelimiting::PauseQuotaBucketWorkflowRequest {
                    reason: record.correlation_id,
                    details: Some(ratelimiting::pause_quota_bucket_workflow_request::Details {
                        initiated_by: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
