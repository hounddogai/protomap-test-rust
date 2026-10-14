//! `route_work_order_code` reads `acme.maintenance.v1.WorkOrderCalculationService/QuoteWorkOrder` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::maintenance::v1 as maintenance;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient::new(channel.clone());
    let found = source
        .quote_work_order(maintenance::QuoteWorkOrderRequest {
            context: Some(context.clone()),
            reason: "default-route-work-order-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = found.breakdown.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = WorkOrderCodeRecord {
        currency_code: acme_kit::text::normalize(&breakdown.currency_code),
        correlation_id: result.correlation_id,
        message: result.message,
    };
    dispatch_work_order_code("labeling-task-query", record, channel, &context).await
}

/// What `route_work_order_code` keeps of each QuoteWorkOrder response.
#[derive(Clone, Debug, Default)]
pub struct WorkOrderCodeRecord {
    pub currency_code: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `route_work_order_code` hands each record.
trait WorkOrderCodeSink {
    fn write(&self, record: WorkOrderCodeRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketCalculationService`.
struct QuotaBucketCalculationSink {
    client: ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient<Channel>,
}

impl WorkOrderCodeSink for QuotaBucketCalculationSink {
    fn write(&self, record: WorkOrderCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_quota_bucket(ratelimiting::SimulateQuotaBucketRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        quota_bucket_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.sellers.v1.SellerQueryService`.
struct SellerQuerySink {
    client: sellers::seller_query_service_client::SellerQueryServiceClient<Channel>,
}

impl WorkOrderCodeSink for SellerQuerySink {
    fn write(&self, record: WorkOrderCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_sellers(sellers::SuggestSellersRequest {
                    filter: Some(sellers::suggest_sellers_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.labeling.v1.LabelingTaskQueryService`.
struct LabelingTaskQuerySink {
    client: labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl WorkOrderCodeSink for LabelingTaskQuerySink {
    fn write(&self, record: WorkOrderCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(labeling::SuggestLabelingTasksRequest {
                    filter: Some(labeling::suggest_labeling_tasks_request::Filter {
                        query: record.message,
                        owner_id: record.currency_code,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.suggest_labeling_tasks(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct WorkOrderCodeLogOnly;

impl WorkOrderCodeSink for WorkOrderCodeLogOnly {
    fn write(&self, record: WorkOrderCodeRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.currency_code);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_work_order_code(
    kind: &str,
    record: WorkOrderCodeRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let quota_bucket_calculation = QuotaBucketCalculationSink {
        client: ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let seller_query = SellerQuerySink {
        client: sellers::seller_query_service_client::SellerQueryServiceClient::new(channel.clone()),
    };
    let labeling_task_query = LabelingTaskQuerySink {
        client: labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(WorkOrderCodeRecord) -> Delivery = match kind {
        "quota-bucket-calculation" => &|record| quota_bucket_calculation.write(record),
        "seller-query" => &|record| seller_query.write(record),
        "labeling-task-query" => &|record| labeling_task_query.write(record),
        _ => &|record| WorkOrderCodeLogOnly.write(record),
    };
    write(record).await
}
