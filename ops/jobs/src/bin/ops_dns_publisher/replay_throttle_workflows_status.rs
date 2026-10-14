//! `replay_throttle_workflows_status` reads
//! `acme.ratelimiting.v1.ThrottleWorkflowService/ListThrottleWorkflows` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::routing::v1 as routing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_workflow_service_client::ThrottleWorkflowServiceClient::new(channel.clone());
    let reply = source
        .list_throttle_workflows(ratelimiting::ListThrottleWorkflowsRequest {
            context: Some(context.clone()),
            order_by: "live-replay-throttle-workflows-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = ThrottleWorkflowsStatusRecord {
        status: result.status,
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        correlation_id: result.correlation_id,
        message: result.message,
    };
    let sinks: Vec<Box<dyn ThrottleWorkflowsStatusWriter>> = vec![
        Box::new(AccessPolicyNotificationSink {
            client:
                authorization::access_policy_notification_service_client::AccessPolicyNotificationServiceClient::new(
                    channel.clone(),
                ),
        }),
        Box::new(FlagRolloutAdminSink {
            client: featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(RouteQuerySink {
            client: routing::route_query_service_client::RouteQueryServiceClient::new(channel),
        }),
    ];
    sinks[1].apply(record).await
}

/// What `replay_throttle_workflows_status` keeps of each ListThrottleWorkflows response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleWorkflowsStatusRecord {
    pub status: String,
    pub next_page_token: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `replay_throttle_workflows_status` hands each record.
trait ThrottleWorkflowsStatusWriter {
    fn apply(&self, record: ThrottleWorkflowsStatusRecord) -> Delivery;
}

/// Passes records to `acme.authorization.v1.AccessPolicyNotificationService`.
struct AccessPolicyNotificationSink {
    client: authorization::access_policy_notification_service_client::AccessPolicyNotificationServiceClient<Channel>,
}

impl ThrottleWorkflowsStatusWriter for AccessPolicyNotificationSink {
    fn apply(&self, record: ThrottleWorkflowsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_access_policy_subscriptions(authorization::ListAccessPolicySubscriptionsRequest {
                    order_by: record.status,
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    filter: Some(authorization::list_access_policy_subscriptions_request::Filter {
                        owner_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagRolloutAdminService`.
struct FlagRolloutAdminSink {
    client: featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleWorkflowsStatusWriter for FlagRolloutAdminSink {
    fn apply(&self, record: ThrottleWorkflowsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::LockFlagRolloutRequest {
                    details: Some(featureflags::lock_flag_rollout_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    flag_rollout_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lock_flag_rollout(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.routing.v1.RouteQueryService`.
struct RouteQuerySink {
    client: routing::route_query_service_client::RouteQueryServiceClient<Channel>,
}

impl ThrottleWorkflowsStatusWriter for RouteQuerySink {
    fn apply(&self, record: ThrottleWorkflowsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_routes(routing::SuggestRoutesRequest {
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    filter: Some(routing::suggest_routes_request::Filter {
                        region: record.message,
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
