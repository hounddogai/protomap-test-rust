//! `replay_anomalies_id` reads `acme.anomalies.v1.AnomalyQueryService/SummarizeAnomalies` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = anomalies::anomaly_query_service_client::AnomalyQueryServiceClient::new(channel.clone());
    let fetched = source
        .summarize_anomalies(anomalies::SummarizeAnomaliesRequest {
            context: Some(context.clone()),
            anomaly_id: "primary-replay-anomalies-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let period = fetched.period.unwrap_or_default();
    let record = AnomaliesIdRecord {
        report_id: fetched.report_id,
        status: result.status,
        timezone: period.timezone,
    };
    deliver_anomalies_id("config-change-calculation", record, channel, &context).await
}

/// What `replay_anomalies_id` keeps of each SummarizeAnomalies response.
#[derive(Clone, Debug, Default)]
pub struct AnomaliesIdRecord {
    pub report_id: String,
    pub status: String,
    pub timezone: String,
}

/// Where `replay_anomalies_id` hands each record.
trait AnomaliesIdTarget {
    fn forward(&self, record: AnomaliesIdRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.QuotaBucketSyncService`.
struct QuotaBucketSyncSink {
    client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient<Channel>,
}

impl AnomaliesIdTarget for QuotaBucketSyncSink {
    fn forward(&self, record: AnomaliesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pull_quota_buckets(ratelimiting::PullQuotaBucketsRequest {
                    details: Some(ratelimiting::pull_quota_buckets_request::Details {
                        cursor: record.report_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigChangeCalculationService`.
struct ConfigChangeCalculationSink {
    client: configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl AnomaliesIdTarget for ConfigChangeCalculationSink {
    fn forward(&self, record: AnomaliesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::GetConfigChangeRatesRequest {
                    config_change_id: record.report_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_config_change_rates(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct AnomaliesIdLogOnly;

impl AnomaliesIdTarget for AnomaliesIdLogOnly {
    fn forward(&self, record: AnomaliesIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.report_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_anomalies_id(
    kind: &str,
    record: AnomaliesIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn AnomaliesIdTarget> = match kind {
        "quota-bucket-sync" => Box::new(QuotaBucketSyncSink {
            client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient::new(channel.clone()),
        }),
        "config-change-calculation" => Box::new(ConfigChangeCalculationSink {
            client: configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
        _ => Box::new(AnomaliesIdLogOnly),
    };
    sink.forward(record).await
}
