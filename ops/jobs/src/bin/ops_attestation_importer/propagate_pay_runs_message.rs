//! `propagate_pay_runs_message` reads `acme.payroll.v1.PayRunService/ListPayRuns` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::payroll::v1 as payroll;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = payroll::pay_run_service_client::PayRunServiceClient::new(channel.clone());
    let found = source
        .list_pay_runs(payroll::ListPayRunsRequest {
            context: Some(context.clone()),
            order_by: "current-propagate-pay-runs-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = PayRunsMessageRecord {
        message: result.message,
        next_page_token: page_info.next_page_token,
        status: result.status,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    act_on_pay_runs_message("gateway-policy-workflow", record, channel, context).await
}

/// What `propagate_pay_runs_message` keeps of each ListPayRuns response.
#[derive(Clone, Debug, Default)]
pub struct PayRunsMessageRecord {
    pub message: String,
    pub next_page_token: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `propagate_pay_runs_message` hands each record.
trait PayRunsMessageWriter {
    fn forward(&self, record: PayRunsMessageRecord) -> Delivery;
}

/// Passes records to `acme.admissions.v1.AdmissionService`.
struct AdmissionSink {
    client: admissions::admission_service_client::AdmissionServiceClient<Channel>,
}

impl PayRunsMessageWriter for AdmissionSink {
    fn forward(&self, record: PayRunsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_admissions(admissions::ListAdmissionsRequest {
                    filter: Some(admissions::list_admissions_request::Filter {
                        owner_id: record.correlation_id,
                        query: record.next_page_token,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.edgegateway.v1.GatewayPolicyWorkflowService`.
struct GatewayPolicyWorkflowSink {
    client: edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl PayRunsMessageWriter for GatewayPolicyWorkflowSink {
    fn forward(&self, record: PayRunsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::CancelGatewayPolicyWorkflowRequest {
                    gateway_policy: Some(edgegateway::GatewayPolicy {
                        description: record.message,
                        ..Default::default()
                    }),
                    details: Some(edgegateway::cancel_gateway_policy_workflow_request::Details {
                        step: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.cancel_gateway_policy_workflow(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.deliveries.v1.DeliveryQueryService`.
struct DeliveryQuerySink {
    client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient<Channel>,
}

impl PayRunsMessageWriter for DeliveryQuerySink {
    fn forward(&self, record: PayRunsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_deliveries(deliveries::SuggestDeliveriesRequest {
                    order_by: record.message,
                    filter: Some(deliveries::suggest_deliveries_request::Filter {
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.GatewayPolicyReportingService`.
struct GatewayPolicyReportingSink {
    client: edgegateway::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient<Channel>,
}

impl PayRunsMessageWriter for GatewayPolicyReportingSink {
    fn forward(&self, record: PayRunsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .fetch_gateway_policy_report(edgegateway::FetchGatewayPolicyReportRequest {
                    gateway_policy_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_pay_runs_message(
    kind: &str,
    record: PayRunsMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(PayRunsMessageRecord) -> Delivery>> = BTreeMap::new();
    let admission_channel = channel.clone();
    actions.insert(
        "admission",
        Box::new(move |record| {
            AdmissionSink {
                client: admissions::admission_service_client::AdmissionServiceClient::new(admission_channel.clone()),
            }
            .forward(record)
        }),
    );
    let gateway_policy_workflow_channel = channel.clone();
    let gateway_policy_workflow_context = context.clone();
    actions.insert(
        "gateway-policy-workflow",
        Box::new(move |record| {
            GatewayPolicyWorkflowSink {
                client: edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient::new(
                    gateway_policy_workflow_channel.clone(),
                ),
                context: gateway_policy_workflow_context.clone(),
            }
            .forward(record)
        }),
    );
    let delivery_query_channel = channel.clone();
    actions.insert(
        "delivery-query",
        Box::new(move |record| {
            DeliveryQuerySink {
                client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(
                    delivery_query_channel.clone(),
                ),
            }
            .forward(record)
        }),
    );
    let gateway_policy_reporting_channel = channel.clone();
    actions.insert(
        "gateway-policy-reporting",
        Box::new(move |record| {
            GatewayPolicyReportingSink {
                client: edgegateway::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient::new(
                    gateway_policy_reporting_channel.clone(),
                ),
            }
            .forward(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
