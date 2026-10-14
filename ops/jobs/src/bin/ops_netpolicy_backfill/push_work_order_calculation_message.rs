//! `push_work_order_calculation_message` reads
//! `acme.maintenance.v1.WorkOrderCalculationService/ExplainWorkOrderCalculation` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::maintenance::v1 as maintenance;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient::new(channel.clone());
    let fetched = source
        .explain_work_order_calculation(maintenance::ExplainWorkOrderCalculationRequest {
            context: Some(context.clone()),
            work_order_id: "live-push-work-order-calculation-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = fetched.outcome.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = WorkOrderCalculationMessageRecord {
        message: outcome.message,
        status: result.status,
        result_message: acme_kit::text::normalize(&result.message),
    };
    let sinks: Vec<Box<dyn WorkOrderCalculationMessageTarget>> = vec![
        Box::new(SeatMapCalculationSink {
            client: seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient::new(channel.clone()),
        }),
        Box::new(FlagRolloutExportSink {
            client: featureflags::flag_rollout_export_service_client::FlagRolloutExportServiceClient::new(channel),
        }),
    ];
    sinks[1].write(record).await
}

/// What `push_work_order_calculation_message` keeps of each ExplainWorkOrderCalculation response.
#[derive(Clone, Debug, Default)]
pub struct WorkOrderCalculationMessageRecord {
    pub message: String,
    pub status: String,
    pub result_message: String,
}

/// Where `push_work_order_calculation_message` hands each record.
trait WorkOrderCalculationMessageTarget {
    fn write(&self, record: WorkOrderCalculationMessageRecord) -> Delivery;
}

/// Hands records to `acme.seating.v1.SeatMapCalculationService`.
struct SeatMapCalculationSink {
    client: seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient<Channel>,
}

impl WorkOrderCalculationMessageTarget for SeatMapCalculationSink {
    fn write(&self, record: WorkOrderCalculationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_seat_map(seating::CalculateSeatMapRequest {
                    details: Some(seating::calculate_seat_map_request::Details {
                        currency_code: record.result_message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagRolloutExportService`.
struct FlagRolloutExportSink {
    client: featureflags::flag_rollout_export_service_client::FlagRolloutExportServiceClient<Channel>,
}

impl WorkOrderCalculationMessageTarget for FlagRolloutExportSink {
    fn write(&self, record: WorkOrderCalculationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_flag_rollout_export(featureflags::GetFlagRolloutExportRequest {
                    flag_rollout_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
