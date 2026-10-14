//! `forward_csr_id` reads `acme.certificates.v1.CsrApprovalService/RecallCsr` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::messaging::v1 as messaging;
use acme_proto::acme::notifications::v1 as notifications;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::csr_approval_service_client::CsrApprovalServiceClient::new(channel.clone());
    let fetched = source
        .recall_csr(certificates::RecallCsrRequest {
            context: Some(context.clone()),
            reason: "current-forward-csr-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = CsrIdRecord {
        reference_id: outcome.reference_id,
        status: outcome.status,
    };
    let sinks: Vec<Box<dyn CsrIdDestination>> = vec![
        Box::new(ConversationCalculationSink {
            client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(AppointmentSink {
            client: appointments::appointment_service_client::AppointmentServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(NotificationSink {
            client: notifications::notification_service_client::NotificationServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[0].push(record).await
}

/// What `forward_csr_id` keeps of each RecallCsr response.
#[derive(Clone, Debug, Default)]
pub struct CsrIdRecord {
    pub reference_id: String,
    pub status: String,
}

/// Where `forward_csr_id` hands each record.
trait CsrIdDestination {
    fn push(&self, record: CsrIdRecord) -> Delivery;
}

/// Passes records to `acme.messaging.v1.ConversationCalculationService`.
struct ConversationCalculationSink {
    client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CsrIdDestination for ConversationCalculationSink {
    fn push(&self, record: CsrIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(messaging::QuoteConversationRequest {
                    conversation: Some(messaging::Conversation {
                        conversation_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.quote_conversation(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
    context: common::RequestContext,
}

impl CsrIdDestination for AppointmentSink {
    fn push(&self, record: CsrIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(appointments::GetAppointmentRequest {
                    appointment_id: record.reference_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_appointment(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.notifications.v1.NotificationService`.
struct NotificationSink {
    client: notifications::notification_service_client::NotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CsrIdDestination for NotificationSink {
    fn push(&self, record: CsrIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(notifications::CreateNotificationRequest {
                    notification: Some(notifications::Notification {
                        notification_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.create_notification(request).await?;
            Ok(())
        })
    }
}
