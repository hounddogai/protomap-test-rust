//! `propagate_rate_limit_status` reads `acme.ratelimiting.v1.RateLimitSyncService/PushRateLimit` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient::new(channel.clone());
    let fetched = source
        .push_rate_limit(ratelimiting::PushRateLimitRequest {
            context: Some(context.clone()),
            rate_limit_id: "nightly-propagate-rate-limit-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = RateLimitStatusRecord {
        status: result.status,
        outcome_status: outcome.status,
        message: result.message,
    };
    let sinks: Vec<Box<dyn RateLimitStatusDestination>> = vec![
        Box::new(ConfigChangeApprovalSink {
            client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(EnvironmentValidationSink {
            client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(RateLimitLifecycleSink {
            client: ratelimiting::rate_limit_lifecycle_service_client::RateLimitLifecycleServiceClient::new(channel),
        }),
    ];
    sinks[2].push(record).await
}

/// What `propagate_rate_limit_status` keeps of each PushRateLimit response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitStatusRecord {
    pub status: String,
    pub outcome_status: String,
    pub message: String,
}

/// Where `propagate_rate_limit_status` hands each record.
trait RateLimitStatusDestination {
    fn push(&self, record: RateLimitStatusRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.ConfigChangeApprovalService`.
struct ConfigChangeApprovalSink {
    client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient<Channel>,
}

impl RateLimitStatusDestination for ConfigChangeApprovalSink {
    fn push(&self, record: RateLimitStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .submit_config_change(configuration::SubmitConfigChangeRequest {
                    details: Some(configuration::submit_config_change_request::Details {
                        approver_email: record.outcome_status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.EnvironmentValidationService`.
struct EnvironmentValidationSink {
    client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient<Channel>,
}

impl RateLimitStatusDestination for EnvironmentValidationSink {
    fn push(&self, record: RateLimitStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_environment(configuration::ScoreEnvironmentRequest {
                    environment: Some(configuration::Environment {
                        name: record.message,
                        ..Default::default()
                    }),
                    environment_id: record.outcome_status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.RateLimitLifecycleService`.
struct RateLimitLifecycleSink {
    client: ratelimiting::rate_limit_lifecycle_service_client::RateLimitLifecycleServiceClient<Channel>,
}

impl RateLimitStatusDestination for RateLimitLifecycleSink {
    fn push(&self, record: RateLimitStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_rate_limit_lifecycle(ratelimiting::GetRateLimitLifecycleRequest {
                    rate_limit_id: record.outcome_status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
