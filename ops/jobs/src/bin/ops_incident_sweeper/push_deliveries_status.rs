//! `push_deliveries_status` reads `acme.deliveries.v1.DeliveryQueryService/CountDeliveries` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel.clone());
    let reply = source
        .count_deliveries(deliveries::CountDeliveriesRequest {
            context: Some(context.clone()),
            order_by: "primary-push-deliveries-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = DeliveriesStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
        message: acme_kit::text::normalize(&result.message),
    };
    let sinks: Vec<Box<dyn DeliveriesStatusDestination>> = vec![
        Box::new(NotificationQuerySink { client: notifications::notification_query_service_client::NotificationQueryServiceClient::new(channel.clone()) }),
        Box::new(PermissionCheckNotificationSink { client: authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(channel.clone()), context: context.clone() }),
        Box::new(VenueValidationSink { client: venues::venue_validation_service_client::VenueValidationServiceClient::new(channel) }),
    ];
    sinks[2].apply(record).await
}

/// What `push_deliveries_status` keeps of each CountDeliveries response.
#[derive(Clone, Debug, Default)]
pub struct DeliveriesStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `push_deliveries_status` hands each record.
trait DeliveriesStatusDestination {
    fn apply(&self, record: DeliveriesStatusRecord) -> Delivery;
}

/// Writes records to `acme.notifications.v1.NotificationQueryService`.
struct NotificationQuerySink {
    client: notifications::notification_query_service_client::NotificationQueryServiceClient<Channel>,
}

impl DeliveriesStatusDestination for NotificationQuerySink {
    fn apply(&self, record: DeliveriesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_notifications(notifications::BrowseNotificationsRequest {
                    filter: Some(notifications::browse_notifications_request::Filter {
                        query: record.status,
                        region: record.message,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.PermissionCheckNotificationService`.
struct PermissionCheckNotificationSink {
    client:
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl DeliveriesStatusDestination for PermissionCheckNotificationSink {
    fn apply(&self, record: DeliveriesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::ListPermissionCheckSubscriptionsRequest {
                    order_by: record.message,
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    filter: Some(authorization::list_permission_check_subscriptions_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_permission_check_subscriptions(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.venues.v1.VenueValidationService`.
struct VenueValidationSink {
    client: venues::venue_validation_service_client::VenueValidationServiceClient<Channel>,
}

impl DeliveriesStatusDestination for VenueValidationSink {
    fn apply(&self, record: DeliveriesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_venue(venues::NormalizeVenueRequest {
                    venue: Some(venues::Venue {
                        code: record.message,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
