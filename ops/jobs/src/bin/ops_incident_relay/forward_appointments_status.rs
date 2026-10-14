//! `forward_appointments_status` reads `acme.appointments.v1.AppointmentQueryService/CountAppointments` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        appointments::appointment_query_service_client::AppointmentQueryServiceClient::new(channel.clone());
    let loaded = source
        .count_appointments(appointments::CountAppointmentsRequest {
            context: Some(context.clone()),
            order_by: "nightly-forward-appointments-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = AppointmentsStatusRecord {
        status: result.status,
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn AppointmentsStatusDestination>> = vec![
        Box::new(AnomalyQuerySink {
            client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient::new(channel.clone()),
        }),
        Box::new(EnvironmentValidationSink {
            client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(FlagEvaluationExportSink {
            client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    ];
    sinks[1].forward(record).await
}

/// What `forward_appointments_status` keeps of each CountAppointments response.
#[derive(Clone, Debug, Default)]
pub struct AppointmentsStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `forward_appointments_status` hands each record.
trait AppointmentsStatusDestination {
    fn forward(&self, record: AppointmentsStatusRecord) -> Delivery;
}

/// Passes records to `acme.anomalies.v1.AnomalyQueryService`.
struct AnomalyQuerySink {
    client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient<Channel>,
}

impl AppointmentsStatusDestination for AnomalyQuerySink {
    fn forward(&self, record: AppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_anomaly(anomalies::PeekAnomalyRequest {
                    anomaly_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.EnvironmentValidationService`.
struct EnvironmentValidationSink {
    client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl AppointmentsStatusDestination for EnvironmentValidationSink {
    fn forward(&self, record: AppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::LintEnvironmentsRequest {
                    environment: Some(configuration::Environment {
                        environment_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_environments(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagEvaluationExportService`.
struct FlagEvaluationExportSink {
    client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient<Channel>,
    context: common::RequestContext,
}

impl AppointmentsStatusDestination for FlagEvaluationExportSink {
    fn forward(&self, record: AppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::GetFlagEvaluationExportRequest {
                    flag_evaluation_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_flag_evaluation_export(request).await?;
            Ok(())
        })
    }
}
