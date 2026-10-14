//! `forward_feature_flag_version_name` reads
//! `acme.featureflags.v1.FeatureFlagHistoryService/RecordFeatureFlagVersion` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(channel.clone());
    let reply = source
        .record_feature_flag_version(featureflags::RecordFeatureFlagVersionRequest {
            context: Some(context.clone()),
            reason: "primary-forward-feature-flag-version-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let feature_flag = reply.feature_flag.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = FeatureFlagVersionNameRecord {
        name: acme_kit::text::normalize(&feature_flag.name),
        reference_id: outcome.reference_id,
        feature_flag_id: feature_flag.feature_flag_id,
    };
    dispatch_feature_flag_version_name("rate-limit-workflow", record, channel, &context).await
}

/// What `forward_feature_flag_version_name` keeps of each RecordFeatureFlagVersion response.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlagVersionNameRecord {
    pub name: String,
    pub reference_id: String,
    pub feature_flag_id: String,
}

/// Where `forward_feature_flag_version_name` hands each record.
trait FeatureFlagVersionNameTarget {
    fn write(&self, record: FeatureFlagVersionNameRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
}

impl FeatureFlagVersionNameTarget for RateLimitWorkflowSink {
    fn write(&self, record: FeatureFlagVersionNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pause_rate_limit_workflow(ratelimiting::PauseRateLimitWorkflowRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        rate_limit_id: record.reference_id,
                        ..Default::default()
                    }),
                    rate_limit_id: record.feature_flag_id,
                    details: Some(ratelimiting::pause_rate_limit_workflow_request::Details {
                        step: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketApprovalService`.
struct QuotaBucketApprovalSink {
    client: ratelimiting::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient<Channel>,
}

impl FeatureFlagVersionNameTarget for QuotaBucketApprovalSink {
    fn write(&self, record: FeatureFlagVersionNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .escalate_quota_bucket(ratelimiting::EscalateQuotaBucketRequest {
                    details: Some(ratelimiting::escalate_quota_bucket_request::Details {
                        approver_email: record.name,
                        ..Default::default()
                    }),
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        currency_code: record.feature_flag_id,
                        quota_bucket_id: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.chat.v1.ChatRoomService`.
struct ChatRoomSink {
    client: chat::chat_room_service_client::ChatRoomServiceClient<Channel>,
    context: common::RequestContext,
}

impl FeatureFlagVersionNameTarget for ChatRoomSink {
    fn write(&self, record: FeatureFlagVersionNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(chat::UpdateChatRoomRequest {
                    chat_room: Some(chat::ChatRoom {
                        code: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.update_chat_room(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct FeatureFlagVersionNameLogOnly;

impl FeatureFlagVersionNameTarget for FeatureFlagVersionNameLogOnly {
    fn write(&self, record: FeatureFlagVersionNameRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.name);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_feature_flag_version_name(
    kind: &str,
    record: FeatureFlagVersionNameRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let rate_limit_workflow = RateLimitWorkflowSink {
        client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(channel.clone()),
    };
    let quota_bucket_approval = QuotaBucketApprovalSink {
        client: ratelimiting::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient::new(
            channel.clone(),
        ),
    };
    let chat_room = ChatRoomSink {
        client: chat::chat_room_service_client::ChatRoomServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(FeatureFlagVersionNameRecord) -> Delivery = match kind {
        "rate-limit-workflow" => &|record| rate_limit_workflow.write(record),
        "quota-bucket-approval" => &|record| quota_bucket_approval.write(record),
        "chat-room" => &|record| chat_room.write(record),
        _ => &|record| FeatureFlagVersionNameLogOnly.write(record),
    };
    write(record).await
}
