//! `route_forum_status` reads `acme.community.v1.ForumService/DeleteForum` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = community::forum_service_client::ForumServiceClient::new(channel.clone());
    let fetched = source
        .delete_forum(community::DeleteForumRequest {
            context: Some(context.clone()),
            forum_id: "default-route-forum-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = ForumStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
    };
    act_on_forum_status("role-binding-notification", record, channel, context).await
}

/// What `route_forum_status` keeps of each DeleteForum response.
#[derive(Clone, Debug, Default)]
pub struct ForumStatusRecord {
    pub status: String,
    pub correlation_id: String,
}

/// Where `route_forum_status` hands each record.
trait ForumStatusSink {
    fn apply(&self, record: ForumStatusRecord) -> Delivery;
}

/// Hands records to `acme.authorization.v1.RoleBindingHistoryService`.
struct RoleBindingHistorySink {
    client: authorization::role_binding_history_service_client::RoleBindingHistoryServiceClient<Channel>,
}

impl ForumStatusSink for RoleBindingHistorySink {
    fn apply(&self, record: ForumStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_role_binding_versions(authorization::CompareRoleBindingVersionsRequest {
                    role_binding_id: record.status,
                    role_binding: Some(authorization::RoleBinding {
                        role_binding_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.RoleBindingNotificationService`.
struct RoleBindingNotificationSink {
    client: authorization::role_binding_notification_service_client::RoleBindingNotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ForumStatusSink for RoleBindingNotificationSink {
    fn apply(&self, record: ForumStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::SendRoleBindingTestNoticeRequest {
                    role_binding_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.send_role_binding_test_notice(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_forum_status(
    kind: &str,
    record: ForumStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(ForumStatusRecord) -> Delivery>> = BTreeMap::new();
    let role_binding_history_channel = channel.clone();
    actions.insert(
        "role-binding-history",
        Box::new(move |record| {
            RoleBindingHistorySink {
                client: authorization::role_binding_history_service_client::RoleBindingHistoryServiceClient::new(
                    role_binding_history_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let role_binding_notification_channel = channel.clone();
    let role_binding_notification_context = context.clone();
    actions.insert(
        "role-binding-notification",
        Box::new(move |record| {
            RoleBindingNotificationSink {
                client:
                    authorization::role_binding_notification_service_client::RoleBindingNotificationServiceClient::new(
                        role_binding_notification_channel.clone(),
                    ),
                context: role_binding_notification_context.clone(),
            }
            .apply(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
