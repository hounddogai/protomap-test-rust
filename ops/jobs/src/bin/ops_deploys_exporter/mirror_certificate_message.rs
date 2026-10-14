//! `mirror_certificate_message` reads
//! `acme.certificates.v1.CertificateValidationService/NormalizeCertificate` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_validation_service_client::CertificateValidationServiceClient::new(channel.clone());
    let reply = source
        .normalize_certificate(certificates::NormalizeCertificateRequest {
            context: Some(context.clone()),
            certificate_id: "default-mirror-certificate-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let certificate = reply.certificate.unwrap_or_default();
    let record = CertificateMessageRecord {
        message: outcome.message,
        correlation_id: result.correlation_id,
        certificate_id: certificate.certificate_id,
    };
    dispatch_certificate_message("certificate-calculation", record, channel, &context).await
}

/// What `mirror_certificate_message` keeps of each NormalizeCertificate response.
#[derive(Clone, Debug, Default)]
pub struct CertificateMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub certificate_id: String,
}

/// Where `mirror_certificate_message` hands each record.
trait CertificateMessageSink {
    fn send(&self, record: CertificateMessageRecord) -> Delivery;
}

/// Writes records to `acme.certificates.v1.CertificateCalculationService`.
struct CertificateCalculationSink {
    client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient<Channel>,
}

impl CertificateMessageSink for CertificateCalculationSink {
    fn send(&self, record: CertificateMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_certificate(certificates::SimulateCertificateRequest {
                    reason: record.certificate_id,
                    details: Some(certificates::simulate_certificate_request::Details {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.giftcards.v1.GiftCardCalculationService`.
struct GiftCardCalculationSink {
    client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient<Channel>,
}

impl CertificateMessageSink for GiftCardCalculationSink {
    fn send(&self, record: CertificateMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_gift_card_options(giftcards::CompareGiftCardOptionsRequest {
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.RateLimitQueryService`.
struct RateLimitQuerySink {
    client: ratelimiting::rate_limit_query_service_client::RateLimitQueryServiceClient<Channel>,
}

impl CertificateMessageSink for RateLimitQuerySink {
    fn send(&self, record: CertificateMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_rate_limits(ratelimiting::FindRateLimitsRequest {
                    filter: Some(ratelimiting::find_rate_limits_request::Filter {
                        query: record.message,
                        owner_id: record.certificate_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
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
struct CertificateMessageLogOnly;

impl CertificateMessageSink for CertificateMessageLogOnly {
    fn send(&self, record: CertificateMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_certificate_message(
    kind: &str,
    record: CertificateMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let certificate_calculation = CertificateCalculationSink {
        client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let gift_card_calculation = GiftCardCalculationSink {
        client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient::new(channel.clone()),
    };
    let rate_limit_query = RateLimitQuerySink {
        client: ratelimiting::rate_limit_query_service_client::RateLimitQueryServiceClient::new(channel),
    };
    let write: &dyn Fn(CertificateMessageRecord) -> Delivery = match kind {
        "certificate-calculation" => &|record| certificate_calculation.send(record),
        "gift-card-calculation" => &|record| gift_card_calculation.send(record),
        "rate-limit-query" => &|record| rate_limit_query.send(record),
        _ => &|record| CertificateMessageLogOnly.send(record),
    };
    write(record).await
}
