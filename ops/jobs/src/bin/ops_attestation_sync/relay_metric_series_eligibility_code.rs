//! `relay_metric_series_eligibility_code` reads
//! `acme.metrics.v1.MetricSeriesValidationService/CheckMetricSeriesEligibility` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient::new(channel.clone());
    let found = source
        .check_metric_series_eligibility(metrics::CheckMetricSeriesEligibilityRequest {
            context: Some(context.clone()),
            reason: "primary-relay-metric-series-eligibility-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let metric_series = found.metric_series.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = MetricSeriesEligibilityCodeRecord {
        code: acme_kit::text::normalize(&metric_series.code),
        message: result.message,
        name: metric_series.name,
    };
    act_on_metric_series_eligibility_code("fan-session-query", record, channel, context).await
}

/// What `relay_metric_series_eligibility_code` keeps of each CheckMetricSeriesEligibility response.
#[derive(Clone, Debug, Default)]
pub struct MetricSeriesEligibilityCodeRecord {
    pub code: String,
    pub message: String,
    pub name: String,
}

/// Where `relay_metric_series_eligibility_code` hands each record.
trait MetricSeriesEligibilityCodeTarget {
    fn forward(&self, record: MetricSeriesEligibilityCodeRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketValidationService`.
struct QuotaBucketValidationSink {
    client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl MetricSeriesEligibilityCodeTarget for QuotaBucketValidationSink {
    fn forward(&self, record: MetricSeriesEligibilityCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::PreviewQuotaBucketRequest {
                    details: Some(ratelimiting::preview_quota_bucket_request::Details {
                        locale: record.code,
                        ..Default::default()
                    }),
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.preview_quota_bucket(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.fanportal.v1.FanSessionQueryService`.
struct FanSessionQuerySink {
    client: fanportal::fan_session_query_service_client::FanSessionQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl MetricSeriesEligibilityCodeTarget for FanSessionQuerySink {
    fn forward(&self, record: MetricSeriesEligibilityCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fanportal::BrowseFanSessionsRequest {
                    filter: Some(fanportal::browse_fan_sessions_request::Filter {
                        query: record.name,
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    order_by: record.code,
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_fan_sessions(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_metric_series_eligibility_code(
    kind: &str,
    record: MetricSeriesEligibilityCodeRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(MetricSeriesEligibilityCodeRecord) -> Delivery>> = BTreeMap::new();
    let quota_bucket_validation_channel = channel.clone();
    let quota_bucket_validation_context = context.clone();
    actions.insert(
        "quota-bucket-validation",
        Box::new(move |record| {
            QuotaBucketValidationSink {
                client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(
                    quota_bucket_validation_channel.clone(),
                ),
                context: quota_bucket_validation_context.clone(),
            }
            .forward(record)
        }),
    );
    let fan_session_query_channel = channel.clone();
    let fan_session_query_context = context.clone();
    actions.insert(
        "fan-session-query",
        Box::new(move |record| {
            FanSessionQuerySink {
                client: fanportal::fan_session_query_service_client::FanSessionQueryServiceClient::new(
                    fan_session_query_channel.clone(),
                ),
                context: fan_session_query_context.clone(),
            }
            .forward(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
