//! `sync_crypto_key_workflow_message` reads
//! `acme.keymanagement.v1.CryptoKeyWorkflowService/ResumeCryptoKeyWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(channel.clone());
    let fetched = source
        .resume_crypto_key_workflow(keymanagement::ResumeCryptoKeyWorkflowRequest {
            context: Some(context.clone()),
            reason: "primary-sync-crypto-key-workflow-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let crypto_key = fetched.crypto_key.unwrap_or_default();
    let record = CryptoKeyWorkflowMessageRecord {
        message: result.message,
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
        crypto_key_id: crypto_key.crypto_key_id,
    };
    let mut sinks: BTreeMap<String, Box<dyn CryptoKeyWorkflowMessageWriter>> = BTreeMap::new();
    sinks.insert(
        "permission-check-admin".to_owned(),
        Box::new(PermissionCheckAdminSink {
            client: authorization::permission_check_admin_service_client::PermissionCheckAdminServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "rate-limit-validation".to_owned(),
        Box::new(RateLimitValidationSink {
            client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "throttle-calculation".to_owned(),
        Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "forum".to_owned(),
        Box::new(ForumSink {
            client: community::forum_service_client::ForumServiceClient::new(channel),
        }),
    );
    let target = std::env::var("SYNC_CRYPTO_KEY_WORKFLOW_MESSAGE_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.apply(record).await,
        None => Ok(()),
    }
}

/// What `sync_crypto_key_workflow_message` keeps of each ResumeCryptoKeyWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyWorkflowMessageRecord {
    pub message: String,
    pub reference_id: String,
    pub crypto_key_id: String,
}

/// Where `sync_crypto_key_workflow_message` hands each record.
trait CryptoKeyWorkflowMessageWriter {
    fn apply(&self, record: CryptoKeyWorkflowMessageRecord) -> Delivery;
}

/// Hands records to `acme.authorization.v1.PermissionCheckAdminService`.
struct PermissionCheckAdminSink {
    client: authorization::permission_check_admin_service_client::PermissionCheckAdminServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyWorkflowMessageWriter for PermissionCheckAdminSink {
    fn apply(&self, record: CryptoKeyWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::UnlockPermissionCheckRequest {
                    details: Some(authorization::unlock_permission_check_request::Details {
                        reason_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.unlock_permission_check(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.RateLimitValidationService`.
struct RateLimitValidationSink {
    client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient<Channel>,
}

impl CryptoKeyWorkflowMessageWriter for RateLimitValidationSink {
    fn apply(&self, record: CryptoKeyWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_rate_limit_decision(ratelimiting::ExplainRateLimitDecisionRequest {
                    reason: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
}

impl CryptoKeyWorkflowMessageWriter for ThrottleCalculationSink {
    fn apply(&self, record: CryptoKeyWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_throttle_options(ratelimiting::CompareThrottleOptionsRequest {
                    throttle: Some(ratelimiting::Throttle {
                        kind: record.reference_id,
                        source: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.community.v1.ForumService`.
struct ForumSink {
    client: community::forum_service_client::ForumServiceClient<Channel>,
}

impl CryptoKeyWorkflowMessageWriter for ForumSink {
    fn apply(&self, record: CryptoKeyWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_forum(community::GetForumRequest {
                    forum_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
