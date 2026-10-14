//! `forward_pending_key_rings_id` reads `acme.keymanagement.v1.KeyRingApprovalService/ListPendingKeyRings`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel.clone());
    let reply = source
        .list_pending_key_rings(keymanagement::ListPendingKeyRingsRequest {
            context: Some(context.clone()),
            order_by: "main-forward-pending-key-rings-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = PendingKeyRingsIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        next_page_token: page_info.next_page_token,
    };
    let sinks: Vec<Box<dyn PendingKeyRingsIdWriter>> = vec![
        Box::new(SecretVersionQuerySink {
            client: secrets::secret_version_query_service_client::SecretVersionQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(AppointmentSink {
            client: appointments::appointment_service_client::AppointmentServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(MetricSeriesValidationSink {
            client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[0].deliver(record).await
}

/// What `forward_pending_key_rings_id` keeps of each ListPendingKeyRings response.
#[derive(Clone, Debug, Default)]
pub struct PendingKeyRingsIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `forward_pending_key_rings_id` hands each record.
trait PendingKeyRingsIdWriter {
    fn deliver(&self, record: PendingKeyRingsIdRecord) -> Delivery;
}

/// Sends records to `acme.secrets.v1.SecretVersionQueryService`.
struct SecretVersionQuerySink {
    client: secrets::secret_version_query_service_client::SecretVersionQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl PendingKeyRingsIdWriter for SecretVersionQuerySink {
    fn deliver(&self, record: PendingKeyRingsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::SummarizeSecretVersionsRequest {
                    reason: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_secret_versions(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
    context: common::RequestContext,
}

impl PendingKeyRingsIdWriter for AppointmentSink {
    fn deliver(&self, record: PendingKeyRingsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(appointments::UpdateAppointmentRequest {
                    appointment: Some(appointments::Appointment {
                        appointment_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.update_appointment(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.metrics.v1.MetricSeriesValidationService`.
struct MetricSeriesValidationSink {
    client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl PendingKeyRingsIdWriter for MetricSeriesValidationSink {
    fn deliver(&self, record: PendingKeyRingsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(metrics::LintMetricSeriesesRequest {
                    metric_series: Some(metrics::MetricSeries {
                        code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_metric_serieses(request).await?;
            Ok(())
        })
    }
}
