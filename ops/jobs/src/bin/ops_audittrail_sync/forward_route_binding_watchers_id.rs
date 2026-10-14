//! `forward_route_binding_watchers_id` reads
//! `acme.edgegateway.v1.RouteBindingNotificationService/NotifyRouteBindingWatchers` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::route_binding_notification_service_client::RouteBindingNotificationServiceClient::new(
        channel.clone(),
    );
    let loaded = source
        .notify_route_binding_watchers(edgegateway::NotifyRouteBindingWatchersRequest {
            context: Some(context.clone()),
            route_binding_id: "nightly-forward-route-binding-watchers-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let route_binding = loaded.route_binding.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = RouteBindingWatchersIdRecord {
        route_binding_id: route_binding.route_binding_id,
        reference_id: outcome.reference_id,
        description: acme_kit::text::normalize(&route_binding.description),
        status: outcome.status,
    };
    act_on_route_binding_watchers_id("rate-limit-reporting", record, channel, context).await
}

/// What `forward_route_binding_watchers_id` keeps of each NotifyRouteBindingWatchers response.
#[derive(Clone, Debug, Default)]
pub struct RouteBindingWatchersIdRecord {
    pub route_binding_id: String,
    pub reference_id: String,
    pub description: String,
    pub status: String,
}

/// Where `forward_route_binding_watchers_id` hands each record.
trait RouteBindingWatchersIdDestination {
    fn send(&self, record: RouteBindingWatchersIdRecord) -> Delivery;
}

/// Hands records to `acme.giftcards.v1.GiftCardCalculationService`.
struct GiftCardCalculationSink {
    client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient<Channel>,
}

impl RouteBindingWatchersIdDestination for GiftCardCalculationSink {
    fn send(&self, record: RouteBindingWatchersIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_gift_card(giftcards::CalculateGiftCardRequest {
                    gift_card: Some(giftcards::GiftCard {
                        gift_card_id: record.route_binding_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.RateLimitReportingService`.
struct RateLimitReportingSink {
    client: ratelimiting::rate_limit_reporting_service_client::RateLimitReportingServiceClient<Channel>,
}

impl RouteBindingWatchersIdDestination for RateLimitReportingSink {
    fn send(&self, record: RouteBindingWatchersIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_rate_limit_reports(ratelimiting::ListRateLimitReportsRequest {
                    filter: Some(ratelimiting::list_rate_limit_reports_request::Filter {
                        region: record.route_binding_id,
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
async fn act_on_route_binding_watchers_id(
    kind: &str,
    record: RouteBindingWatchersIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(RouteBindingWatchersIdRecord) -> Delivery>> = BTreeMap::new();
    let gift_card_calculation_channel = channel.clone();
    actions.insert(
        "gift-card-calculation",
        Box::new(move |record| {
            GiftCardCalculationSink {
                client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient::new(
                    gift_card_calculation_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    let rate_limit_reporting_channel = channel.clone();
    actions.insert(
        "rate-limit-reporting",
        Box::new(move |record| {
            RateLimitReportingSink {
                client: ratelimiting::rate_limit_reporting_service_client::RateLimitReportingServiceClient::new(
                    rate_limit_reporting_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
