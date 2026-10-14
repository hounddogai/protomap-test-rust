//! `copy_rate_limits_id` reads `acme.ratelimiting.v1.RateLimitSyncService/ReconcileRateLimits` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient::new(channel.clone());
    let loaded = source
        .reconcile_rate_limits(ratelimiting::ReconcileRateLimitsRequest {
            context: Some(context.clone()),
            reason: "main-copy-rate-limits-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = loaded.outcome.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = RateLimitsIdRecord {
        reference_id: outcome.reference_id,
        correlation_id: result.correlation_id,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn RateLimitsIdDestination>)> = vec![
        (
            "appointment",
            Box::new(AppointmentSink {
                client: appointments::appointment_service_client::AppointmentServiceClient::new(channel.clone()),
                context: context.clone(),
            }),
        ),
        (
            "show",
            Box::new(ShowSink {
                client: shows::show_service_client::ShowServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "show" {
            continue;
        }
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `copy_rate_limits_id` keeps of each ReconcileRateLimits response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitsIdRecord {
    pub reference_id: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `copy_rate_limits_id` hands each record.
trait RateLimitsIdDestination {
    fn send(&self, record: RateLimitsIdRecord) -> Delivery;
}

/// Sends records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
    context: common::RequestContext,
}

impl RateLimitsIdDestination for AppointmentSink {
    fn send(&self, record: RateLimitsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(appointments::GetAppointmentRequest {
                    appointment_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_appointment(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.shows.v1.ShowService`.
struct ShowSink {
    client: shows::show_service_client::ShowServiceClient<Channel>,
    context: common::RequestContext,
}

impl RateLimitsIdDestination for ShowSink {
    fn send(&self, record: RateLimitsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(shows::ListShowsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_shows(request).await?;
            Ok(())
        })
    }
}
