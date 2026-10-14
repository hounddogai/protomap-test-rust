//! `propagate_environment_reports_message` reads
//! `acme.configuration.v1.EnvironmentReportingService/ListEnvironmentReports` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::customers::v1 as customers;
use acme_proto::acme::expenses::v1 as expenses;
use acme_proto::acme::labeling::v1 as labeling;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::environment_reporting_service_client::EnvironmentReportingServiceClient::new(channel.clone());
    let reply = source
        .list_environment_reports(configuration::ListEnvironmentReportsRequest {
            context: Some(context.clone()),
            order_by: "main-propagate-environment-reports-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = EnvironmentReportsMessageRecord {
        message: result.message,
        next_page_token: page_info.next_page_token,
        correlation_id: result.correlation_id,
    };
    act_on_environment_reports_message("customer-query", record, channel, context).await
}

/// What `propagate_environment_reports_message` keeps of each ListEnvironmentReports response.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentReportsMessageRecord {
    pub message: String,
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `propagate_environment_reports_message` hands each record.
trait EnvironmentReportsMessageSink {
    fn send(&self, record: EnvironmentReportsMessageRecord) -> Delivery;
}

/// Writes records to `acme.customers.v1.CustomerQueryService`.
struct CustomerQuerySink {
    client: customers::customer_query_service_client::CustomerQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl EnvironmentReportsMessageSink for CustomerQuerySink {
    fn send(&self, record: EnvironmentReportsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(customers::BrowseCustomersRequest {
                    filter: Some(customers::browse_customers_request::Filter {
                        owner_id: record.message,
                        region: record.next_page_token,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_customers(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.labeling.v1.LabelingTaskService`.
struct LabelingTaskSink {
    client: labeling::labeling_task_service_client::LabelingTaskServiceClient<Channel>,
}

impl EnvironmentReportsMessageSink for LabelingTaskSink {
    fn send(&self, record: EnvironmentReportsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_labeling_task(labeling::GetLabelingTaskRequest {
                    labeling_task_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.expenses.v1.ExpenseReportService`.
struct ExpenseReportSink {
    client: expenses::expense_report_service_client::ExpenseReportServiceClient<Channel>,
    context: common::RequestContext,
}

impl EnvironmentReportsMessageSink for ExpenseReportSink {
    fn send(&self, record: EnvironmentReportsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(expenses::CreateExpenseReportRequest {
                    request_id: record.message,
                    expense_report: Some(expenses::ExpenseReport {
                        notes: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.create_expense_report(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_environment_reports_message(
    kind: &str,
    record: EnvironmentReportsMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(EnvironmentReportsMessageRecord) -> Delivery>> = BTreeMap::new();
    let customer_query_channel = channel.clone();
    let customer_query_context = context.clone();
    actions.insert(
        "customer-query",
        Box::new(move |record| {
            CustomerQuerySink {
                client: customers::customer_query_service_client::CustomerQueryServiceClient::new(
                    customer_query_channel.clone(),
                ),
                context: customer_query_context.clone(),
            }
            .send(record)
        }),
    );
    let labeling_task_channel = channel.clone();
    actions.insert(
        "labeling-task",
        Box::new(move |record| {
            LabelingTaskSink {
                client: labeling::labeling_task_service_client::LabelingTaskServiceClient::new(
                    labeling_task_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    let expense_report_channel = channel.clone();
    let expense_report_context = context.clone();
    actions.insert(
        "expense-report",
        Box::new(move |record| {
            ExpenseReportSink {
                client: expenses::expense_report_service_client::ExpenseReportServiceClient::new(
                    expense_report_channel.clone(),
                ),
                context: expense_report_context.clone(),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
