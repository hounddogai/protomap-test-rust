//! `backfill_labeling_tasks_id` reads `acme.labeling.v1.LabelingTaskQueryService/ResolveLabelingTasks` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::labeling::v1 as labeling;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient::new(channel.clone());
    let found = source
        .resolve_labeling_tasks(labeling::ResolveLabelingTasksRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = LabelingTasksIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: result.status,
        next_page_token: page_info.next_page_token,
    };
    deliver_labeling_tasks_id("key-ring-approval", record, channel, &context).await
}

/// What `backfill_labeling_tasks_id` keeps of each ResolveLabelingTasks response.
#[derive(Clone, Debug, Default)]
pub struct LabelingTasksIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `backfill_labeling_tasks_id` hands each record.
trait LabelingTasksIdWriter {
    fn deliver(&self, record: LabelingTasksIdRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FeatureFlagHistoryService`.
struct FeatureFlagHistorySink {
    client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient<Channel>,
}

impl LabelingTasksIdWriter for FeatureFlagHistorySink {
    fn deliver(&self, record: LabelingTasksIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .annotate_feature_flag_version(featureflags::AnnotateFeatureFlagVersionRequest {
                    feature_flag_id: record.next_page_token,
                    feature_flag: Some(featureflags::FeatureFlag {
                        feature_flag_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.KeyRingApprovalService`.
struct KeyRingApprovalSink {
    client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl LabelingTasksIdWriter for KeyRingApprovalSink {
    fn deliver(&self, record: LabelingTasksIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ApproveKeyRingRequest {
                    reason: record.correlation_id,
                    details: Some(keymanagement::approve_key_ring_request::Details {
                        approver_email: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.approve_key_ring(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct LabelingTasksIdLogOnly;

impl LabelingTasksIdWriter for LabelingTasksIdLogOnly {
    fn deliver(&self, record: LabelingTasksIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_labeling_tasks_id(
    kind: &str,
    record: LabelingTasksIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn LabelingTasksIdWriter> = match kind {
        "feature-flag-history" => Box::new(FeatureFlagHistorySink {
            client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(
                channel.clone(),
            ),
        }),
        "key-ring-approval" => Box::new(KeyRingApprovalSink {
            client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(LabelingTasksIdLogOnly),
    };
    sink.deliver(record).await
}
