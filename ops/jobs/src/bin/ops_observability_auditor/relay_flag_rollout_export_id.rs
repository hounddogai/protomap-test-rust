//! `relay_flag_rollout_export_id` reads
//! `acme.featureflags.v1.FlagRolloutExportService/ScheduleFlagRolloutExport` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::maintenance::v1 as maintenance;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_rollout_export_service_client::FlagRolloutExportServiceClient::new(channel.clone());
    let found = source
        .schedule_flag_rollout_export(featureflags::ScheduleFlagRolloutExportRequest {
            context: Some(context.clone()),
            reason: "nightly-relay-flag-rollout-export-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let record = FlagRolloutExportIdRecord {
        report_id: found.report_id,
        message: acme_kit::text::normalize(&result.message),
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    act_on_flag_rollout_export_id("work-order-calculation", record, channel, context).await
}

/// What `relay_flag_rollout_export_id` keeps of each ScheduleFlagRolloutExport response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutExportIdRecord {
    pub report_id: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `relay_flag_rollout_export_id` hands each record.
trait FlagRolloutExportIdSink {
    fn deliver(&self, record: FlagRolloutExportIdRecord) -> Delivery;
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
}

impl FlagRolloutExportIdSink for CryptoKeyCalculationSink {
    fn deliver(&self, record: FlagRolloutExportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_crypto_key(keymanagement::EstimateCryptoKeyRequest {
                    crypto_key: Some(keymanagement::CryptoKey {
                        currency_code: record.message,
                        crypto_key_id: record.correlation_id,
                        ..Default::default()
                    }),
                    crypto_key_id: record.report_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.seating.v1.SeatMapQueryService`.
struct SeatMapQuerySink {
    client: seating::seat_map_query_service_client::SeatMapQueryServiceClient<Channel>,
}

impl FlagRolloutExportIdSink for SeatMapQuerySink {
    fn deliver(&self, record: FlagRolloutExportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_seat_maps(seating::BrowseSeatMapsRequest {
                    filter: Some(seating::browse_seat_maps_request::Filter {
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.maintenance.v1.WorkOrderCalculationService`.
struct WorkOrderCalculationSink {
    client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient<Channel>,
}

impl FlagRolloutExportIdSink for WorkOrderCalculationSink {
    fn deliver(&self, record: FlagRolloutExportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_work_order(maintenance::CalculateWorkOrderRequest {
                    work_order: Some(maintenance::WorkOrder {
                        work_order_id: record.message,
                        ..Default::default()
                    }),
                    work_order_id: record.report_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_flag_rollout_export_id(
    kind: &str,
    record: FlagRolloutExportIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(FlagRolloutExportIdRecord) -> Delivery>> = BTreeMap::new();
    let crypto_key_calculation_channel = channel.clone();
    actions.insert(
        "crypto-key-calculation",
        Box::new(move |record| {
            CryptoKeyCalculationSink {
                client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(
                    crypto_key_calculation_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    let seat_map_query_channel = channel.clone();
    actions.insert(
        "seat-map-query",
        Box::new(move |record| {
            SeatMapQuerySink {
                client: seating::seat_map_query_service_client::SeatMapQueryServiceClient::new(
                    seat_map_query_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    let work_order_calculation_channel = channel.clone();
    actions.insert(
        "work-order-calculation",
        Box::new(move |record| {
            WorkOrderCalculationSink {
                client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient::new(
                    work_order_calculation_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
