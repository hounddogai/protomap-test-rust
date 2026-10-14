//! `hand_off_config_changes_status` reads
//! `acme.configuration.v1.ConfigChangeCalculationService/RecalculateConfigChanges` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::metrics::v1 as metrics;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient::new(
        channel.clone(),
    );
    let reply = source
        .recalculate_config_changes(configuration::RecalculateConfigChangesRequest {
            context: Some(context.clone()),
            config_change_id: "nightly-hand-off-config-changes-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let config_change = reply.config_change.unwrap_or_default();
    let record = OffConfigChangesStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        correlation_id: result.correlation_id,
        kind: config_change.kind,
    };
    deliver_off_config_changes_status("vehicle-query", record, channel, &context).await
}

/// What `hand_off_config_changes_status` keeps of each RecalculateConfigChanges response.
#[derive(Clone, Debug, Default)]
pub struct OffConfigChangesStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub kind: String,
}

/// Where `hand_off_config_changes_status` hands each record.
trait OffConfigChangesStatusWriter {
    fn forward(&self, record: OffConfigChangesStatusRecord) -> Delivery;
}

/// Sends records to `acme.fleet.v1.VehicleQueryService`.
struct VehicleQuerySink {
    client: fleet::vehicle_query_service_client::VehicleQueryServiceClient<Channel>,
}

impl OffConfigChangesStatusWriter for VehicleQuerySink {
    fn forward(&self, record: OffConfigChangesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_vehicles(fleet::FindVehiclesRequest {
                    filter: Some(fleet::find_vehicles_request::Filter {
                        region: record.status,
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    order_by: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.metrics.v1.MetricSeriesService`.
struct MetricSeriesSink {
    client: metrics::metric_series_service_client::MetricSeriesServiceClient<Channel>,
}

impl OffConfigChangesStatusWriter for MetricSeriesSink {
    fn forward(&self, record: OffConfigChangesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_metric_serieses(metrics::SearchMetricSeriesesRequest {
                    order_by: record.kind,
                    filter: Some(metrics::search_metric_serieses_request::Filter {
                        owner_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyService`.
struct CryptoKeySink {
    client: keymanagement::crypto_key_service_client::CryptoKeyServiceClient<Channel>,
}

impl OffConfigChangesStatusWriter for CryptoKeySink {
    fn forward(&self, record: OffConfigChangesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_crypto_key(keymanagement::GetCryptoKeyRequest {
                    crypto_key_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct OffConfigChangesStatusLogOnly;

impl OffConfigChangesStatusWriter for OffConfigChangesStatusLogOnly {
    fn forward(&self, record: OffConfigChangesStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_off_config_changes_status(
    kind: &str,
    record: OffConfigChangesStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn OffConfigChangesStatusWriter> = match kind {
        "vehicle-query" => Box::new(VehicleQuerySink {
            client: fleet::vehicle_query_service_client::VehicleQueryServiceClient::new(channel.clone()),
        }),
        "metric-series" => Box::new(MetricSeriesSink {
            client: metrics::metric_series_service_client::MetricSeriesServiceClient::new(channel.clone()),
        }),
        "crypto-key" => Box::new(CryptoKeySink {
            client: keymanagement::crypto_key_service_client::CryptoKeyServiceClient::new(channel),
        }),
        _ => Box::new(OffConfigChangesStatusLogOnly),
    };
    sink.forward(record).await
}
