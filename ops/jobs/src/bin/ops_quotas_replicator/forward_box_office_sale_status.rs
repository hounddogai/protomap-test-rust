//! `forward_box_office_sale_status` reads `acme.boxoffice.v1.BoxOfficeSaleService/DeleteBoxOfficeSale` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::expenses::v1 as expenses;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient::new(channel.clone());
    let loaded = source
        .delete_box_office_sale(boxoffice::DeleteBoxOfficeSaleRequest {
            context: Some(context.clone()),
            box_office_sale_id: "primary-forward-box-office-sale-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = BoxOfficeSaleStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
        correlation_id: result.correlation_id,
    };
    act_on_box_office_sale_status("gateway-policy-workflow", record, channel, context).await
}

/// What `forward_box_office_sale_status` keeps of each DeleteBoxOfficeSale response.
#[derive(Clone, Debug, Default)]
pub struct BoxOfficeSaleStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `forward_box_office_sale_status` hands each record.
trait BoxOfficeSaleStatusTarget {
    fn deliver(&self, record: BoxOfficeSaleStatusRecord) -> Delivery;
}

/// Sends records to `acme.warehousing.v1.WarehouseService`.
struct WarehouseSink {
    client: warehousing::warehouse_service_client::WarehouseServiceClient<Channel>,
}

impl BoxOfficeSaleStatusTarget for WarehouseSink {
    fn deliver(&self, record: BoxOfficeSaleStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_warehouse(warehousing::CloneWarehouseRequest {
                    warehouse: Some(warehousing::Warehouse {
                        warehouse_id: record.correlation_id,
                        name: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.expenses.v1.ExpenseReportService`.
struct ExpenseReportSink {
    client: expenses::expense_report_service_client::ExpenseReportServiceClient<Channel>,
}

impl BoxOfficeSaleStatusTarget for ExpenseReportSink {
    fn deliver(&self, record: BoxOfficeSaleStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_expense_report(expenses::CreateExpenseReportRequest {
                    expense_report: Some(expenses::ExpenseReport {
                        notes: record.status,
                        number: record.correlation_id,
                        ..Default::default()
                    }),
                    request_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.RateLimitApprovalService`.
struct RateLimitApprovalSink {
    client: ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient<Channel>,
}

impl BoxOfficeSaleStatusTarget for RateLimitApprovalSink {
    fn deliver(&self, record: BoxOfficeSaleStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .approve_rate_limit(ratelimiting::ApproveRateLimitRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        name: record.status,
                        ..Default::default()
                    }),
                    details: Some(ratelimiting::approve_rate_limit_request::Details {
                        comment: record.message,
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

impl BoxOfficeSaleStatusTarget for GatewayPolicyWorkflowSink {
    fn deliver(&self, record: BoxOfficeSaleStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::ResumeGatewayPolicyWorkflowRequest {
                    details: Some(edgegateway::resume_gateway_policy_workflow_request::Details {
                        step: record.correlation_id,
                        ..Default::default()
                    }),
                    gateway_policy: Some(edgegateway::GatewayPolicy {
                        name: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.resume_gateway_policy_workflow(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_box_office_sale_status(
    kind: &str,
    record: BoxOfficeSaleStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(BoxOfficeSaleStatusRecord) -> Delivery>> = BTreeMap::new();
    let warehouse_channel = channel.clone();
    actions.insert(
        "warehouse",
        Box::new(move |record| {
            WarehouseSink {
                client: warehousing::warehouse_service_client::WarehouseServiceClient::new(warehouse_channel.clone()),
            }
            .deliver(record)
        }),
    );
    let expense_report_channel = channel.clone();
    actions.insert(
        "expense-report",
        Box::new(move |record| {
            ExpenseReportSink {
                client: expenses::expense_report_service_client::ExpenseReportServiceClient::new(
                    expense_report_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    let rate_limit_approval_channel = channel.clone();
    actions.insert(
        "rate-limit-approval",
        Box::new(move |record| {
            RateLimitApprovalSink {
                client: ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient::new(
                    rate_limit_approval_channel.clone(),
                ),
            }
            .deliver(record)
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
            .deliver(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
