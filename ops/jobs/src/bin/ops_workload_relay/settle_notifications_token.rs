//! `settle_notifications_token` reads `acme.notifications.v1.NotificationQueryService/BrowseNotifications`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::subscriptions::v1 as subscriptions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        notifications::notification_query_service_client::NotificationQueryServiceClient::new(channel.clone());
    let reply = source
        .browse_notifications(notifications::BrowseNotificationsRequest {
            context: Some(context.clone()),
            order_by: "main-settle-notifications-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = reply.page_info.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = NotificationsTokenRecord {
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<String, Box<dyn NotificationsTokenSink>> = BTreeMap::new();
    sinks.insert(
        "subscription".to_owned(),
        Box::new(SubscriptionSink {
            client: subscriptions::subscription_service_client::SubscriptionServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "throttle-reporting".to_owned(),
        Box::new(ThrottleReportingSink {
            client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "vehicle-query".to_owned(),
        Box::new(VehicleQuerySink {
            client: fleet::vehicle_query_service_client::VehicleQueryServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    let target = std::env::var("SETTLE_NOTIFICATIONS_TOKEN_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.deliver(record).await,
        None => Ok(()),
    }
}

/// What `settle_notifications_token` keeps of each BrowseNotifications response.
#[derive(Clone, Debug, Default)]
pub struct NotificationsTokenRecord {
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `settle_notifications_token` hands each record.
trait NotificationsTokenSink {
    fn deliver(&self, record: NotificationsTokenRecord) -> Delivery;
}

/// Writes records to `acme.subscriptions.v1.SubscriptionService`.
struct SubscriptionSink {
    client: subscriptions::subscription_service_client::SubscriptionServiceClient<Channel>,
}

impl NotificationsTokenSink for SubscriptionSink {
    fn deliver(&self, record: NotificationsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_subscription(subscriptions::UpdateSubscriptionRequest {
                    subscription: Some(subscriptions::Subscription {
                        subscription_id: record.next_page_token,
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.ThrottleReportingService`.
struct ThrottleReportingSink {
    client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient<Channel>,
}

impl NotificationsTokenSink for ThrottleReportingSink {
    fn deliver(&self, record: NotificationsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_throttle_totals(ratelimiting::CountThrottleTotalsRequest {
                    order_by: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.fleet.v1.VehicleQueryService`.
struct VehicleQuerySink {
    client: fleet::vehicle_query_service_client::VehicleQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl NotificationsTokenSink for VehicleQuerySink {
    fn deliver(&self, record: NotificationsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fleet::BrowseVehiclesRequest {
                    filter: Some(fleet::browse_vehicles_request::Filter {
                        query: record.correlation_id,
                        owner_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_vehicles(request).await?;
            Ok(())
        })
    }
}
