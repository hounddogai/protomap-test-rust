//! `route_gateway_policy_watchers_status` reads
//! `acme.edgegateway.v1.GatewayPolicyNotificationService/NotifyGatewayPolicyWatchers` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::helpdesk::v1 as helpdesk;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient::new(
            channel.clone(),
        );
    let fetched = source
        .notify_gateway_policy_watchers(edgegateway::NotifyGatewayPolicyWatchersRequest {
            context: Some(context.clone()),
            gateway_policy_id: "default-route-gateway-policy-watchers-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = GatewayPolicyWatchersStatusRecord {
        status: result.status,
        outcome_status: outcome.status,
        message: acme_kit::text::normalize(&result.message),
    };
    dispatch_gateway_policy_watchers_status("rate-limit-workflow", record, channel, &context).await
}

/// What `route_gateway_policy_watchers_status` keeps of each NotifyGatewayPolicyWatchers response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyWatchersStatusRecord {
    pub status: String,
    pub outcome_status: String,
    pub message: String,
}

/// Where `route_gateway_policy_watchers_status` hands each record.
trait GatewayPolicyWatchersStatusTarget {
    fn send(&self, record: GatewayPolicyWatchersStatusRecord) -> Delivery;
}

/// Sends records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
}

impl GatewayPolicyWatchersStatusTarget for RateLimitWorkflowSink {
    fn send(&self, record: GatewayPolicyWatchersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_rate_limit_workflow(ratelimiting::StartRateLimitWorkflowRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        rate_limit_id: record.message,
                        name: record.outcome_status,
                        description: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.helpdesk.v1.TicketService`.
struct TicketSink {
    client: helpdesk::ticket_service_client::TicketServiceClient<Channel>,
}

impl GatewayPolicyWatchersStatusTarget for TicketSink {
    fn send(&self, record: GatewayPolicyWatchersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_tickets(helpdesk::ListTicketsRequest {
                    filter: Some(helpdesk::list_tickets_request::Filter {
                        owner_id: record.outcome_status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct GatewayPolicyWatchersStatusLogOnly;

impl GatewayPolicyWatchersStatusTarget for GatewayPolicyWatchersStatusLogOnly {
    fn send(&self, record: GatewayPolicyWatchersStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_gateway_policy_watchers_status(
    kind: &str,
    record: GatewayPolicyWatchersStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let rate_limit_workflow = RateLimitWorkflowSink {
        client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(channel.clone()),
    };
    let ticket = TicketSink {
        client: helpdesk::ticket_service_client::TicketServiceClient::new(channel),
    };
    let write: &dyn Fn(GatewayPolicyWatchersStatusRecord) -> Delivery = match kind {
        "rate-limit-workflow" => &|record| rate_limit_workflow.send(record),
        "ticket" => &|record| ticket.send(record),
        _ => &|record| GatewayPolicyWatchersStatusLogOnly.send(record),
    };
    write(record).await
}
