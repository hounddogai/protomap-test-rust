//! `copy_rate_limits_status` reads `acme.ratelimiting.v1.RateLimitSyncService/PullRateLimits` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient::new(channel.clone());
    let found = source
        .pull_rate_limits(ratelimiting::PullRateLimitsRequest {
            context: Some(context.clone()),
            reason: "default-copy-rate-limits-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let rate_limit = found.rate_limit.unwrap_or_default();
    let record = RateLimitsStatusRecord {
        status: outcome.status,
        rate_limit_id: rate_limit.rate_limit_id,
    };
    deliver_rate_limits_status("feature-flag-admin", record, channel, &context).await
}

/// What `copy_rate_limits_status` keeps of each PullRateLimits response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitsStatusRecord {
    pub status: String,
    pub rate_limit_id: String,
}

/// Where `copy_rate_limits_status` hands each record.
trait RateLimitsStatusSink {
    fn forward(&self, record: RateLimitsStatusRecord) -> Delivery;
}

/// Passes records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
}

impl RateLimitsStatusSink for AccessGrantValidationSink {
    fn forward(&self, record: RateLimitsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .validate_access_grant(secrets::ValidateAccessGrantRequest {
                    details: Some(secrets::validate_access_grant_request::Details {
                        locale: record.rate_limit_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FeatureFlagAdminService`.
struct FeatureFlagAdminSink {
    client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient<Channel>,
}

impl RateLimitsStatusSink for FeatureFlagAdminSink {
    fn forward(&self, record: RateLimitsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .archive_feature_flag(featureflags::ArchiveFeatureFlagRequest {
                    details: Some(featureflags::archive_feature_flag_request::Details {
                        reason_code: record.rate_limit_id,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.EnvironmentApprovalService`.
struct EnvironmentApprovalSink {
    client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient<Channel>,
}

impl RateLimitsStatusSink for EnvironmentApprovalSink {
    fn forward(&self, record: RateLimitsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .comment_environment_approval(configuration::CommentEnvironmentApprovalRequest {
                    environment: Some(configuration::Environment {
                        name: record.status,
                        ..Default::default()
                    }),
                    environment_id: record.rate_limit_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct RateLimitsStatusLogOnly;

impl RateLimitsStatusSink for RateLimitsStatusLogOnly {
    fn forward(&self, record: RateLimitsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_rate_limits_status(
    kind: &str,
    record: RateLimitsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn RateLimitsStatusSink> = match kind {
        "access-grant-validation" => Box::new(AccessGrantValidationSink {
            client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        "feature-flag-admin" => Box::new(FeatureFlagAdminSink {
            client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient::new(
                channel.clone(),
            ),
        }),
        "environment-approval" => Box::new(EnvironmentApprovalSink {
            client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient::new(channel),
        }),
        _ => Box::new(RateLimitsStatusLogOnly),
    };
    sink.forward(record).await
}
