//! `settle_csr_decision_id` reads `acme.certificates.v1.CsrValidationService/ExplainCsrDecision` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::csr_validation_service_client::CsrValidationServiceClient::new(channel.clone());
    let fetched = source
        .explain_csr_decision(certificates::ExplainCsrDecisionRequest {
            context: Some(context.clone()),
            csr_id: "default-settle-csr-decision-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let csr = fetched.csr.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = CsrDecisionIdRecord {
        csr_id: csr.csr_id,
        status: result.status,
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
        subject: csr.subject,
    };
    let sinks: Vec<(&str, Box<dyn CsrDecisionIdWriter>)> = vec![
        (
            "route-binding-gateway",
            Box::new(RouteBindingGatewaySink {
                client: edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "seat-map",
            Box::new(SeatMapSink {
                client: seating::seat_map_service_client::SeatMapServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "route-binding-gateway" {
            continue;
        }
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `settle_csr_decision_id` keeps of each ExplainCsrDecision response.
#[derive(Clone, Debug, Default)]
pub struct CsrDecisionIdRecord {
    pub csr_id: String,
    pub status: String,
    pub reference_id: String,
    pub subject: String,
}

/// Where `settle_csr_decision_id` hands each record.
trait CsrDecisionIdWriter {
    fn send(&self, record: CsrDecisionIdRecord) -> Delivery;
}

/// Passes records to `acme.edgegateway.v1.RouteBindingGatewayService`.
struct RouteBindingGatewaySink {
    client: edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient<Channel>,
}

impl CsrDecisionIdWriter for RouteBindingGatewaySink {
    fn send(&self, record: CsrDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_route_binding_checkout(edgegateway::StartRouteBindingCheckoutRequest {
                    route_binding: Some(edgegateway::RouteBinding {
                        route_binding_id: record.status,
                        description: record.csr_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.seating.v1.SeatMapService`.
struct SeatMapSink {
    client: seating::seat_map_service_client::SeatMapServiceClient<Channel>,
}

impl CsrDecisionIdWriter for SeatMapSink {
    fn send(&self, record: CsrDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_seat_map(seating::CloneSeatMapRequest {
                    seat_map: Some(seating::SeatMap {
                        parent_id: record.subject,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
