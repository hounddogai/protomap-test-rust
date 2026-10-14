//! `forward_api_key_updates_message` reads
//! `acme.edgegateway.v1.ApiKeyNotificationService/SubscribeApiKeyUpdates` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::api_key_notification_service_client::ApiKeyNotificationServiceClient::new(channel.clone());
    let reply = source
        .subscribe_api_key_updates(edgegateway::SubscribeApiKeyUpdatesRequest {
            context: Some(context.clone()),
            api_key_id: "current-forward-api-key-updates-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let api_key = reply.api_key.unwrap_or_default();
    let record = ApiKeyUpdatesMessageRecord {
        message: result.message,
        api_key_id: api_key.api_key_id,
    };
    deliver_api_key_updates_message("throttle-calculation", record, channel, &context).await
}

/// What `forward_api_key_updates_message` keeps of each SubscribeApiKeyUpdates response.
#[derive(Clone, Debug, Default)]
pub struct ApiKeyUpdatesMessageRecord {
    pub message: String,
    pub api_key_id: String,
}

/// Where `forward_api_key_updates_message` hands each record.
trait ApiKeyUpdatesMessageWriter {
    fn apply(&self, record: ApiKeyUpdatesMessageRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationHistoryService`.
struct FlagEvaluationHistorySink {
    client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ApiKeyUpdatesMessageWriter for FlagEvaluationHistorySink {
    fn apply(&self, record: ApiKeyUpdatesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::CompareFlagEvaluationVersionsRequest {
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        correlation_id: record.api_key_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_flag_evaluation_versions(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.notifications.v1.NotificationService`.
struct NotificationSink {
    client: notifications::notification_service_client::NotificationServiceClient<Channel>,
}

impl ApiKeyUpdatesMessageWriter for NotificationSink {
    fn apply(&self, record: ApiKeyUpdatesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_notification(notifications::CloneNotificationRequest {
                    notification: Some(notifications::Notification {
                        subject: record.api_key_id,
                        ..Default::default()
                    }),
                    request_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ApiKeyUpdatesMessageWriter for ThrottleCalculationSink {
    fn apply(&self, record: ApiKeyUpdatesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::EstimateThrottleRequest {
                    throttle: Some(ratelimiting::Throttle {
                        source: record.api_key_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.estimate_throttle(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ApiKeyUpdatesMessageLogOnly;

impl ApiKeyUpdatesMessageWriter for ApiKeyUpdatesMessageLogOnly {
    fn apply(&self, record: ApiKeyUpdatesMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_api_key_updates_message(
    kind: &str,
    record: ApiKeyUpdatesMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn ApiKeyUpdatesMessageWriter> = match kind {
        "flag-evaluation-history" => Box::new(FlagEvaluationHistorySink {
            client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "notification" => Box::new(NotificationSink {
            client: notifications::notification_service_client::NotificationServiceClient::new(channel.clone()),
        }),
        "throttle-calculation" => Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(ApiKeyUpdatesMessageLogOnly),
    };
    sink.apply(record).await
}
