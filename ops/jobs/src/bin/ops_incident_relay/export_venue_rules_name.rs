//! `export_venue_rules_name` reads `acme.venues.v1.VenueValidationService/GetVenueRules` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = venues::venue_validation_service_client::VenueValidationServiceClient::new(channel.clone());
    let reply = source
        .get_venue_rules(venues::GetVenueRulesRequest {
            context: Some(context.clone()),
            venue_id: "default-export-venue-rules-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let venue = reply.venue.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = VenueRulesNameRecord {
        name: venue.name,
        status: result.status,
        code: acme_kit::text::normalize(&venue.code),
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    let sinks: Vec<(&str, Box<dyn VenueRulesNameTarget>)> = vec![
        ("certificate-authority-workflow", Box::new(CertificateAuthorityWorkflowSink { client: certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient::new(channel.clone()) })),
        ("rate-limit-query", Box::new(RateLimitQuerySink { client: ratelimiting::rate_limit_query_service_client::RateLimitQueryServiceClient::new(channel.clone()) })),
        ("rate-limit-validation", Box::new(RateLimitValidationSink { client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(channel) })),
    ];
    for (kind, sink) in &sinks {
        if *kind != "rate-limit-validation" {
            continue;
        }
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `export_venue_rules_name` keeps of each GetVenueRules response.
#[derive(Clone, Debug, Default)]
pub struct VenueRulesNameRecord {
    pub name: String,
    pub status: String,
    pub code: String,
    pub correlation_id: String,
}

/// Where `export_venue_rules_name` hands each record.
trait VenueRulesNameTarget {
    fn send(&self, record: VenueRulesNameRecord) -> Delivery;
}

/// Writes records to `acme.certificates.v1.CertificateAuthorityWorkflowService`.
struct CertificateAuthorityWorkflowSink {
    client:
        certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient<Channel>,
}

impl VenueRulesNameTarget for CertificateAuthorityWorkflowSink {
    fn send(&self, record: VenueRulesNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_certificate_authority_workflow(certificates::RetryCertificateAuthorityWorkflowRequest {
                    details: Some(certificates::retry_certificate_authority_workflow_request::Details {
                        step: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.RateLimitQueryService`.
struct RateLimitQuerySink {
    client: ratelimiting::rate_limit_query_service_client::RateLimitQueryServiceClient<Channel>,
}

impl VenueRulesNameTarget for RateLimitQuerySink {
    fn send(&self, record: VenueRulesNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_rate_limit(ratelimiting::LookupRateLimitRequest {
                    rate_limit_id: record.code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.RateLimitValidationService`.
struct RateLimitValidationSink {
    client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient<Channel>,
}

impl VenueRulesNameTarget for RateLimitValidationSink {
    fn send(&self, record: VenueRulesNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_rate_limit_decision(ratelimiting::ExplainRateLimitDecisionRequest {
                    rate_limit_id: record.status,
                    rate_limit: Some(ratelimiting::RateLimit {
                        rate_limit_id: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
