//! `propagate_secret_version_workflow_status` reads
//! `acme.secrets.v1.SecretVersionWorkflowService/ResumeSecretVersionWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient::new(channel.clone());
    let loaded = source
        .resume_secret_version_workflow(secrets::ResumeSecretVersionWorkflowRequest {
            context: Some(context.clone()),
            secret_version_id: "primary-propagate-secret-version-workflow-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = loaded.outcome.unwrap_or_default();
    let secret_version = loaded.secret_version.unwrap_or_default();
    let record = SecretVersionWorkflowStatusRecord {
        status: outcome.status,
        kind: secret_version.kind,
        correlation_id: secret_version.correlation_id,
        source: secret_version.source,
    };
    deliver_secret_version_workflow_status("config-set-lifecycle", record, channel, &context).await
}

/// What `propagate_secret_version_workflow_status` keeps of each ResumeSecretVersionWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct SecretVersionWorkflowStatusRecord {
    pub status: String,
    pub kind: String,
    pub correlation_id: String,
    pub source: String,
}

/// Where `propagate_secret_version_workflow_status` hands each record.
trait SecretVersionWorkflowStatusTarget {
    fn push(&self, record: SecretVersionWorkflowStatusRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigSetLifecycleService`.
struct ConfigSetLifecycleSink {
    client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient<Channel>,
}

impl SecretVersionWorkflowStatusTarget for ConfigSetLifecycleSink {
    fn push(&self, record: SecretVersionWorkflowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_config_set(configuration::RenewConfigSetRequest {
                    details: Some(configuration::renew_config_set_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    config_set_id: record.source,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.RateLimitSyncService`.
struct RateLimitSyncSink {
    client: ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient<Channel>,
}

impl SecretVersionWorkflowStatusTarget for RateLimitSyncSink {
    fn push(&self, record: SecretVersionWorkflowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reconcile_rate_limits(ratelimiting::ReconcileRateLimitsRequest {
                    details: Some(ratelimiting::reconcile_rate_limits_request::Details {
                        cursor: record.source,
                        peer_region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.secrets.v1.AccessGrantApprovalService`.
struct AccessGrantApprovalSink {
    client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient<Channel>,
}

impl SecretVersionWorkflowStatusTarget for AccessGrantApprovalSink {
    fn push(&self, record: SecretVersionWorkflowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reject_access_grant(secrets::RejectAccessGrantRequest {
                    access_grant: Some(secrets::AccessGrant {
                        name: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.certificates.v1.CertificateAuthorityValidationService`.
struct CertificateAuthorityValidationSink {
    client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient<
        Channel,
    >,
}

impl SecretVersionWorkflowStatusTarget for CertificateAuthorityValidationSink {
    fn push(&self, record: SecretVersionWorkflowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_certificate_authority_rules(certificates::GetCertificateAuthorityRulesRequest {
                    certificate_authority_id: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct SecretVersionWorkflowStatusLogOnly;

impl SecretVersionWorkflowStatusTarget for SecretVersionWorkflowStatusLogOnly {
    fn push(&self, record: SecretVersionWorkflowStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_secret_version_workflow_status(
    kind: &str,
    record: SecretVersionWorkflowStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn SecretVersionWorkflowStatusTarget> = match kind {
        "config-set-lifecycle" => Box::new(ConfigSetLifecycleSink { client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(channel.clone()) }),
        "rate-limit-sync" => Box::new(RateLimitSyncSink { client: ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient::new(channel.clone()) }),
        "access-grant-approval" => Box::new(AccessGrantApprovalSink { client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient::new(channel.clone()) }),
        "certificate-authority-validation" => Box::new(CertificateAuthorityValidationSink { client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient::new(channel) }),
        _ => Box::new(SecretVersionWorkflowStatusLogOnly),
    };
    sink.push(record).await
}
