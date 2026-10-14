//! `copy_quota_bucket_totals_id` reads
//! `acme.ratelimiting.v1.QuotaBucketReportingService/CountQuotaBucketTotals` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::payroll::v1 as payroll;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient::new(channel.clone());
    let fetched = source
        .count_quota_bucket_totals(ratelimiting::CountQuotaBucketTotalsRequest {
            context: Some(context.clone()),
            order_by: "primary-copy-quota-bucket-totals-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = QuotaBucketTotalsIdRecord {
        correlation_id: result.correlation_id,
        status: acme_kit::text::normalize(&result.status),
    };
    let mut sinks: BTreeMap<&str, Box<dyn QuotaBucketTotalsIdDestination>> = BTreeMap::new();
    sinks.insert(
        "key-ring-query",
        Box::new(KeyRingQuerySink {
            client: keymanagement::key_ring_query_service_client::KeyRingQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "pay-run",
        Box::new(PayRunSink {
            client: payroll::pay_run_service_client::PayRunServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "api-key-query",
        Box::new(ApiKeyQuerySink {
            client: edgegateway::api_key_query_service_client::ApiKeyQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("pay-run") {
        Some(sink) => sink.apply(record).await,
        None => anyhow::bail!("no sink registered for pay-run"),
    }
}

/// What `copy_quota_bucket_totals_id` keeps of each CountQuotaBucketTotals response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketTotalsIdRecord {
    pub correlation_id: String,
    pub status: String,
}

/// Where `copy_quota_bucket_totals_id` hands each record.
trait QuotaBucketTotalsIdDestination {
    fn apply(&self, record: QuotaBucketTotalsIdRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRingQueryService`.
struct KeyRingQuerySink {
    client: keymanagement::key_ring_query_service_client::KeyRingQueryServiceClient<Channel>,
}

impl QuotaBucketTotalsIdDestination for KeyRingQuerySink {
    fn apply(&self, record: QuotaBucketTotalsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_key_ring(keymanagement::LookupKeyRingRequest {
                    key_ring_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.payroll.v1.PayRunService`.
struct PayRunSink {
    client: payroll::pay_run_service_client::PayRunServiceClient<Channel>,
}

impl QuotaBucketTotalsIdDestination for PayRunSink {
    fn apply(&self, record: QuotaBucketTotalsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_pay_run(payroll::UpdatePayRunRequest {
                    pay_run: Some(payroll::PayRun {
                        number: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.ApiKeyQueryService`.
struct ApiKeyQuerySink {
    client: edgegateway::api_key_query_service_client::ApiKeyQueryServiceClient<Channel>,
}

impl QuotaBucketTotalsIdDestination for ApiKeyQuerySink {
    fn apply(&self, record: QuotaBucketTotalsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_api_keys(edgegateway::SummarizeApiKeysRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
