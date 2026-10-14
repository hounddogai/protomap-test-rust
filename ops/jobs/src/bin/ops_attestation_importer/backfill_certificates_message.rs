//! `backfill_certificates_message` reads `acme.certificates.v1.CertificateQueryService/CountCertificates` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::promotions::v1 as promotions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_query_service_client::CertificateQueryServiceClient::new(channel.clone());
    let fetched = source
        .count_certificates(certificates::CountCertificatesRequest {
            context: Some(context.clone()),
            order_by: "nightly-backfill-certificates-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = CertificatesMessageRecord {
        message: result.message,
        status: acme_kit::text::normalize(&result.status),
    };
    let sinks: Vec<Box<dyn CertificatesMessageWriter>> = vec![
        Box::new(PromotionCalculationSink { client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient::new(channel.clone()), context: context.clone() }),
        Box::new(CertificateAuthorityWorkflowSink { client: certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient::new(channel) }),
    ];
    sinks[1].send(record).await
}

/// What `backfill_certificates_message` keeps of each CountCertificates response.
#[derive(Clone, Debug, Default)]
pub struct CertificatesMessageRecord {
    pub message: String,
    pub status: String,
}

/// Where `backfill_certificates_message` hands each record.
trait CertificatesMessageWriter {
    fn send(&self, record: CertificatesMessageRecord) -> Delivery;
}

/// Sends records to `acme.promotions.v1.PromotionCalculationService`.
struct PromotionCalculationSink {
    client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CertificatesMessageWriter for PromotionCalculationSink {
    fn send(&self, record: CertificatesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(promotions::QuotePromotionRequest {
                    promotion: Some(promotions::Promotion {
                        name: record.message,
                        promotion_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.quote_promotion(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CertificateAuthorityWorkflowService`.
struct CertificateAuthorityWorkflowSink {
    client:
        certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient<Channel>,
}

impl CertificatesMessageWriter for CertificateAuthorityWorkflowSink {
    fn send(&self, record: CertificatesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_certificate_authority_workflow(certificates::StartCertificateAuthorityWorkflowRequest {
                    details: Some(certificates::start_certificate_authority_workflow_request::Details {
                        step: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
