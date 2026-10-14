//! `propagate_flag_evaluation_source` reads
//! `acme.featureflags.v1.FlagEvaluationAdminService/RepairFlagEvaluation` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::promotions::v1 as promotions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(channel.clone());
    let found = source
        .repair_flag_evaluation(featureflags::RepairFlagEvaluationRequest {
            context: Some(context.clone()),
            flag_evaluation_id: "live-propagate-flag-evaluation-source".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let flag_evaluation = found.flag_evaluation.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = FlagEvaluationSourceRecord {
        source: flag_evaluation.source,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: outcome.status,
        flag_evaluation_correlation_id: flag_evaluation.correlation_id,
    };
    let sinks: Vec<Box<dyn FlagEvaluationSourceTarget>> = vec![
        Box::new(PromotionCalculationSink {
            client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(MetricSeriesQuerySink {
            client: metrics::metric_series_query_service_client::MetricSeriesQueryServiceClient::new(channel),
        }),
    ];
    for sink in &sinks {
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `propagate_flag_evaluation_source` keeps of each RepairFlagEvaluation response.
#[derive(Clone, Debug, Default)]
pub struct FlagEvaluationSourceRecord {
    pub source: String,
    pub correlation_id: String,
    pub status: String,
    pub flag_evaluation_correlation_id: String,
}

/// Where `propagate_flag_evaluation_source` hands each record.
trait FlagEvaluationSourceTarget {
    fn forward(&self, record: FlagEvaluationSourceRecord) -> Delivery;
}

/// Sends records to `acme.promotions.v1.PromotionCalculationService`.
struct PromotionCalculationSink {
    client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient<Channel>,
}

impl FlagEvaluationSourceTarget for PromotionCalculationSink {
    fn forward(&self, record: FlagEvaluationSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_promotion(promotions::SimulatePromotionRequest {
                    reason: record.status,
                    promotion: Some(promotions::Promotion {
                        promotion_id: record.source,
                        ..Default::default()
                    }),
                    details: Some(promotions::simulate_promotion_request::Details {
                        currency_code: record.flag_evaluation_correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.metrics.v1.MetricSeriesQueryService`.
struct MetricSeriesQuerySink {
    client: metrics::metric_series_query_service_client::MetricSeriesQueryServiceClient<Channel>,
}

impl FlagEvaluationSourceTarget for MetricSeriesQuerySink {
    fn forward(&self, record: FlagEvaluationSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_metric_serieses(metrics::CountMetricSeriesesRequest {
                    filter: Some(metrics::count_metric_serieses_request::Filter {
                        owner_id: record.flag_evaluation_correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
