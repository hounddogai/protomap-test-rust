//! `forward_throttle_rates_id` reads `acme.ratelimiting.v1.ThrottleCalculationService/GetThrottleRates` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel.clone());
    let reply = source
        .get_throttle_rates(ratelimiting::GetThrottleRatesRequest {
            context: Some(context.clone()),
            throttle_id: "live-forward-throttle-rates-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let throttle = reply.throttle.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = ThrottleRatesIdRecord {
        throttle_id: throttle.throttle_id,
        message: result.message,
    };
    act_on_throttle_rates_id("chat-room-query", record, channel, context).await
}

/// What `forward_throttle_rates_id` keeps of each GetThrottleRates response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleRatesIdRecord {
    pub throttle_id: String,
    pub message: String,
}

/// Where `forward_throttle_rates_id` hands each record.
trait ThrottleRatesIdSink {
    fn apply(&self, record: ThrottleRatesIdRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigChangeApprovalService`.
struct ConfigChangeApprovalSink {
    client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient<Channel>,
}

impl ThrottleRatesIdSink for ConfigChangeApprovalSink {
    fn apply(&self, record: ThrottleRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_pending_config_changes(configuration::ListPendingConfigChangesRequest {
                    filter: Some(configuration::list_pending_config_changes_request::Filter {
                        query: record.throttle_id,
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.KeyRotationCalculationService`.
struct KeyRotationCalculationSink {
    client: keymanagement::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient<Channel>,
}

impl ThrottleRatesIdSink for KeyRotationCalculationSink {
    fn apply(&self, record: ThrottleRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_key_rotations(keymanagement::RecalculateKeyRotationsRequest {
                    details: Some(keymanagement::recalculate_key_rotations_request::Details {
                        currency_code: record.throttle_id,
                        ..Default::default()
                    }),
                    key_rotation_id: record.message,
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
}

impl ThrottleRatesIdSink for ChatRoomQuerySink {
    fn apply(&self, record: ThrottleRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_chat_rooms(chat::CountChatRoomsRequest {
                    order_by: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_throttle_rates_id(
    kind: &str,
    record: ThrottleRatesIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(ThrottleRatesIdRecord) -> Delivery>> = BTreeMap::new();
    let config_change_approval_channel = channel.clone();
    actions.insert(
        "config-change-approval",
        Box::new(move |record| {
            ConfigChangeApprovalSink {
                client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient::new(
                    config_change_approval_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let key_rotation_calculation_channel = channel.clone();
    actions.insert(
        "key-rotation-calculation",
        Box::new(move |record| {
            KeyRotationCalculationSink {
                client:
                    keymanagement::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient::new(
                        key_rotation_calculation_channel.clone(),
                    ),
            }
            .apply(record)
        }),
    );
    let chat_room_query_channel = channel.clone();
    actions.insert(
        "chat-room-query",
        Box::new(move |record| {
            ChatRoomQuerySink {
                client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient::new(
                    chat_room_query_channel.clone(),
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
