//! `copy_certificates_status` reads `acme.certificates.v1.CertificateValidationService/LintCertificates` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_validation_service_client::CertificateValidationServiceClient::new(channel.clone());
    let reply = source
        .lint_certificates(certificates::LintCertificatesRequest {
            context: Some(context.clone()),
            certificate_id: "current-copy-certificates-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = CertificatesStatusRecord {
        status: result.status,
        message: result.message,
        reference_id: outcome.reference_id,
    };
    deliver_certificates_status("throttle-calculation", record, channel, &context).await
}

/// What `copy_certificates_status` keeps of each LintCertificates response.
#[derive(Clone, Debug, Default)]
pub struct CertificatesStatusRecord {
    pub status: String,
    pub message: String,
    pub reference_id: String,
}

/// Where `copy_certificates_status` hands each record.
trait CertificatesStatusDestination {
    fn apply(&self, record: CertificatesStatusRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
}

impl CertificatesStatusDestination for ThrottleCalculationSink {
    fn apply(&self, record: CertificatesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_throttle_rates(ratelimiting::GetThrottleRatesRequest {
                    throttle_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.SecretVersionWorkflowService`.
struct SecretVersionWorkflowSink {
    client: secrets::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient<Channel>,
}

impl CertificatesStatusDestination for SecretVersionWorkflowSink {
    fn apply(&self, record: CertificatesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_secret_version_workflow(secrets::RetrySecretVersionWorkflowRequest {
                    secret_version: Some(secrets::SecretVersion {
                        secret_version_id: record.status,
                        source: record.reference_id,
                        ..Default::default()
                    }),
                    details: Some(secrets::retry_secret_version_workflow_request::Details {
                        initiated_by: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct CertificatesStatusLogOnly;

impl CertificatesStatusDestination for CertificatesStatusLogOnly {
    fn apply(&self, record: CertificatesStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_certificates_status(
    kind: &str,
    record: CertificatesStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn CertificatesStatusDestination> = match kind {
        "throttle-calculation" => Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        "secret-version-workflow" => Box::new(SecretVersionWorkflowSink {
            client: secrets::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient::new(channel),
        }),
        _ => Box::new(CertificatesStatusLogOnly),
    };
    sink.apply(record).await
}
