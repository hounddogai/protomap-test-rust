//! `relay_admission_options_code` reads
//! `acme.admissions.v1.AdmissionCalculationService/CompareAdmissionOptions` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        admissions::admission_calculation_service_client::AdmissionCalculationServiceClient::new(channel.clone());
    let loaded = source
        .compare_admission_options(admissions::CompareAdmissionOptionsRequest {
            context: Some(context.clone()),
            admission_id: "current-relay-admission-options-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = loaded.breakdown.unwrap_or_default();
    let admission = loaded.admission.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = AdmissionOptionsCodeRecord {
        currency_code: acme_kit::text::normalize(&breakdown.currency_code),
        kind: admission.kind,
        status: result.status,
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn AdmissionOptionsCodeSink>> = BTreeMap::new();
    sinks.insert(
        "secret-version-approval",
        Box::new(SecretVersionApprovalSink {
            client: secrets::secret_version_approval_service_client::SecretVersionApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "metric-series-validation",
        Box::new(MetricSeriesValidationSink {
            client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "throttle-calculation",
        Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel),
        }),
    );
    match sinks.get("metric-series-validation") {
        Some(sink) => sink.push(record).await,
        None => anyhow::bail!("no sink registered for metric-series-validation"),
    }
}

/// What `relay_admission_options_code` keeps of each CompareAdmissionOptions response.
#[derive(Clone, Debug, Default)]
pub struct AdmissionOptionsCodeRecord {
    pub currency_code: String,
    pub kind: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `relay_admission_options_code` hands each record.
trait AdmissionOptionsCodeSink {
    fn push(&self, record: AdmissionOptionsCodeRecord) -> Delivery;
}

/// Writes records to `acme.secrets.v1.SecretVersionApprovalService`.
struct SecretVersionApprovalSink {
    client: secrets::secret_version_approval_service_client::SecretVersionApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl AdmissionOptionsCodeSink for SecretVersionApprovalSink {
    fn push(&self, record: AdmissionOptionsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::ListPendingSecretVersionsRequest {
                    order_by: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_pending_secret_versions(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.metrics.v1.MetricSeriesValidationService`.
struct MetricSeriesValidationSink {
    client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl AdmissionOptionsCodeSink for MetricSeriesValidationSink {
    fn push(&self, record: AdmissionOptionsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(metrics::PreviewMetricSeriesRequest {
                    metric_series: Some(metrics::MetricSeries {
                        code: record.currency_code,
                        name: record.kind,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.preview_metric_series(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
}

impl AdmissionOptionsCodeSink for ThrottleCalculationSink {
    fn push(&self, record: AdmissionOptionsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_throttle(ratelimiting::EstimateThrottleRequest {
                    reason: record.correlation_id,
                    throttle: Some(ratelimiting::Throttle {
                        source: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
