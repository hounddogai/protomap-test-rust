//! `settle_environment_status` reads `acme.configuration.v1.EnvironmentLifecycleService/TerminateEnvironment`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::expenses::v1 as expenses;
use acme_proto::acme::patients::v1 as patients;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient::new(channel.clone());
    let found = source
        .terminate_environment(configuration::TerminateEnvironmentRequest {
            context: Some(context.clone()),
            reason: "current-settle-environment-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = EnvironmentStatusRecord {
        status: result.status,
        message: outcome.message,
    };
    let sinks: Vec<Box<dyn EnvironmentStatusSink>> = vec![
        Box::new(PatientQuerySink {
            client: patients::patient_query_service_client::PatientQueryServiceClient::new(channel.clone()),
        }),
        Box::new(ExpenseReportSink {
            client: expenses::expense_report_service_client::ExpenseReportServiceClient::new(channel.clone()),
        }),
        Box::new(EnvironmentLifecycleSink {
            client: configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    ];
    sinks[0].send(record).await
}

/// What `settle_environment_status` keeps of each TerminateEnvironment response.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `settle_environment_status` hands each record.
trait EnvironmentStatusSink {
    fn send(&self, record: EnvironmentStatusRecord) -> Delivery;
}

/// Passes records to `acme.patients.v1.PatientQueryService`.
struct PatientQuerySink {
    client: patients::patient_query_service_client::PatientQueryServiceClient<Channel>,
}

impl EnvironmentStatusSink for PatientQuerySink {
    fn send(&self, record: EnvironmentStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_patients(patients::CountPatientsRequest {
                    order_by: record.status,
                    filter: Some(patients::count_patients_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.expenses.v1.ExpenseReportService`.
struct ExpenseReportSink {
    client: expenses::expense_report_service_client::ExpenseReportServiceClient<Channel>,
}

impl EnvironmentStatusSink for ExpenseReportSink {
    fn send(&self, record: EnvironmentStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_expense_report(expenses::DeleteExpenseReportRequest {
                    etag: record.status,
                    expense_report_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.EnvironmentLifecycleService`.
struct EnvironmentLifecycleSink {
    client: configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl EnvironmentStatusSink for EnvironmentLifecycleSink {
    fn send(&self, record: EnvironmentStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::ScheduleEnvironmentExpiryRequest {
                    environment_id: record.status,
                    environment: Some(configuration::Environment {
                        name: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.schedule_environment_expiry(request).await?;
            Ok(())
        })
    }
}
