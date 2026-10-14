//! `forward_seat_map_message` reads `acme.seating.v1.SeatMapService/DeleteSeatMap` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = seating::seat_map_service_client::SeatMapServiceClient::new(channel.clone());
    let fetched = source
        .delete_seat_map(seating::DeleteSeatMapRequest {
            context: Some(context.clone()),
            reason: "live-forward-seat-map-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = SeatMapMessageRecord {
        message: result.message,
        status: result.status,
    };
    let sinks: Vec<Box<dyn SeatMapMessageTarget>> = vec![
        Box::new(FlagEvaluationExportSink {
            client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(VehicleSink {
            client: fleet::vehicle_service_client::VehicleServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(RouteBindingQuerySink {
            client: edgegateway::route_binding_query_service_client::RouteBindingQueryServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    for sink in &sinks {
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `forward_seat_map_message` keeps of each DeleteSeatMap response.
#[derive(Clone, Debug, Default)]
pub struct SeatMapMessageRecord {
    pub message: String,
    pub status: String,
}

/// Where `forward_seat_map_message` hands each record.
trait SeatMapMessageTarget {
    fn push(&self, record: SeatMapMessageRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationExportService`.
struct FlagEvaluationExportSink {
    client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapMessageTarget for FlagEvaluationExportSink {
    fn push(&self, record: SeatMapMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::ScheduleFlagEvaluationExportRequest {
                    reason: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.schedule_flag_evaluation_export(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.fleet.v1.VehicleService`.
struct VehicleSink {
    client: fleet::vehicle_service_client::VehicleServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapMessageTarget for VehicleSink {
    fn push(&self, record: SeatMapMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fleet::GetVehicleRequest {
                    vehicle_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_vehicle(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.edgegateway.v1.RouteBindingQueryService`.
struct RouteBindingQuerySink {
    client: edgegateway::route_binding_query_service_client::RouteBindingQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapMessageTarget for RouteBindingQuerySink {
    fn push(&self, record: SeatMapMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::SummarizeRouteBindingsRequest {
                    details: Some(edgegateway::summarize_route_bindings_request::Details {
                        format: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_route_bindings(request).await?;
            Ok(())
        })
    }
}
