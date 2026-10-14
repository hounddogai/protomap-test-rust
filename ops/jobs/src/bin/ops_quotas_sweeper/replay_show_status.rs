//! `replay_show_status` reads `acme.shows.v1.ShowService/CreateShow` and hands what it keeps to the sink of
//! its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = shows::show_service_client::ShowServiceClient::new(channel.clone());
    let found = source
        .create_show(shows::CreateShowRequest {
            context: Some(context.clone()),
            request_id: "default-replay-show-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let show = found.show.unwrap_or_default();
    let record = ShowStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        title: show.title,
        show_id: show.show_id,
    };
    let sinks: Vec<(&str, Box<dyn ShowStatusWriter>)> = vec![
        (
            "feature-flag-history",
            Box::new(FeatureFlagHistorySink {
                client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "chat-room-query",
            Box::new(ChatRoomQuerySink {
                client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient::new(channel.clone()),
                context: context.clone(),
            }),
        ),
        (
            "rate-limit-calculation",
            Box::new(RateLimitCalculationSink {
                client: ratelimiting::rate_limit_calculation_service_client::RateLimitCalculationServiceClient::new(
                    channel,
                ),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "chat-room-query" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `replay_show_status` keeps of each CreateShow response.
#[derive(Clone, Debug, Default)]
pub struct ShowStatusRecord {
    pub status: String,
    pub title: String,
    pub show_id: String,
}

/// Where `replay_show_status` hands each record.
trait ShowStatusWriter {
    fn deliver(&self, record: ShowStatusRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FeatureFlagHistoryService`.
struct FeatureFlagHistorySink {
    client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient<Channel>,
}

impl ShowStatusWriter for FeatureFlagHistorySink {
    fn deliver(&self, record: ShowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_feature_flag_versions(featureflags::ListFeatureFlagVersionsRequest {
                    order_by: record.show_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.chat.v1.ChatRoomQueryService`.
struct ChatRoomQuerySink {
    client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ShowStatusWriter for ChatRoomQuerySink {
    fn deliver(&self, record: ShowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(chat::PeekChatRoomRequest {
                    chat_room_id: record.title,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_chat_room(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.RateLimitCalculationService`.
struct RateLimitCalculationSink {
    client: ratelimiting::rate_limit_calculation_service_client::RateLimitCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ShowStatusWriter for RateLimitCalculationSink {
    fn deliver(&self, record: ShowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::SimulateRateLimitRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        description: record.status,
                        rate_limit_id: record.title,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.simulate_rate_limit(request).await?;
            Ok(())
        })
    }
}
