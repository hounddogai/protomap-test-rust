//! `carry_notification_status` reads
//! `acme.notifications.v1.NotificationValidationService/PreviewNotification` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = notifications::notification_validation_service_client::NotificationValidationServiceClient::new(
        channel.clone(),
    );
    let found = source
        .preview_notification(notifications::PreviewNotificationRequest {
            context: Some(context.clone()),
            reason: "main-carry-notification-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let notification = found.notification.unwrap_or_default();
    let record = NotificationStatusRecord {
        status: result.status,
        outcome_status: outcome.status,
        notification_id: notification.notification_id,
        message: acme_kit::text::normalize(&result.message),
    };
    deliver_notification_status("vehicle-query", record, channel, &context).await
}

/// What `carry_notification_status` keeps of each PreviewNotification response.
#[derive(Clone, Debug, Default)]
pub struct NotificationStatusRecord {
    pub status: String,
    pub outcome_status: String,
    pub notification_id: String,
    pub message: String,
}

/// Where `carry_notification_status` hands each record.
trait NotificationStatusSink {
    fn send(&self, record: NotificationStatusRecord) -> Delivery;
}

/// Passes records to `acme.fleet.v1.VehicleQueryService`.
struct VehicleQuerySink {
    client: fleet::vehicle_query_service_client::VehicleQueryServiceClient<Channel>,
}

impl NotificationStatusSink for VehicleQuerySink {
    fn send(&self, record: NotificationStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_vehicles(fleet::BrowseVehiclesRequest {
                    filter: Some(fleet::browse_vehicles_request::Filter {
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

/// Writes records to `acme.seating.v1.SeatMapService`.
struct SeatMapSink {
    client: seating::seat_map_service_client::SeatMapServiceClient<Channel>,
    context: common::RequestContext,
}

impl NotificationStatusSink for SeatMapSink {
    fn send(&self, record: NotificationStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(seating::ListSeatMapsRequest {
                    filter: Some(seating::list_seat_maps_request::Filter {
                        owner_id: record.notification_id,
                        region: record.outcome_status,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_seat_maps(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct NotificationStatusLogOnly;

impl NotificationStatusSink for NotificationStatusLogOnly {
    fn send(&self, record: NotificationStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_notification_status(
    kind: &str,
    record: NotificationStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn NotificationStatusSink> = match kind {
        "vehicle-query" => Box::new(VehicleQuerySink {
            client: fleet::vehicle_query_service_client::VehicleQueryServiceClient::new(channel.clone()),
        }),
        "seat-map" => Box::new(SeatMapSink {
            client: seating::seat_map_service_client::SeatMapServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(NotificationStatusLogOnly),
    };
    sink.send(record).await
}
