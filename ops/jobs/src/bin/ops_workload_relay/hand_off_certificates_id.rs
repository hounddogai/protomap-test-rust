//! `hand_off_certificates_id` reads
//! `acme.certificates.v1.CertificateCalculationService/RecalculateCertificates` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_calculation_service_client::CertificateCalculationServiceClient::new(channel.clone());
    let reply = source
        .recalculate_certificates(certificates::RecalculateCertificatesRequest {
            context: Some(context.clone()),
            certificate_id: "nightly-hand-off-certificates-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let certificate = reply.certificate.unwrap_or_default();
    let breakdown = reply.breakdown.unwrap_or_default();
    let record = OffCertificatesIdRecord {
        certificate_id: certificate.certificate_id,
        currency_code: breakdown.currency_code,
        number: certificate.number,
        notes: certificate.notes,
    };
    let sinks: Vec<(&str, Box<dyn OffCertificatesIdDestination>)> = vec![
        (
            "sponsor-query",
            Box::new(SponsorQuerySink {
                client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "audit-event-validation",
            Box::new(AuditEventValidationSink {
                client: audit::audit_event_validation_service_client::AuditEventValidationServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "audit-event-validation" {
            continue;
        }
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `hand_off_certificates_id` keeps of each RecalculateCertificates response.
#[derive(Clone, Debug, Default)]
pub struct OffCertificatesIdRecord {
    pub certificate_id: String,
    pub currency_code: String,
    pub number: String,
    pub notes: String,
}

/// Where `hand_off_certificates_id` hands each record.
trait OffCertificatesIdDestination {
    fn forward(&self, record: OffCertificatesIdRecord) -> Delivery;
}

/// Sends records to `acme.sponsorships.v1.SponsorQueryService`.
struct SponsorQuerySink {
    client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient<Channel>,
}

impl OffCertificatesIdDestination for SponsorQuerySink {
    fn forward(&self, record: OffCertificatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_sponsor(sponsorships::LookupSponsorRequest {
                    sponsor_id: record.notes,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.audit.v1.AuditEventValidationService`.
struct AuditEventValidationSink {
    client: audit::audit_event_validation_service_client::AuditEventValidationServiceClient<Channel>,
}

impl OffCertificatesIdDestination for AuditEventValidationSink {
    fn forward(&self, record: OffCertificatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_audit_event_rules(audit::GetAuditEventRulesRequest {
                    audit_event_id: record.certificate_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
