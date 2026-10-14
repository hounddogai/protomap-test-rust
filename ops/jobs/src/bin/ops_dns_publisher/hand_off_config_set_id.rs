//! `hand_off_config_set_id` reads `acme.configuration.v1.ConfigSetLifecycleService/SuspendConfigSet` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(channel.clone());
    let fetched = source
        .suspend_config_set(configuration::SuspendConfigSetRequest {
            context: Some(context.clone()),
            config_set_id: "main-hand-off-config-set-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_set = fetched.config_set.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = OffConfigSetIdRecord {
        parent_id: config_set.parent_id,
        message: result.message,
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
    };
    let sinks: Vec<(&str, Box<dyn OffConfigSetIdDestination>)> = vec![
        (
            "throttle-workflow",
            Box::new(ThrottleWorkflowSink {
                client: ratelimiting::throttle_workflow_service_client::ThrottleWorkflowServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "video-visit-query",
            Box::new(VideoVisitQuerySink {
                client: telehealth::video_visit_query_service_client::VideoVisitQueryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "role-binding-history",
            Box::new(RoleBindingHistorySink {
                client: authorization::role_binding_history_service_client::RoleBindingHistoryServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "throttle-workflow" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `hand_off_config_set_id` keeps of each SuspendConfigSet response.
#[derive(Clone, Debug, Default)]
pub struct OffConfigSetIdRecord {
    pub parent_id: String,
    pub message: String,
    pub reference_id: String,
}

/// Where `hand_off_config_set_id` hands each record.
trait OffConfigSetIdDestination {
    fn apply(&self, record: OffConfigSetIdRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.ThrottleWorkflowService`.
struct ThrottleWorkflowSink {
    client: ratelimiting::throttle_workflow_service_client::ThrottleWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffConfigSetIdDestination for ThrottleWorkflowSink {
    fn apply(&self, record: OffConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::CancelThrottleWorkflowRequest {
                    reason: record.parent_id,
                    details: Some(ratelimiting::cancel_throttle_workflow_request::Details {
                        initiated_by: record.reference_id,
                        ..Default::default()
                    }),
                    throttle: Some(ratelimiting::Throttle {
                        throttle_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.cancel_throttle_workflow(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.telehealth.v1.VideoVisitQueryService`.
struct VideoVisitQuerySink {
    client: telehealth::video_visit_query_service_client::VideoVisitQueryServiceClient<Channel>,
}

impl OffConfigSetIdDestination for VideoVisitQuerySink {
    fn apply(&self, record: OffConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_video_visits(telehealth::BrowseVideoVisitsRequest {
                    order_by: record.reference_id,
                    filter: Some(telehealth::browse_video_visits_request::Filter {
                        owner_id: record.message,
                        region: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.authorization.v1.RoleBindingHistoryService`.
struct RoleBindingHistorySink {
    client: authorization::role_binding_history_service_client::RoleBindingHistoryServiceClient<Channel>,
}

impl OffConfigSetIdDestination for RoleBindingHistorySink {
    fn apply(&self, record: OffConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_role_binding_versions(authorization::ListRoleBindingVersionsRequest {
                    filter: Some(authorization::list_role_binding_versions_request::Filter {
                        region: record.reference_id,
                        owner_id: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
