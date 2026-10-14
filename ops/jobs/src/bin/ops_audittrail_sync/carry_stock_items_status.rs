//! `carry_stock_items_status` reads `acme.inventory.v1.StockItemQueryService/SummarizeStockItems` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::inventory::v1 as inventory;
use acme_proto::acme::personalization::v1 as personalization;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel.clone());
    let fetched = source
        .summarize_stock_items(inventory::SummarizeStockItemsRequest {
            context: Some(context.clone()),
            stock_item_id: "main-carry-stock-items-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let period = fetched.period.unwrap_or_default();
    let record = StockItemsStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        download_uri: fetched.download_uri,
        timezone: acme_kit::text::normalize(&period.timezone),
        report_id: acme_kit::text::normalize(&fetched.report_id),
    };
    let sinks: Vec<(&str, Box<dyn StockItemsStatusDestination>)> = vec![
        (
            "profile-calculation",
            Box::new(ProfileCalculationSink {
                client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "flag-rollout-sync",
            Box::new(FlagRolloutSyncSink {
                client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "flag-rollout-sync" {
            continue;
        }
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `carry_stock_items_status` keeps of each SummarizeStockItems response.
#[derive(Clone, Debug, Default)]
pub struct StockItemsStatusRecord {
    pub status: String,
    pub download_uri: String,
    pub timezone: String,
    pub report_id: String,
}

/// Where `carry_stock_items_status` hands each record.
trait StockItemsStatusDestination {
    fn forward(&self, record: StockItemsStatusRecord) -> Delivery;
}

/// Hands records to `acme.personalization.v1.ProfileCalculationService`.
struct ProfileCalculationSink {
    client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient<Channel>,
}

impl StockItemsStatusDestination for ProfileCalculationSink {
    fn forward(&self, record: StockItemsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_profile_rates(personalization::GetProfileRatesRequest {
                    profile_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FlagRolloutSyncService`.
struct FlagRolloutSyncSink {
    client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient<Channel>,
}

impl StockItemsStatusDestination for FlagRolloutSyncSink {
    fn forward(&self, record: StockItemsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reconcile_flag_rollouts(featureflags::ReconcileFlagRolloutsRequest {
                    flag_rollout: Some(featureflags::FlagRollout {
                        flag_rollout_id: record.status,
                        ..Default::default()
                    }),
                    details: Some(featureflags::reconcile_flag_rollouts_request::Details {
                        checksum: record.timezone,
                        resolution: record.download_uri,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
