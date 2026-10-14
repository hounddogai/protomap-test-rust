//! `push_secret_approval_status_id` reads `acme.secrets.v1.SecretApprovalService/GetSecretApprovalStatus` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = secrets::secret_approval_service_client::SecretApprovalServiceClient::new(channel.clone());
    let reply = source
        .get_secret_approval_status(secrets::GetSecretApprovalStatusRequest {
            context: Some(context.clone()),
            secret_id: "live-push-secret-approval-status-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let secret = reply.secret.unwrap_or_default();
    let record = SecretApprovalStatusIdRecord {
        correlation_id: result.correlation_id,
        currency_code: acme_kit::text::normalize(&secret.currency_code),
        secret_id: secret.secret_id,
        message: result.message,
    };
    let sinks: Vec<(&str, Box<dyn SecretApprovalStatusIdSink>)> = vec![
        (
            "quota-bucket-validation",
            Box::new(QuotaBucketValidationSink {
                client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "key-ring-lifecycle",
            Box::new(KeyRingLifecycleSink {
                client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "secret-version-validation",
            Box::new(SecretVersionValidationSink {
                client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "quota-bucket-validation" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `push_secret_approval_status_id` keeps of each GetSecretApprovalStatus response.
#[derive(Clone, Debug, Default)]
pub struct SecretApprovalStatusIdRecord {
    pub correlation_id: String,
    pub currency_code: String,
    pub secret_id: String,
    pub message: String,
}

/// Where `push_secret_approval_status_id` hands each record.
trait SecretApprovalStatusIdSink {
    fn deliver(&self, record: SecretApprovalStatusIdRecord) -> Delivery;
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketValidationService`.
struct QuotaBucketValidationSink {
    client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient<Channel>,
}

impl SecretApprovalStatusIdSink for QuotaBucketValidationSink {
    fn deliver(&self, record: SecretApprovalStatusIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_quota_buckets(ratelimiting::LintQuotaBucketsRequest {
                    quota_bucket_id: record.correlation_id,
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        currency_code: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.KeyRingLifecycleService`.
struct KeyRingLifecycleSink {
    client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl SecretApprovalStatusIdSink for KeyRingLifecycleSink {
    fn deliver(&self, record: SecretApprovalStatusIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::SuspendKeyRingRequest {
                    key_ring_id: record.message,
                    details: Some(keymanagement::suspend_key_ring_request::Details {
                        reason_code: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suspend_key_ring(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.SecretVersionValidationService`.
struct SecretVersionValidationSink {
    client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient<Channel>,
}

impl SecretApprovalStatusIdSink for SecretVersionValidationSink {
    fn deliver(&self, record: SecretApprovalStatusIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_secret_version_decision(secrets::ExplainSecretVersionDecisionRequest {
                    secret_version: Some(secrets::SecretVersion {
                        kind: record.currency_code,
                        ..Default::default()
                    }),
                    secret_version_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
