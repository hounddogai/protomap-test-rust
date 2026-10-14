//! `replay_audit_events_status` reads `acme.audit.v1.AuditEventQueryService/CountAuditEvents` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::helpdesk::v1 as helpdesk;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = audit::audit_event_query_service_client::AuditEventQueryServiceClient::new(channel.clone());
    let reply = source
        .count_audit_events(audit::CountAuditEventsRequest {
            context: Some(context.clone()),
            order_by: "live-replay-audit-events-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = AuditEventsStatusRecord {
        status: result.status,
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
    };
    deliver_audit_events_status("csr-calculation", record, channel, &context).await
}

/// What `replay_audit_events_status` keeps of each CountAuditEvents response.
#[derive(Clone, Debug, Default)]
pub struct AuditEventsStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `replay_audit_events_status` hands each record.
trait AuditEventsStatusDestination {
    fn deliver(&self, record: AuditEventsStatusRecord) -> Delivery;
}

/// Hands records to `acme.helpdesk.v1.TicketService`.
struct TicketSink {
    client: helpdesk::ticket_service_client::TicketServiceClient<Channel>,
}

impl AuditEventsStatusDestination for TicketSink {
    fn deliver(&self, record: AuditEventsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_ticket(helpdesk::UpdateTicketRequest {
                    ticket: Some(helpdesk::Ticket {
                        subject: record.status,
                        ticket_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.yards.v1.YardService`.
struct YardSink {
    client: yards::yard_service_client::YardServiceClient<Channel>,
}

impl AuditEventsStatusDestination for YardSink {
    fn deliver(&self, record: AuditEventsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_yards(yards::SearchYardsRequest {
                    filter: Some(yards::search_yards_request::Filter {
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.certificates.v1.CsrCalculationService`.
struct CsrCalculationSink {
    client: certificates::csr_calculation_service_client::CsrCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl AuditEventsStatusDestination for CsrCalculationSink {
    fn deliver(&self, record: AuditEventsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::CompareCsrOptionsRequest {
                    csr_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_csr_options(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct AuditEventsStatusLogOnly;

impl AuditEventsStatusDestination for AuditEventsStatusLogOnly {
    fn deliver(&self, record: AuditEventsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_audit_events_status(
    kind: &str,
    record: AuditEventsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn AuditEventsStatusDestination> = match kind {
        "ticket" => Box::new(TicketSink {
            client: helpdesk::ticket_service_client::TicketServiceClient::new(channel.clone()),
        }),
        "yard" => Box::new(YardSink {
            client: yards::yard_service_client::YardServiceClient::new(channel.clone()),
        }),
        "csr-calculation" => Box::new(CsrCalculationSink {
            client: certificates::csr_calculation_service_client::CsrCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(AuditEventsStatusLogOnly),
    };
    sink.deliver(record).await
}
