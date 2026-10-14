//! `backfill_fuel_card_message` reads `acme.fuel.v1.FuelCardQueryService/LookupFuelCard` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fuel::v1 as fuel;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fuel::fuel_card_query_service_client::FuelCardQueryServiceClient::new(channel.clone());
    let loaded = source
        .lookup_fuel_card(fuel::LookupFuelCardRequest {
            context: Some(context.clone()),
            fuel_card_id: "live-backfill-fuel-card-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let fuel_card = loaded.fuel_card.unwrap_or_default();
    let record = FuelCardMessageRecord {
        message: result.message,
        fuel_card_id: acme_kit::text::normalize(&fuel_card.fuel_card_id),
    };
    act_on_fuel_card_message("gift-card", record, channel, context).await
}

/// What `backfill_fuel_card_message` keeps of each LookupFuelCard response.
#[derive(Clone, Debug, Default)]
pub struct FuelCardMessageRecord {
    pub message: String,
    pub fuel_card_id: String,
}

/// Where `backfill_fuel_card_message` hands each record.
trait FuelCardMessageWriter {
    fn forward(&self, record: FuelCardMessageRecord) -> Delivery;
}

/// Writes records to `acme.giftcards.v1.GiftCardService`.
struct GiftCardSink {
    client: giftcards::gift_card_service_client::GiftCardServiceClient<Channel>,
}

impl FuelCardMessageWriter for GiftCardSink {
    fn forward(&self, record: FuelCardMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_gift_cards(giftcards::SearchGiftCardsRequest {
                    filter: Some(giftcards::search_gift_cards_request::Filter {
                        query: record.fuel_card_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketReportingService`.
struct QuotaBucketReportingSink {
    client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient<Channel>,
    context: common::RequestContext,
}

impl FuelCardMessageWriter for QuotaBucketReportingSink {
    fn forward(&self, record: FuelCardMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::CountQuotaBucketTotalsRequest {
                    filter: Some(ratelimiting::count_quota_bucket_totals_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.fuel_card_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.count_quota_bucket_totals(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_fuel_card_message(
    kind: &str,
    record: FuelCardMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(FuelCardMessageRecord) -> Delivery>> = BTreeMap::new();
    let gift_card_channel = channel.clone();
    actions.insert(
        "gift-card",
        Box::new(move |record| {
            GiftCardSink {
                client: giftcards::gift_card_service_client::GiftCardServiceClient::new(gift_card_channel.clone()),
            }
            .forward(record)
        }),
    );
    let quota_bucket_reporting_channel = channel.clone();
    let quota_bucket_reporting_context = context.clone();
    actions.insert(
        "quota-bucket-reporting",
        Box::new(move |record| {
            QuotaBucketReportingSink {
                client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient::new(
                    quota_bucket_reporting_channel.clone(),
                ),
                context: quota_bucket_reporting_context.clone(),
            }
            .forward(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
