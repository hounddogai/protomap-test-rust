//! `forward_certificate_authority_id` reads
//! `acme.certificates.v1.CertificateAuthorityCalculationService/SimulateCertificateAuthority` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient::new(channel.clone());
    let fetched = source
        .simulate_certificate_authority(certificates::SimulateCertificateAuthorityRequest {
            context: Some(context.clone()),
            certificate_authority_id: "nightly-forward-certificate-authority-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let certificate_authority = fetched.certificate_authority.unwrap_or_default();
    let breakdown = fetched.breakdown.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = CertificateAuthorityIdRecord {
        certificate_authority_id: certificate_authority.certificate_authority_id,
        parent_id: certificate_authority.parent_id,
        currency_code: breakdown.currency_code,
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<(&str, Box<dyn CertificateAuthorityIdWriter>)> = vec![
        (
            "delivery-query",
            Box::new(DeliveryQuerySink {
                client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel.clone()),
                context: context.clone(),
            }),
        ),
        (
            "secret-version-validation",
            Box::new(SecretVersionValidationSink {
                client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "document-query",
            Box::new(DocumentQuerySink {
                client: documents::document_query_service_client::DocumentQueryServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "secret-version-validation" {
            continue;
        }
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `forward_certificate_authority_id` keeps of each SimulateCertificateAuthority response.
#[derive(Clone, Debug, Default)]
pub struct CertificateAuthorityIdRecord {
    pub certificate_authority_id: String,
    pub parent_id: String,
    pub currency_code: String,
    pub correlation_id: String,
}

/// Where `forward_certificate_authority_id` hands each record.
trait CertificateAuthorityIdWriter {
    fn send(&self, record: CertificateAuthorityIdRecord) -> Delivery;
}

/// Hands records to `acme.deliveries.v1.DeliveryQueryService`.
struct DeliveryQuerySink {
    client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl CertificateAuthorityIdWriter for DeliveryQuerySink {
    fn send(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(deliveries::SummarizeDeliveriesRequest {
                    details: Some(deliveries::summarize_deliveries_request::Details {
                        cron: record.correlation_id,
                        format: record.parent_id,
                        ..Default::default()
                    }),
                    reason: record.currency_code,
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_deliveries(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.secrets.v1.SecretVersionValidationService`.
struct SecretVersionValidationSink {
    client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient<Channel>,
}

impl CertificateAuthorityIdWriter for SecretVersionValidationSink {
    fn send(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .validate_secret_version(secrets::ValidateSecretVersionRequest {
                    secret_version: Some(secrets::SecretVersion {
                        source: record.correlation_id,
                        correlation_id: record.certificate_authority_id,
                        kind: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.documents.v1.DocumentQueryService`.
struct DocumentQuerySink {
    client: documents::document_query_service_client::DocumentQueryServiceClient<Channel>,
}

impl CertificateAuthorityIdWriter for DocumentQuerySink {
    fn send(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_documents(documents::BrowseDocumentsRequest {
                    order_by: record.parent_id,
                    filter: Some(documents::browse_documents_request::Filter {
                        query: record.certificate_authority_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
