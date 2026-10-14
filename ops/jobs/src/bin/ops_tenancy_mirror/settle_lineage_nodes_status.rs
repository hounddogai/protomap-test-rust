//! `settle_lineage_nodes_status` reads `acme.lineage.v1.LineageNodeQueryService/CountLineageNodes` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::lending::v1 as lending;
use acme_proto::acme::lineage::v1 as lineage;
use acme_proto::acme::notifications::v1 as notifications;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = lineage::lineage_node_query_service_client::LineageNodeQueryServiceClient::new(channel.clone());
    let loaded = source
        .count_lineage_nodes(lineage::CountLineageNodesRequest {
            context: Some(context.clone()),
            order_by: "default-settle-lineage-nodes-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = LineageNodesStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
    };
    let sinks: Vec<Box<dyn LineageNodesStatusSink>> = vec![
        Box::new(GiftCardCalculationSink {
            client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(LoanApplicationQuerySink {
            client: lending::loan_application_query_service_client::LoanApplicationQueryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(AppointmentQuerySink {
            client: appointments::appointment_query_service_client::AppointmentQueryServiceClient::new(channel.clone()),
        }),
        Box::new(NotificationValidationSink {
            client: notifications::notification_validation_service_client::NotificationValidationServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    ];
    for sink in &sinks {
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `settle_lineage_nodes_status` keeps of each CountLineageNodes response.
#[derive(Clone, Debug, Default)]
pub struct LineageNodesStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `settle_lineage_nodes_status` hands each record.
trait LineageNodesStatusSink {
    fn deliver(&self, record: LineageNodesStatusRecord) -> Delivery;
}

/// Passes records to `acme.giftcards.v1.GiftCardCalculationService`.
struct GiftCardCalculationSink {
    client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl LineageNodesStatusSink for GiftCardCalculationSink {
    fn deliver(&self, record: LineageNodesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(giftcards::CalculateGiftCardRequest {
                    reason: record.message,
                    gift_card: Some(giftcards::GiftCard {
                        currency_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.calculate_gift_card(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.lending.v1.LoanApplicationQueryService`.
struct LoanApplicationQuerySink {
    client: lending::loan_application_query_service_client::LoanApplicationQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl LineageNodesStatusSink for LoanApplicationQuerySink {
    fn deliver(&self, record: LineageNodesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(lending::CountLoanApplicationsRequest {
                    filter: Some(lending::count_loan_applications_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.count_loan_applications(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.appointments.v1.AppointmentQueryService`.
struct AppointmentQuerySink {
    client: appointments::appointment_query_service_client::AppointmentQueryServiceClient<Channel>,
}

impl LineageNodesStatusSink for AppointmentQuerySink {
    fn deliver(&self, record: LineageNodesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_appointments(appointments::FindAppointmentsRequest {
                    order_by: record.message,
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.notifications.v1.NotificationValidationService`.
struct NotificationValidationSink {
    client: notifications::notification_validation_service_client::NotificationValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl LineageNodesStatusSink for NotificationValidationSink {
    fn deliver(&self, record: LineageNodesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(notifications::GetNotificationRulesRequest {
                    notification_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_notification_rules(request).await?;
            Ok(())
        })
    }
}
