//! `sync_pay_run_status` reads `acme.payroll.v1.PayRunService/UpdatePayRun` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::payroll::v1 as payroll;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = payroll::pay_run_service_client::PayRunServiceClient::new(channel.clone());
    let reply = source
        .update_pay_run(payroll::UpdatePayRunRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let pay_run = reply.pay_run.unwrap_or_default();
    let record = PayRunStatusRecord {
        status: result.status,
        number: acme_kit::text::normalize(&pay_run.number),
        message: result.message,
    };
    let sinks: Vec<(&str, Box<dyn PayRunStatusWriter>)> = vec![
        (
            "vehicle",
            Box::new(VehicleSink {
                client: fleet::vehicle_service_client::VehicleServiceClient::new(channel.clone()),
                context: context.clone(),
            }),
        ),
        (
            "delivery-query",
            Box::new(DeliveryQuerySink {
                client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "vehicle" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `sync_pay_run_status` keeps of each UpdatePayRun response.
#[derive(Clone, Debug, Default)]
pub struct PayRunStatusRecord {
    pub status: String,
    pub number: String,
    pub message: String,
}

/// Where `sync_pay_run_status` hands each record.
trait PayRunStatusWriter {
    fn push(&self, record: PayRunStatusRecord) -> Delivery;
}

/// Passes records to `acme.fleet.v1.VehicleService`.
struct VehicleSink {
    client: fleet::vehicle_service_client::VehicleServiceClient<Channel>,
    context: common::RequestContext,
}

impl PayRunStatusWriter for VehicleSink {
    fn push(&self, record: PayRunStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fleet::ListVehiclesRequest {
                    filter: Some(fleet::list_vehicles_request::Filter {
                        owner_id: record.number,
                        ..Default::default()
                    }),
                    order_by: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_vehicles(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.deliveries.v1.DeliveryQueryService`.
struct DeliveryQuerySink {
    client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl PayRunStatusWriter for DeliveryQuerySink {
    fn push(&self, record: PayRunStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(deliveries::BrowseDeliveriesRequest {
                    filter: Some(deliveries::browse_deliveries_request::Filter {
                        query: record.message,
                        region: record.number,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_deliveries(request).await?;
            Ok(())
        })
    }
}
