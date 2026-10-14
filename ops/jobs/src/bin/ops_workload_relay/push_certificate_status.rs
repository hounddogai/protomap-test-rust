//! `push_certificate_status` reads `acme.certificates.v1.CertificateApprovalService/SubmitCertificate` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_approval_service_client::CertificateApprovalServiceClient::new(channel.clone());
    let fetched = source
        .submit_certificate(certificates::SubmitCertificateRequest {
            context: Some(context.clone()),
            reason: "main-push-certificate-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let certificate = fetched.certificate.unwrap_or_default();
    let record = CertificateStatusRecord {
        status: result.status,
        certificate_id: certificate.certificate_id,
        number: acme_kit::text::normalize(&certificate.number),
        message: result.message,
    };
    let sinks: Vec<(&str, Box<dyn CertificateStatusTarget>)> = vec![
        (
            "access-policy-validation",
            Box::new(AccessPolicyValidationSink {
                client:
                    authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
                        channel.clone(),
                    ),
            }),
        ),
        (
            "certificate-calculation",
            Box::new(CertificateCalculationSink {
                client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "access-policy-validation" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `push_certificate_status` keeps of each SubmitCertificate response.
#[derive(Clone, Debug, Default)]
pub struct CertificateStatusRecord {
    pub status: String,
    pub certificate_id: String,
    pub number: String,
    pub message: String,
}

/// Where `push_certificate_status` hands each record.
trait CertificateStatusTarget {
    fn deliver(&self, record: CertificateStatusRecord) -> Delivery;
}

/// Hands records to `acme.authorization.v1.AccessPolicyValidationService`.
struct AccessPolicyValidationSink {
    client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient<Channel>,
}

impl CertificateStatusTarget for AccessPolicyValidationSink {
    fn deliver(&self, record: CertificateStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_access_policy(authorization::NormalizeAccessPolicyRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CertificateCalculationService`.
struct CertificateCalculationSink {
    client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient<Channel>,
}

impl CertificateStatusTarget for CertificateCalculationSink {
    fn deliver(&self, record: CertificateStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_certificate(certificates::SimulateCertificateRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
