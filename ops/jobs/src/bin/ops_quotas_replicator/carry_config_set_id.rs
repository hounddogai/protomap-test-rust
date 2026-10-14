//! `carry_config_set_id` reads `acme.configuration.v1.ConfigSetService/UpdateConfigSet` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::config_set_service_client::ConfigSetServiceClient::new(channel.clone());
    let fetched = source
        .update_config_set(configuration::UpdateConfigSetRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let config_set = fetched.config_set.unwrap_or_default();
    let record = ConfigSetIdRecord {
        correlation_id: result.correlation_id,
        parent_id: config_set.parent_id,
        message: result.message,
        config_set_id: config_set.config_set_id,
    };
    dispatch_config_set_id("sponsor-calculation", record, channel, &context).await
}

/// What `carry_config_set_id` keeps of each UpdateConfigSet response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetIdRecord {
    pub correlation_id: String,
    pub parent_id: String,
    pub message: String,
    pub config_set_id: String,
}

/// Where `carry_config_set_id` hands each record.
trait ConfigSetIdDestination {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery;
}

/// Passes records to `acme.chat.v1.ChatRoomService`.
struct ChatRoomSink {
    client: chat::chat_room_service_client::ChatRoomServiceClient<Channel>,
}

impl ConfigSetIdDestination for ChatRoomSink {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_chat_rooms(chat::SearchChatRoomsRequest {
                    filter: Some(chat::search_chat_rooms_request::Filter {
                        region: record.correlation_id,
                        owner_id: record.config_set_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.sponsorships.v1.SponsorQueryService`.
struct SponsorQuerySink {
    client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetIdDestination for SponsorQuerySink {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sponsorships::CountSponsorsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.count_sponsors(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.sponsorships.v1.SponsorCalculationService`.
struct SponsorCalculationSink {
    client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient<Channel>,
}

impl ConfigSetIdDestination for SponsorCalculationSink {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_sponsor(sponsorships::EstimateSponsorRequest {
                    sponsor: Some(sponsorships::Sponsor {
                        sponsor_id: record.message,
                        ..Default::default()
                    }),
                    sponsor_id: record.config_set_id,
                    reason: record.parent_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRingValidationService`.
struct KeyRingValidationSink {
    client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient<Channel>,
}

impl ConfigSetIdDestination for KeyRingValidationSink {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_key_ring_rules(keymanagement::GetKeyRingRulesRequest {
                    key_ring_id: record.parent_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ConfigSetIdLogOnly;

impl ConfigSetIdDestination for ConfigSetIdLogOnly {
    fn apply(&self, record: ConfigSetIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_config_set_id(
    kind: &str,
    record: ConfigSetIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let chat_room = ChatRoomSink {
        client: chat::chat_room_service_client::ChatRoomServiceClient::new(channel.clone()),
    };
    let sponsor_query = SponsorQuerySink {
        client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let sponsor_calculation = SponsorCalculationSink {
        client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient::new(channel.clone()),
    };
    let key_ring_validation = KeyRingValidationSink {
        client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient::new(channel),
    };
    let write: &dyn Fn(ConfigSetIdRecord) -> Delivery = match kind {
        "chat-room" => &|record| chat_room.apply(record),
        "sponsor-query" => &|record| sponsor_query.apply(record),
        "sponsor-calculation" => &|record| sponsor_calculation.apply(record),
        "key-ring-validation" => &|record| key_ring_validation.apply(record),
        _ => &|record| ConfigSetIdLogOnly.apply(record),
    };
    write(record).await
}
