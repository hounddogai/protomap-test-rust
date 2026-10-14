//! `relay_config_set_id` reads `acme.configuration.v1.ConfigSetApprovalService/RecallConfigSet` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::tours::v1 as tours;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient::new(channel.clone());
    let found = source
        .recall_config_set(configuration::RecallConfigSetRequest {
            context: Some(context.clone()),
            reason: "live-relay-config-set-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_set = found.config_set.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = ConfigSetIdRecord {
        parent_id: config_set.parent_id,
        name: config_set.name,
        config_set_id: acme_kit::text::normalize(&config_set.config_set_id),
        message: outcome.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ConfigSetIdWriter>> = BTreeMap::new();
    sinks.insert(
        "environment-validation",
        Box::new(EnvironmentValidationSink {
            client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "tour-query",
        Box::new(TourQuerySink {
            client: tours::tour_query_service_client::TourQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "metric-series",
        Box::new(MetricSeriesSink {
            client: metrics::metric_series_service_client::MetricSeriesServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("tour-query") else {
        anyhow::bail!("no sink registered for tour-query");
    };
    sink.apply(record).await
}

/// What `relay_config_set_id` keeps of each RecallConfigSet response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetIdRecord {
    pub parent_id: String,
    pub name: String,
    pub config_set_id: String,
    pub message: String,
}

/// Where `relay_config_set_id` hands each record.
trait ConfigSetIdWriter {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.EnvironmentValidationService`.
struct EnvironmentValidationSink {
    client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient<Channel>,
}

impl ConfigSetIdWriter for EnvironmentValidationSink {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_environment(configuration::ScoreEnvironmentRequest {
                    environment: Some(configuration::Environment {
                        code: record.name,
                        environment_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.tours.v1.TourQueryService`.
struct TourQuerySink {
    client: tours::tour_query_service_client::TourQueryServiceClient<Channel>,
}

impl ConfigSetIdWriter for TourQuerySink {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_tours(tours::FindToursRequest {
                    filter: Some(tours::find_tours_request::Filter {
                        owner_id: record.message,
                        region: record.config_set_id,
                        ..Default::default()
                    }),
                    order_by: record.parent_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.metrics.v1.MetricSeriesService`.
struct MetricSeriesSink {
    client: metrics::metric_series_service_client::MetricSeriesServiceClient<Channel>,
}

impl ConfigSetIdWriter for MetricSeriesSink {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_metric_series(metrics::DeleteMetricSeriesRequest {
                    etag: record.config_set_id,
                    reason: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
