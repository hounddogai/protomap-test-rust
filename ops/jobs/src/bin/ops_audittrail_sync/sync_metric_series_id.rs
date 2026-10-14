//! `sync_metric_series_id` reads `acme.metrics.v1.MetricSeriesQueryService/LookupMetricSeries` and hands what
//! it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::metrics::v1 as metrics;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = metrics::metric_series_query_service_client::MetricSeriesQueryServiceClient::new(channel.clone());
    let fetched = source
        .lookup_metric_series(metrics::LookupMetricSeriesRequest {
            context: Some(context.clone()),
            metric_series_id: "default-sync-metric-series-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let metric_series = fetched.metric_series.unwrap_or_default();
    let record = MetricSeriesIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        name: metric_series.name,
        metric_series_id: metric_series.metric_series_id,
        message: result.message,
    };
    let mut sinks: BTreeMap<String, Box<dyn MetricSeriesIdWriter>> = BTreeMap::new();
    sinks.insert(
        "flag-evaluation-calculation".to_owned(),
        Box::new(FlagEvaluationCalculationSink {
            client:
                featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient::new(
                    channel.clone(),
                ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "flag-evaluation-admin".to_owned(),
        Box::new(FlagEvaluationAdminSink {
            client: featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "csr-workflow".to_owned(),
        Box::new(CsrWorkflowSink {
            client: certificates::csr_workflow_service_client::CsrWorkflowServiceClient::new(channel),
        }),
    );
    let target = std::env::var("SYNC_METRIC_SERIES_ID_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.apply(record).await,
        None => Ok(()),
    }
}

/// What `sync_metric_series_id` keeps of each LookupMetricSeries response.
#[derive(Clone, Debug, Default)]
pub struct MetricSeriesIdRecord {
    pub correlation_id: String,
    pub name: String,
    pub metric_series_id: String,
    pub message: String,
}

/// Where `sync_metric_series_id` hands each record.
trait MetricSeriesIdWriter {
    fn apply(&self, record: MetricSeriesIdRecord) -> Delivery;
}

/// Sends records to `acme.featureflags.v1.FlagEvaluationCalculationService`.
struct FlagEvaluationCalculationSink {
    client: featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl MetricSeriesIdWriter for FlagEvaluationCalculationSink {
    fn apply(&self, record: MetricSeriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::GetFlagEvaluationRatesRequest {
                    flag_evaluation_id: record.metric_series_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_flag_evaluation_rates(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationAdminService`.
struct FlagEvaluationAdminSink {
    client: featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient<Channel>,
}

impl MetricSeriesIdWriter for FlagEvaluationAdminSink {
    fn apply(&self, record: MetricSeriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .restore_flag_evaluation(featureflags::RestoreFlagEvaluationRequest {
                    flag_evaluation_id: record.message,
                    reason: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.certificates.v1.CsrWorkflowService`.
struct CsrWorkflowSink {
    client: certificates::csr_workflow_service_client::CsrWorkflowServiceClient<Channel>,
}

impl MetricSeriesIdWriter for CsrWorkflowSink {
    fn apply(&self, record: MetricSeriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_csr_workflow(certificates::RetryCsrWorkflowRequest {
                    csr: Some(certificates::Csr {
                        csr_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
