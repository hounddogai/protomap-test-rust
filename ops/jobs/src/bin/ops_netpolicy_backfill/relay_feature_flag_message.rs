//! `relay_feature_flag_message` reads `acme.featureflags.v1.FeatureFlagQueryService/LookupFeatureFlag` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::metrics::v1 as metrics;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(channel.clone());
    let loaded = source
        .lookup_feature_flag(featureflags::LookupFeatureFlagRequest {
            context: Some(context.clone()),
            feature_flag_id: "nightly-relay-feature-flag-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let feature_flag = loaded.feature_flag.unwrap_or_default();
    let record = FeatureFlagMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
        name: feature_flag.name,
    };
    act_on_feature_flag_message("metric-series-validation", record, channel, context).await
}

/// What `relay_feature_flag_message` keeps of each LookupFeatureFlag response.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlagMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub name: String,
}

/// Where `relay_feature_flag_message` hands each record.
trait FeatureFlagMessageSink {
    fn write(&self, record: FeatureFlagMessageRecord) -> Delivery;
}

/// Writes records to `acme.fleet.v1.VehicleService`.
struct VehicleSink {
    client: fleet::vehicle_service_client::VehicleServiceClient<Channel>,
}

impl FeatureFlagMessageSink for VehicleSink {
    fn write(&self, record: FeatureFlagMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_vehicle(fleet::CreateVehicleRequest {
                    vehicle: Some(fleet::Vehicle {
                        code: record.message,
                        vehicle_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.metrics.v1.MetricSeriesValidationService`.
struct MetricSeriesValidationSink {
    client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient<Channel>,
}

impl FeatureFlagMessageSink for MetricSeriesValidationSink {
    fn write(&self, record: FeatureFlagMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_metric_serieses(metrics::LintMetricSeriesesRequest {
                    details: Some(metrics::lint_metric_serieses_request::Details {
                        locale: record.name,
                        ..Default::default()
                    }),
                    metric_series: Some(metrics::MetricSeries {
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_feature_flag_message(
    kind: &str,
    record: FeatureFlagMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(FeatureFlagMessageRecord) -> Delivery>> = BTreeMap::new();
    let vehicle_channel = channel.clone();
    actions.insert(
        "vehicle",
        Box::new(move |record| {
            VehicleSink {
                client: fleet::vehicle_service_client::VehicleServiceClient::new(vehicle_channel.clone()),
            }
            .write(record)
        }),
    );
    let metric_series_validation_channel = channel.clone();
    actions.insert(
        "metric-series-validation",
        Box::new(move |record| {
            MetricSeriesValidationSink {
                client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient::new(
                    metric_series_validation_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
