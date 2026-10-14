//! `forward_seat_maps_id` reads `acme.seating.v1.SeatMapQueryService/BrowseSeatMaps` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::helpdesk::v1 as helpdesk;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = seating::seat_map_query_service_client::SeatMapQueryServiceClient::new(channel.clone());
    let loaded = source
        .browse_seat_maps(seating::BrowseSeatMapsRequest {
            context: Some(context.clone()),
            order_by: "current-forward-seat-maps-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = SeatMapsIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: result.status,
    };
    dispatch_seat_maps_id("ticket", record, channel, &context).await
}

/// What `forward_seat_maps_id` keeps of each BrowseSeatMaps response.
#[derive(Clone, Debug, Default)]
pub struct SeatMapsIdRecord {
    pub correlation_id: String,
    pub status: String,
}

/// Where `forward_seat_maps_id` hands each record.
trait SeatMapsIdTarget {
    fn apply(&self, record: SeatMapsIdRecord) -> Delivery;
}

/// Hands records to `acme.labeling.v1.LabelingTaskQueryService`.
struct LabelingTaskQuerySink {
    client: labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapsIdTarget for LabelingTaskQuerySink {
    fn apply(&self, record: SeatMapsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(labeling::CountLabelingTasksRequest {
                    filter: Some(labeling::count_labeling_tasks_request::Filter {
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.count_labeling_tasks(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.admissions.v1.AdmissionCalculationService`.
struct AdmissionCalculationSink {
    client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapsIdTarget for AdmissionCalculationSink {
    fn apply(&self, record: SeatMapsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(admissions::GetAdmissionRatesRequest {
                    admission_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_admission_rates(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.helpdesk.v1.TicketService`.
struct TicketSink {
    client: helpdesk::ticket_service_client::TicketServiceClient<Channel>,
}

impl SeatMapsIdTarget for TicketSink {
    fn apply(&self, record: SeatMapsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_ticket(helpdesk::UpdateTicketRequest {
                    ticket: Some(helpdesk::Ticket {
                        subject: record.correlation_id,
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
struct SeatMapsIdLogOnly;

impl SeatMapsIdTarget for SeatMapsIdLogOnly {
    fn apply(&self, record: SeatMapsIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_seat_maps_id(
    kind: &str,
    record: SeatMapsIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let labeling_task_query = LabelingTaskQuerySink {
        client: labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let admission_calculation = AdmissionCalculationSink {
        client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient::new(
            channel.clone(),
        ),
        context: context.clone(),
    };
    let ticket = TicketSink {
        client: helpdesk::ticket_service_client::TicketServiceClient::new(channel),
    };
    let write: &dyn Fn(SeatMapsIdRecord) -> Delivery = match kind {
        "labeling-task-query" => &|record| labeling_task_query.apply(record),
        "admission-calculation" => &|record| admission_calculation.apply(record),
        "ticket" => &|record| ticket.apply(record),
        _ => &|record| SeatMapsIdLogOnly.apply(record),
    };
    write(record).await
}
