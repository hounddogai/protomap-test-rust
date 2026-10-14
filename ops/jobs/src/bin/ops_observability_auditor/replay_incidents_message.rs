//! `replay_incidents_message` reads `acme.itsm.v1.IncidentService/SearchIncidents` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::itsm::v1 as itsm;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = itsm::incident_service_client::IncidentServiceClient::new(channel.clone());
    let found = source
        .search_incidents(itsm::SearchIncidentsRequest {
            context: Some(context.clone()),
            order_by: "nightly-replay-incidents-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = IncidentsMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        next_page_token: page_info.next_page_token,
        status: acme_kit::text::normalize(&result.status),
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn IncidentsMessageWriter>> = vec![
        Box::new(QuotaBucketApprovalSink {
            client: ratelimiting::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(FlagRolloutValidationSink {
            client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(ThrottleValidationSink {
            client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient::new(channel),
        }),
    ];
    sinks[0].push(record).await
}

/// What `replay_incidents_message` keeps of each SearchIncidents response.
#[derive(Clone, Debug, Default)]
pub struct IncidentsMessageRecord {
    pub message: String,
    pub next_page_token: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `replay_incidents_message` hands each record.
trait IncidentsMessageWriter {
    fn push(&self, record: IncidentsMessageRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.QuotaBucketApprovalService`.
struct QuotaBucketApprovalSink {
    client: ratelimiting::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl IncidentsMessageWriter for QuotaBucketApprovalSink {
    fn push(&self, record: IncidentsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::GetQuotaBucketApprovalStatusRequest {
                    quota_bucket_id: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_quota_bucket_approval_status(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagRolloutValidationService`.
struct FlagRolloutValidationSink {
    client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient<Channel>,
}

impl IncidentsMessageWriter for FlagRolloutValidationSink {
    fn push(&self, record: IncidentsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_flag_rollout(featureflags::NormalizeFlagRolloutRequest {
                    flag_rollout_id: record.message,
                    flag_rollout: Some(featureflags::FlagRollout {
                        flag_rollout_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleValidationService`.
struct ThrottleValidationSink {
    client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient<Channel>,
}

impl IncidentsMessageWriter for ThrottleValidationSink {
    fn push(&self, record: IncidentsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_throttle(ratelimiting::ScoreThrottleRequest {
                    details: Some(ratelimiting::score_throttle_request::Details {
                        currency_code: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
