//! `sync_vehicles_status` reads `acme.fleet.v1.VehicleService/ListVehicles` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fleet::v1 as fleet;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fleet::vehicle_service_client::VehicleServiceClient::new(channel.clone());
    let fetched = source
        .list_vehicles(fleet::ListVehiclesRequest {
            context: Some(context.clone()),
            order_by: "current-sync-vehicles-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = VehiclesStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<(&str, Box<dyn VehiclesStatusWriter>)> = vec![
        (
            "box-office-sale-query",
            Box::new(BoxOfficeSaleQuerySink {
                client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "flag-evaluation-sync",
            Box::new(FlagEvaluationSyncSink {
                client: featureflags::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "gateway-policy-workflow",
            Box::new(GatewayPolicyWorkflowSink {
                client: edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "gateway-policy-workflow" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `sync_vehicles_status` keeps of each ListVehicles response.
#[derive(Clone, Debug, Default)]
pub struct VehiclesStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `sync_vehicles_status` hands each record.
trait VehiclesStatusWriter {
    fn apply(&self, record: VehiclesStatusRecord) -> Delivery;
}

/// Writes records to `acme.boxoffice.v1.BoxOfficeSaleQueryService`.
struct BoxOfficeSaleQuerySink {
    client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl VehiclesStatusWriter for BoxOfficeSaleQuerySink {
    fn apply(&self, record: VehiclesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(boxoffice::SuggestBoxOfficeSalesRequest {
                    filter: Some(boxoffice::suggest_box_office_sales_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suggest_box_office_sales(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagEvaluationSyncService`.
struct FlagEvaluationSyncSink {
    client: featureflags::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient<Channel>,
    context: common::RequestContext,
}

impl VehiclesStatusWriter for FlagEvaluationSyncSink {
    fn apply(&self, record: VehiclesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::ListFlagEvaluationConflictsRequest {
                    order_by: record.correlation_id,
                    filter: Some(featureflags::list_flag_evaluation_conflicts_request::Filter {
                        owner_id: record.status,
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_flag_evaluation_conflicts(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.edgegateway.v1.GatewayPolicyWorkflowService`.
struct GatewayPolicyWorkflowSink {
    client: edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient<Channel>,
}

impl VehiclesStatusWriter for GatewayPolicyWorkflowSink {
    fn apply(&self, record: VehiclesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resume_gateway_policy_workflow(edgegateway::ResumeGatewayPolicyWorkflowRequest {
                    details: Some(edgegateway::resume_gateway_policy_workflow_request::Details {
                        step: record.correlation_id,
                        ..Default::default()
                    }),
                    reason: record.message,
                    gateway_policy: Some(edgegateway::GatewayPolicy {
                        gateway_policy_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
