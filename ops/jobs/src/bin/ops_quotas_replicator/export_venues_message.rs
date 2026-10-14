//! `export_venues_message` reads `acme.venues.v1.VenueValidationService/LintVenues` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::checkout::v1 as checkout;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = venues::venue_validation_service_client::VenueValidationServiceClient::new(channel.clone());
    let found = source
        .lint_venues(venues::LintVenuesRequest {
            context: Some(context.clone()),
            reason: "live-export-venues-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let venue = found.venue.unwrap_or_default();
    let record = VenuesMessageRecord {
        message: result.message,
        venue_id: acme_kit::text::normalize(&venue.venue_id),
    };
    let mut sinks: BTreeMap<&str, Box<dyn VenuesMessageTarget>> = BTreeMap::new();
    sinks.insert(
        "notification-query",
        Box::new(NotificationQuerySink {
            client: notifications::notification_query_service_client::NotificationQueryServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "checkout-session-query",
        Box::new(CheckoutSessionQuerySink {
            client: checkout::checkout_session_query_service_client::CheckoutSessionQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("notification-query") {
        Some(sink) => sink.deliver(record).await,
        None => anyhow::bail!("no sink registered for notification-query"),
    }
}

/// What `export_venues_message` keeps of each LintVenues response.
#[derive(Clone, Debug, Default)]
pub struct VenuesMessageRecord {
    pub message: String,
    pub venue_id: String,
}

/// Where `export_venues_message` hands each record.
trait VenuesMessageTarget {
    fn deliver(&self, record: VenuesMessageRecord) -> Delivery;
}

/// Passes records to `acme.notifications.v1.NotificationQueryService`.
struct NotificationQuerySink {
    client: notifications::notification_query_service_client::NotificationQueryServiceClient<Channel>,
}

impl VenuesMessageTarget for NotificationQuerySink {
    fn deliver(&self, record: VenuesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_notifications(notifications::FindNotificationsRequest {
                    filter: Some(notifications::find_notifications_request::Filter {
                        query: record.venue_id,
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.checkout.v1.CheckoutSessionQueryService`.
struct CheckoutSessionQuerySink {
    client: checkout::checkout_session_query_service_client::CheckoutSessionQueryServiceClient<Channel>,
}

impl VenuesMessageTarget for CheckoutSessionQuerySink {
    fn deliver(&self, record: VenuesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_checkout_sessions(checkout::BrowseCheckoutSessionsRequest {
                    filter: Some(checkout::browse_checkout_sessions_request::Filter {
                        query: record.venue_id,
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
