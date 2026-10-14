//! `hand_off_notifications_id` reads `acme.notifications.v1.NotificationQueryService/SummarizeNotifications`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::notifications::v1 as notifications;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        notifications::notification_query_service_client::NotificationQueryServiceClient::new(channel.clone());
    let reply = source
        .summarize_notifications(notifications::SummarizeNotificationsRequest {
            context: Some(context.clone()),
            reason: "primary-hand-off-notifications-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = OffNotificationsIdRecord {
        report_id: reply.report_id,
        message: result.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        download_uri: acme_kit::text::normalize(&reply.download_uri),
    };
    act_on_off_notifications_id("config-set-workflow", record, channel, context).await
}

/// What `hand_off_notifications_id` keeps of each SummarizeNotifications response.
#[derive(Clone, Debug, Default)]
pub struct OffNotificationsIdRecord {
    pub report_id: String,
    pub message: String,
    pub correlation_id: String,
    pub download_uri: String,
}

/// Where `hand_off_notifications_id` hands each record.
trait OffNotificationsIdSink {
    fn apply(&self, record: OffNotificationsIdRecord) -> Delivery;
}

/// Writes records to `acme.broadcasts.v1.BroadcastService`.
struct BroadcastSink {
    client: broadcasts::broadcast_service_client::BroadcastServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffNotificationsIdSink for BroadcastSink {
    fn apply(&self, record: OffNotificationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(broadcasts::DeleteBroadcastRequest {
                    reason: record.message,
                    broadcast_id: record.correlation_id,
                    etag: record.download_uri,
                    ..Default::default()
                }),
                &context,
            )?;
            client.delete_broadcast(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.community.v1.ForumService`.
struct ForumSink {
    client: community::forum_service_client::ForumServiceClient<Channel>,
}

impl OffNotificationsIdSink for ForumSink {
    fn apply(&self, record: OffNotificationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_forum(community::CreateForumRequest {
                    forum: Some(community::Forum {
                        parent_id: record.message,
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

impl OffNotificationsIdSink for ConfigSetWorkflowSink {
    fn apply(&self, record: OffNotificationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_config_set_workflow(configuration::AdvanceConfigSetWorkflowRequest {
                    config_set: Some(configuration::ConfigSet {
                        config_set_id: record.message,
                        parent_id: record.report_id,
                        ..Default::default()
                    }),
                    details: Some(configuration::advance_config_set_workflow_request::Details {
                        step: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
}

impl OffNotificationsIdSink for AppointmentSink {
    fn apply(&self, record: OffNotificationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_appointment(appointments::UpdateAppointmentRequest {
                    appointment: Some(appointments::Appointment {
                        title: record.correlation_id,
                        appointment_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_off_notifications_id(
    kind: &str,
    record: OffNotificationsIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(OffNotificationsIdRecord) -> Delivery>> = BTreeMap::new();
    let broadcast_channel = channel.clone();
    let broadcast_context = context.clone();
    actions.insert(
        "broadcast",
        Box::new(move |record| {
            BroadcastSink {
                client: broadcasts::broadcast_service_client::BroadcastServiceClient::new(broadcast_channel.clone()),
                context: broadcast_context.clone(),
            }
            .apply(record)
        }),
    );
    let forum_channel = channel.clone();
    actions.insert(
        "forum",
        Box::new(move |record| {
            ForumSink {
                client: community::forum_service_client::ForumServiceClient::new(forum_channel.clone()),
            }
            .apply(record)
        }),
    );
    let config_set_workflow_channel = channel.clone();
    actions.insert(
        "config-set-workflow",
        Box::new(move |record| {
            ConfigSetWorkflowSink {
                client: configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient::new(
                    config_set_workflow_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let appointment_channel = channel.clone();
    actions.insert(
        "appointment",
        Box::new(move |record| {
            AppointmentSink {
                client: appointments::appointment_service_client::AppointmentServiceClient::new(
                    appointment_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
