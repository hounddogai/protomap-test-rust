//! `relay_tickets_id` reads `acme.helpdesk.v1.TicketService/ListTickets` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::helpdesk::v1 as helpdesk;
use acme_proto::acme::procurement::v1 as procurement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = helpdesk::ticket_service_client::TicketServiceClient::new(channel.clone());
    let fetched = source
        .list_tickets(helpdesk::ListTicketsRequest {
            context: Some(context.clone()),
            order_by: "primary-relay-tickets-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = TicketsIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
        status: acme_kit::text::normalize(&result.status),
    };
    let sinks: Vec<Box<dyn TicketsIdSink>> = vec![
        Box::new(AuditEventQuerySink {
            client: audit::audit_event_query_service_client::AuditEventQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(FlagRolloutQuerySink {
            client: featureflags::flag_rollout_query_service_client::FlagRolloutQueryServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(PurchaseRequisitionQuerySink {
            client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(FeatureFlagNotificationSink {
            client: featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient::new(
                channel,
            ),
        }),
    ];
    for sink in &sinks {
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `relay_tickets_id` keeps of each ListTickets response.
#[derive(Clone, Debug, Default)]
pub struct TicketsIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub status: String,
}

/// Where `relay_tickets_id` hands each record.
trait TicketsIdSink {
    fn send(&self, record: TicketsIdRecord) -> Delivery;
}

/// Sends records to `acme.audit.v1.AuditEventQueryService`.
struct AuditEventQuerySink {
    client: audit::audit_event_query_service_client::AuditEventQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl TicketsIdSink for AuditEventQuerySink {
    fn send(&self, record: TicketsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(audit::FindAuditEventsRequest {
                    order_by: record.status,
                    filter: Some(audit::find_audit_events_request::Filter {
                        region: record.correlation_id,
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
            client.find_audit_events(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagRolloutQueryService`.
struct FlagRolloutQuerySink {
    client: featureflags::flag_rollout_query_service_client::FlagRolloutQueryServiceClient<Channel>,
}

impl TicketsIdSink for FlagRolloutQuerySink {
    fn send(&self, record: TicketsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_flag_rollout(featureflags::LookupFlagRolloutRequest {
                    flag_rollout_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.procurement.v1.PurchaseRequisitionQueryService`.
struct PurchaseRequisitionQuerySink {
    client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient<Channel>,
}

impl TicketsIdSink for PurchaseRequisitionQuerySink {
    fn send(&self, record: TicketsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_purchase_requisition(procurement::PeekPurchaseRequisitionRequest {
                    purchase_requisition_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FeatureFlagNotificationService`.
struct FeatureFlagNotificationSink {
    client: featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient<Channel>,
}

impl TicketsIdSink for FeatureFlagNotificationSink {
    fn send(&self, record: TicketsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .unsubscribe_feature_flag_updates(featureflags::UnsubscribeFeatureFlagUpdatesRequest {
                    feature_flag_id: record.status,
                    details: Some(featureflags::unsubscribe_feature_flag_updates_request::Details {
                        channel: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
