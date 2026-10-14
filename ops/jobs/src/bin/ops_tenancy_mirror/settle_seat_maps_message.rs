//! `settle_seat_maps_message` reads `acme.seating.v1.SeatMapQueryService/FindSeatMaps` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::messaging::v1 as messaging;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = seating::seat_map_query_service_client::SeatMapQueryServiceClient::new(channel.clone());
    let fetched = source
        .find_seat_maps(seating::FindSeatMapsRequest {
            context: Some(context.clone()),
            order_by: "current-settle-seat-maps-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let page_info = fetched.page_info.unwrap_or_default();
    let record = SeatMapsMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
        status: result.status,
        next_page_token: page_info.next_page_token,
    };
    deliver_seat_maps_message("conversation-calculation", record, channel, &context).await
}

/// What `settle_seat_maps_message` keeps of each FindSeatMaps response.
#[derive(Clone, Debug, Default)]
pub struct SeatMapsMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `settle_seat_maps_message` hands each record.
trait SeatMapsMessageTarget {
    fn apply(&self, record: SeatMapsMessageRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.RateLimitApprovalService`.
struct RateLimitApprovalSink {
    client: ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient<Channel>,
}

impl SeatMapsMessageTarget for RateLimitApprovalSink {
    fn apply(&self, record: SeatMapsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .escalate_rate_limit(ratelimiting::EscalateRateLimitRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigSetValidationService`.
struct ConfigSetValidationSink {
    client: configuration::config_set_validation_service_client::ConfigSetValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapsMessageTarget for ConfigSetValidationSink {
    fn apply(&self, record: SeatMapsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::PreviewConfigSetRequest {
                    config_set_id: record.correlation_id,
                    reason: record.next_page_token,
                    config_set: Some(configuration::ConfigSet {
                        config_set_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.preview_config_set(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.messaging.v1.ConversationCalculationService`.
struct ConversationCalculationSink {
    client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient<Channel>,
}

impl SeatMapsMessageTarget for ConversationCalculationSink {
    fn apply(&self, record: SeatMapsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .quote_conversation(messaging::QuoteConversationRequest {
                    details: Some(messaging::quote_conversation_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    conversation_id: record.message,
                    conversation: Some(messaging::Conversation {
                        subject: record.status,
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
struct SeatMapsMessageLogOnly;

impl SeatMapsMessageTarget for SeatMapsMessageLogOnly {
    fn apply(&self, record: SeatMapsMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_seat_maps_message(
    kind: &str,
    record: SeatMapsMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn SeatMapsMessageTarget> = match kind {
        "rate-limit-approval" => Box::new(RateLimitApprovalSink {
            client: ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient::new(
                channel.clone(),
            ),
        }),
        "config-set-validation" => Box::new(ConfigSetValidationSink {
            client: configuration::config_set_validation_service_client::ConfigSetValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "conversation-calculation" => Box::new(ConversationCalculationSink {
            client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient::new(
                channel,
            ),
        }),
        _ => Box::new(SeatMapsMessageLogOnly),
    };
    sink.apply(record).await
}
