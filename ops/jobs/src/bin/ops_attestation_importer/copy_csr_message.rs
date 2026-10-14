//! `copy_csr_message` reads `acme.certificates.v1.CsrCalculationService/SimulateCsr` and hands what it keeps
//! to `acme.ratelimiting.v1.ThrottleLifecycleService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::csr_calculation_service_client::CsrCalculationServiceClient::new(channel.clone());
    let found = source
        .simulate_csr(certificates::SimulateCsrRequest {
            context: Some(context.clone()),
            csr_id: "nightly-copy-csr-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let breakdown = found.breakdown.unwrap_or_default();
    let record = CsrMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
        currency_code: acme_kit::text::normalize(&breakdown.currency_code),
    };
    ThrottleLifecycleSink {
        client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `copy_csr_message` keeps of each SimulateCsr response.
#[derive(Clone, Debug, Default)]
pub struct CsrMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub currency_code: String,
}

/// Where `copy_csr_message` hands each record.
trait CsrMessageDestination {
    fn forward(&self, record: CsrMessageRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.ThrottleLifecycleService`.
struct ThrottleLifecycleSink {
    client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient<Channel>,
}

impl CsrMessageDestination for ThrottleLifecycleSink {
    fn forward(&self, record: CsrMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suspend_throttle(ratelimiting::SuspendThrottleRequest {
                    details: Some(ratelimiting::suspend_throttle_request::Details {
                        reason_code: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.message,
                    throttle_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
