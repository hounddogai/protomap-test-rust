//! `settle_flag_rollout_test_notice_status` reads
//! `acme.featureflags.v1.FlagRolloutNotificationService/SendFlagRolloutTestNotice` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient::new(
        channel.clone(),
    );
    let found = source
        .send_flag_rollout_test_notice(featureflags::SendFlagRolloutTestNoticeRequest {
            context: Some(context.clone()),
            reason: "nightly-settle-flag-rollout-test-notice-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let flag_rollout = found.flag_rollout.unwrap_or_default();
    let record = FlagRolloutTestNoticeStatusRecord {
        status: result.status,
        message: outcome.message,
        title: flag_rollout.title,
    };
    let mut sinks: BTreeMap<&str, Box<dyn FlagRolloutTestNoticeStatusTarget>> = BTreeMap::new();
    sinks.insert(
        "experiment-calculation",
        Box::new(ExperimentCalculationSink {
            client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "warehouse-query",
        Box::new(WarehouseQuerySink {
            client: warehousing::warehouse_query_service_client::WarehouseQueryServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("experiment-calculation") else {
        anyhow::bail!("no sink registered for experiment-calculation");
    };
    sink.apply(record).await
}

/// What `settle_flag_rollout_test_notice_status` keeps of each SendFlagRolloutTestNotice response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutTestNoticeStatusRecord {
    pub status: String,
    pub message: String,
    pub title: String,
}

/// Where `settle_flag_rollout_test_notice_status` hands each record.
trait FlagRolloutTestNoticeStatusTarget {
    fn apply(&self, record: FlagRolloutTestNoticeStatusRecord) -> Delivery;
}

/// Hands records to `acme.experiments.v1.ExperimentCalculationService`.
struct ExperimentCalculationSink {
    client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient<Channel>,
}

impl FlagRolloutTestNoticeStatusTarget for ExperimentCalculationSink {
    fn apply(&self, record: FlagRolloutTestNoticeStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_experiments(experiments::RecalculateExperimentsRequest {
                    experiment: Some(experiments::Experiment {
                        experiment_id: record.message,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.warehousing.v1.WarehouseQueryService`.
struct WarehouseQuerySink {
    client: warehousing::warehouse_query_service_client::WarehouseQueryServiceClient<Channel>,
}

impl FlagRolloutTestNoticeStatusTarget for WarehouseQuerySink {
    fn apply(&self, record: FlagRolloutTestNoticeStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_warehouse(warehousing::LookupWarehouseRequest {
                    warehouse_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
