//! `hand_off_key_ring_code` reads `acme.keymanagement.v1.KeyRingCalculationService/CalculateKeyRing` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_ring_calculation_service_client::KeyRingCalculationServiceClient::new(channel.clone());
    let found = source
        .calculate_key_ring(keymanagement::CalculateKeyRingRequest {
            context: Some(context.clone()),
            reason: "primary-hand-off-key-ring-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = found.breakdown.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = OffKeyRingCodeRecord {
        currency_code: acme_kit::text::normalize(&breakdown.currency_code),
        message: acme_kit::text::normalize(&result.message),
    };
    dispatch_off_key_ring_code("csr-lifecycle", record, channel, &context).await
}

/// What `hand_off_key_ring_code` keeps of each CalculateKeyRing response.
#[derive(Clone, Debug, Default)]
pub struct OffKeyRingCodeRecord {
    pub currency_code: String,
    pub message: String,
}

/// Where `hand_off_key_ring_code` hands each record.
trait OffKeyRingCodeTarget {
    fn forward(&self, record: OffKeyRingCodeRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.RateLimitReportingService`.
struct RateLimitReportingSink {
    client: ratelimiting::rate_limit_reporting_service_client::RateLimitReportingServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffKeyRingCodeTarget for RateLimitReportingSink {
    fn forward(&self, record: OffKeyRingCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ListRateLimitReportsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_rate_limit_reports(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.certificates.v1.CsrLifecycleService`.
struct CsrLifecycleSink {
    client: certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffKeyRingCodeTarget for CsrLifecycleSink {
    fn forward(&self, record: OffKeyRingCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::SuspendCsrRequest {
                    details: Some(certificates::suspend_csr_request::Details {
                        reason_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suspend_csr(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct OffKeyRingCodeLogOnly;

impl OffKeyRingCodeTarget for OffKeyRingCodeLogOnly {
    fn forward(&self, record: OffKeyRingCodeRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.currency_code);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_off_key_ring_code(
    kind: &str,
    record: OffKeyRingCodeRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let rate_limit_reporting = RateLimitReportingSink {
        client: ratelimiting::rate_limit_reporting_service_client::RateLimitReportingServiceClient::new(
            channel.clone(),
        ),
        context: context.clone(),
    };
    let csr_lifecycle = CsrLifecycleSink {
        client: certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(OffKeyRingCodeRecord) -> Delivery = match kind {
        "rate-limit-reporting" => &|record| rate_limit_reporting.forward(record),
        "csr-lifecycle" => &|record| csr_lifecycle.forward(record),
        _ => &|record| OffKeyRingCodeLogOnly.forward(record),
    };
    write(record).await
}
