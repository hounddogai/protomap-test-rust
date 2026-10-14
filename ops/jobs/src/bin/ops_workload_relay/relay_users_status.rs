//! `relay_users_status` reads `acme.identity.v1.UserValidationService/LintUsers` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::invoicing::v1 as invoicing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = identity::user_validation_service_client::UserValidationServiceClient::new(channel.clone());
    let found = source
        .lint_users(identity::LintUsersRequest {
            context: Some(context.clone()),
            reason: "current-relay-users-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = UsersStatusRecord {
        status: outcome.status,
        result_status: result.status,
        message: result.message,
    };
    let sinks: Vec<Box<dyn UsersStatusDestination>> = vec![
        Box::new(AppointmentSink {
            client: appointments::appointment_service_client::AppointmentServiceClient::new(channel.clone()),
        }),
        Box::new(ConfigSetWorkflowSink {
            client: configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(InvoiceBatchSink {
            client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel),
        }),
    ];
    sinks[1].send(record).await
}

/// What `relay_users_status` keeps of each LintUsers response.
#[derive(Clone, Debug, Default)]
pub struct UsersStatusRecord {
    pub status: String,
    pub result_status: String,
    pub message: String,
}

/// Where `relay_users_status` hands each record.
trait UsersStatusDestination {
    fn send(&self, record: UsersStatusRecord) -> Delivery;
}

/// Sends records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
}

impl UsersStatusDestination for AppointmentSink {
    fn send(&self, record: UsersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_appointment(appointments::UpdateAppointmentRequest {
                    appointment: Some(appointments::Appointment {
                        appointment_id: record.result_status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.ConfigSetWorkflowService`.
struct ConfigSetWorkflowSink {
    client: configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient<Channel>,
}

impl UsersStatusDestination for ConfigSetWorkflowSink {
    fn send(&self, record: UsersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_config_set_workflow(configuration::AdvanceConfigSetWorkflowRequest {
                    config_set: Some(configuration::ConfigSet {
                        name: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.invoicing.v1.InvoiceBatchService`.
struct InvoiceBatchSink {
    client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient<Channel>,
}

impl UsersStatusDestination for InvoiceBatchSink {
    fn send(&self, record: UsersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_invoice_batch(invoicing::DeleteInvoiceBatchRequest {
                    invoice_batch_id: record.result_status,
                    reason: record.status,
                    etag: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
