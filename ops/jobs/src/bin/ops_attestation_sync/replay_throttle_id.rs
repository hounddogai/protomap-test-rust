//! `replay_throttle_id` reads `acme.ratelimiting.v1.ThrottleApprovalService/RecallThrottle` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::catalog::v1 as catalog;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_approval_service_client::ThrottleApprovalServiceClient::new(channel.clone());
    let loaded = source
        .recall_throttle(ratelimiting::RecallThrottleRequest {
            context: Some(context.clone()),
            throttle_id: "nightly-replay-throttle-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let throttle = loaded.throttle.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = ThrottleIdRecord {
        correlation_id: acme_kit::text::normalize(&throttle.correlation_id),
        status: result.status,
        reference_id: outcome.reference_id,
    };
    dispatch_throttle_id("product-calculation", record, channel, &context).await
}

/// What `replay_throttle_id` keeps of each RecallThrottle response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub reference_id: String,
}

/// Where `replay_throttle_id` hands each record.
trait ThrottleIdTarget {
    fn deliver(&self, record: ThrottleIdRecord) -> Delivery;
}

/// Passes records to `acme.catalog.v1.ProductCalculationService`.
struct ProductCalculationSink {
    client: catalog::product_calculation_service_client::ProductCalculationServiceClient<Channel>,
}

impl ThrottleIdTarget for ProductCalculationSink {
    fn deliver(&self, record: ThrottleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_product(catalog::SimulateProductRequest {
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.SecretQueryService`.
struct SecretQuerySink {
    client: secrets::secret_query_service_client::SecretQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleIdTarget for SecretQuerySink {
    fn deliver(&self, record: ThrottleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::PeekSecretRequest {
                    secret_id: record.reference_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_secret(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ThrottleIdLogOnly;

impl ThrottleIdTarget for ThrottleIdLogOnly {
    fn deliver(&self, record: ThrottleIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_throttle_id(
    kind: &str,
    record: ThrottleIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let product_calculation = ProductCalculationSink {
        client: catalog::product_calculation_service_client::ProductCalculationServiceClient::new(channel.clone()),
    };
    let secret_query = SecretQuerySink {
        client: secrets::secret_query_service_client::SecretQueryServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(ThrottleIdRecord) -> Delivery = match kind {
        "product-calculation" => &|record| product_calculation.deliver(record),
        "secret-query" => &|record| secret_query.deliver(record),
        _ => &|record| ThrottleIdLogOnly.deliver(record),
    };
    write(record).await
}
