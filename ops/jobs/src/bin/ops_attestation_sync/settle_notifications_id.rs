//! `settle_notifications_id` reads `acme.notifications.v1.NotificationService/SearchNotifications` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = notifications::notification_service_client::NotificationServiceClient::new(channel.clone());
    let reply = source
        .search_notifications(notifications::SearchNotificationsRequest {
            context: Some(context.clone()),
            order_by: "default-settle-notifications-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = NotificationsIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        message: result.message,
    };
    deliver_notifications_id("flag-rollout-validation", record, channel, &context).await
}

/// What `settle_notifications_id` keeps of each SearchNotifications response.
#[derive(Clone, Debug, Default)]
pub struct NotificationsIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub message: String,
}

/// Where `settle_notifications_id` hands each record.
trait NotificationsIdDestination {
    fn deliver(&self, record: NotificationsIdRecord) -> Delivery;
}

/// Writes records to `acme.secrets.v1.SecretVersionQueryService`.
struct SecretVersionQuerySink {
    client: secrets::secret_version_query_service_client::SecretVersionQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl NotificationsIdDestination for SecretVersionQuerySink {
    fn deliver(&self, record: NotificationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::LookupSecretVersionRequest {
                    secret_version_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_secret_version(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagRolloutValidationService`.
struct FlagRolloutValidationSink {
    client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient<Channel>,
}

impl NotificationsIdDestination for FlagRolloutValidationSink {
    fn deliver(&self, record: NotificationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_flag_rollout(featureflags::ScoreFlagRolloutRequest {
                    flag_rollout_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct NotificationsIdLogOnly;

impl NotificationsIdDestination for NotificationsIdLogOnly {
    fn deliver(&self, record: NotificationsIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_notifications_id(
    kind: &str,
    record: NotificationsIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn NotificationsIdDestination> = match kind {
        "secret-version-query" => Box::new(SecretVersionQuerySink {
            client: secrets::secret_version_query_service_client::SecretVersionQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "flag-rollout-validation" => Box::new(FlagRolloutValidationSink {
            client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient::new(
                channel,
            ),
        }),
        _ => Box::new(NotificationsIdLogOnly),
    };
    sink.deliver(record).await
}
