//! `forward_seat_map_id` reads `acme.seating.v1.SeatMapQueryService/LookupSeatMap` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::promotions::v1 as promotions;
use acme_proto::acme::seating::v1 as seating;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = seating::seat_map_query_service_client::SeatMapQueryServiceClient::new(channel.clone());
    let loaded = source
        .lookup_seat_map(seating::LookupSeatMapRequest {
            context: Some(context.clone()),
            seat_map_id: "primary-forward-seat-map-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let seat_map = loaded.seat_map.unwrap_or_default();
    let record = SeatMapIdRecord {
        correlation_id: result.correlation_id,
        parent_id: acme_kit::text::normalize(&seat_map.parent_id),
        status: acme_kit::text::normalize(&result.status),
        name: seat_map.name,
    };
    act_on_seat_map_id("sponsor", record, channel, context).await
}

/// What `forward_seat_map_id` keeps of each LookupSeatMap response.
#[derive(Clone, Debug, Default)]
pub struct SeatMapIdRecord {
    pub correlation_id: String,
    pub parent_id: String,
    pub status: String,
    pub name: String,
}

/// Where `forward_seat_map_id` hands each record.
trait SeatMapIdDestination {
    fn send(&self, record: SeatMapIdRecord) -> Delivery;
}

/// Sends records to `acme.promotions.v1.PromotionService`.
struct PromotionSink {
    client: promotions::promotion_service_client::PromotionServiceClient<Channel>,
}

impl SeatMapIdDestination for PromotionSink {
    fn send(&self, record: SeatMapIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_promotions(promotions::SearchPromotionsRequest {
                    order_by: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.sponsorships.v1.SponsorService`.
struct SponsorSink {
    client: sponsorships::sponsor_service_client::SponsorServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapIdDestination for SponsorSink {
    fn send(&self, record: SeatMapIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sponsorships::CloneSponsorRequest {
                    sponsor: Some(sponsorships::Sponsor {
                        sponsor_id: record.name,
                        ..Default::default()
                    }),
                    request_id: record.parent_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.clone_sponsor(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_seat_map_id(
    kind: &str,
    record: SeatMapIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(SeatMapIdRecord) -> Delivery>> = BTreeMap::new();
    let promotion_channel = channel.clone();
    actions.insert(
        "promotion",
        Box::new(move |record| {
            PromotionSink {
                client: promotions::promotion_service_client::PromotionServiceClient::new(promotion_channel.clone()),
            }
            .send(record)
        }),
    );
    let sponsor_channel = channel.clone();
    let sponsor_context = context.clone();
    actions.insert(
        "sponsor",
        Box::new(move |record| {
            SponsorSink {
                client: sponsorships::sponsor_service_client::SponsorServiceClient::new(sponsor_channel.clone()),
                context: sponsor_context.clone(),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
