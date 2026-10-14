//! `hand_off_show_title` reads `acme.shows.v1.ShowQueryService/PeekShow` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = shows::show_query_service_client::ShowQueryServiceClient::new(channel.clone());
    let loaded = source
        .peek_show(shows::PeekShowRequest {
            context: Some(context.clone()),
            show_id: "default-hand-off-show-title".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let show = loaded.show.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = OffShowTitleRecord {
        title: acme_kit::text::normalize(&show.title),
        correlation_id: result.correlation_id,
        show_id: show.show_id,
    };
    dispatch_off_show_title("environment-approval", record, channel, &context).await
}

/// What `hand_off_show_title` keeps of each PeekShow response.
#[derive(Clone, Debug, Default)]
pub struct OffShowTitleRecord {
    pub title: String,
    pub correlation_id: String,
    pub show_id: String,
}

/// Where `hand_off_show_title` hands each record.
trait OffShowTitleSink {
    fn deliver(&self, record: OffShowTitleRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FlagRolloutCalculationService`.
struct FlagRolloutCalculationSink {
    client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient<Channel>,
}

impl OffShowTitleSink for FlagRolloutCalculationSink {
    fn deliver(&self, record: OffShowTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_flag_rollout_calculation(featureflags::ExplainFlagRolloutCalculationRequest {
                    flag_rollout: Some(featureflags::FlagRollout {
                        title: record.correlation_id,
                        ..Default::default()
                    }),
                    reason: record.title,
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

impl OffShowTitleSink for MetricSeriesValidationSink {
    fn deliver(&self, record: OffShowTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_metric_series_decision(metrics::ExplainMetricSeriesDecisionRequest {
                    metric_series: Some(metrics::MetricSeries {
                        code: record.show_id,
                        name: record.title,
                        ..Default::default()
                    }),
                    metric_series_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.EnvironmentApprovalService`.
struct EnvironmentApprovalSink {
    client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient<Channel>,
}

impl OffShowTitleSink for EnvironmentApprovalSink {
    fn deliver(&self, record: OffShowTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reject_environment(configuration::RejectEnvironmentRequest {
                    environment: Some(configuration::Environment {
                        code: record.show_id,
                        ..Default::default()
                    }),
                    environment_id: record.title,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FlagRolloutHistoryService`.
struct FlagRolloutHistorySink {
    client: featureflags::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffShowTitleSink for FlagRolloutHistorySink {
    fn deliver(&self, record: OffShowTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::RecordFlagRolloutVersionRequest {
                    flag_rollout: Some(featureflags::FlagRollout {
                        flag_rollout_id: record.show_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.record_flag_rollout_version(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct OffShowTitleLogOnly;

impl OffShowTitleSink for OffShowTitleLogOnly {
    fn deliver(&self, record: OffShowTitleRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.title);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_off_show_title(
    kind: &str,
    record: OffShowTitleRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let flag_rollout_calculation = FlagRolloutCalculationSink {
        client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let metric_series_validation = MetricSeriesValidationSink {
        client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient::new(
            channel.clone(),
        ),
    };
    let environment_approval = EnvironmentApprovalSink {
        client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient::new(
            channel.clone(),
        ),
    };
    let flag_rollout_history = FlagRolloutHistorySink {
        client: featureflags::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(OffShowTitleRecord) -> Delivery = match kind {
        "flag-rollout-calculation" => &|record| flag_rollout_calculation.deliver(record),
        "metric-series-validation" => &|record| metric_series_validation.deliver(record),
        "environment-approval" => &|record| environment_approval.deliver(record),
        "flag-rollout-history" => &|record| flag_rollout_history.deliver(record),
        _ => &|record| OffShowTitleLogOnly.deliver(record),
    };
    write(record).await
}
