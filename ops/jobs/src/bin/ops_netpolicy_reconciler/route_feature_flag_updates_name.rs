//! `route_feature_flag_updates_name` reads
//! `acme.featureflags.v1.FeatureFlagNotificationService/UnsubscribeFeatureFlagUpdates` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient::new(
        channel.clone(),
    );
    let loaded = source
        .unsubscribe_feature_flag_updates(featureflags::UnsubscribeFeatureFlagUpdatesRequest {
            context: Some(context.clone()),
            feature_flag_id: "primary-route-feature-flag-updates-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let feature_flag = loaded.feature_flag.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = FeatureFlagUpdatesNameRecord {
        name: feature_flag.name,
        status: acme_kit::text::normalize(&result.status),
        description: feature_flag.description,
        feature_flag_id: feature_flag.feature_flag_id,
    };
    deliver_feature_flag_updates_name("key-ring-workflow", record, channel, &context).await
}

/// What `route_feature_flag_updates_name` keeps of each UnsubscribeFeatureFlagUpdates response.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlagUpdatesNameRecord {
    pub name: String,
    pub status: String,
    pub description: String,
    pub feature_flag_id: String,
}

/// Where `route_feature_flag_updates_name` hands each record.
trait FeatureFlagUpdatesNameDestination {
    fn apply(&self, record: FeatureFlagUpdatesNameRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.RateLimitLifecycleService`.
struct RateLimitLifecycleSink {
    client: ratelimiting::rate_limit_lifecycle_service_client::RateLimitLifecycleServiceClient<Channel>,
}

impl FeatureFlagUpdatesNameDestination for RateLimitLifecycleSink {
    fn apply(&self, record: FeatureFlagUpdatesNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_rate_limit_lifecycle(ratelimiting::GetRateLimitLifecycleRequest {
                    rate_limit_id: record.feature_flag_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRingWorkflowService`.
struct KeyRingWorkflowSink {
    client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl FeatureFlagUpdatesNameDestination for KeyRingWorkflowSink {
    fn apply(&self, record: FeatureFlagUpdatesNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ListKeyRingWorkflowsRequest {
                    order_by: record.name,
                    filter: Some(keymanagement::list_key_ring_workflows_request::Filter {
                        owner_id: record.status,
                        region: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_key_ring_workflows(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
}

impl FeatureFlagUpdatesNameDestination for KeyRotationWorkflowSink {
    fn apply(&self, record: FeatureFlagUpdatesNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .signal_key_rotation_workflow(keymanagement::SignalKeyRotationWorkflowRequest {
                    reason: record.description,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct FeatureFlagUpdatesNameLogOnly;

impl FeatureFlagUpdatesNameDestination for FeatureFlagUpdatesNameLogOnly {
    fn apply(&self, record: FeatureFlagUpdatesNameRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.name);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_feature_flag_updates_name(
    kind: &str,
    record: FeatureFlagUpdatesNameRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn FeatureFlagUpdatesNameDestination> = match kind {
        "rate-limit-lifecycle" => Box::new(RateLimitLifecycleSink {
            client: ratelimiting::rate_limit_lifecycle_service_client::RateLimitLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
        "key-ring-workflow" => Box::new(KeyRingWorkflowSink {
            client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "key-rotation-workflow" => Box::new(KeyRotationWorkflowSink {
            client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(channel),
        }),
        _ => Box::new(FeatureFlagUpdatesNameLogOnly),
    };
    sink.apply(record).await
}
