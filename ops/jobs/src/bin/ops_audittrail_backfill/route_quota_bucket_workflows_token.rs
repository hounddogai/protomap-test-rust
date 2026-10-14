//! `route_quota_bucket_workflows_token` reads
//! `acme.ratelimiting.v1.QuotaBucketWorkflowService/ListQuotaBucketWorkflows` and hands what it keeps to
//! `acme.careplans.v1.CarePlanQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient::new(channel.clone());
    let found = source
        .list_quota_bucket_workflows(ratelimiting::ListQuotaBucketWorkflowsRequest {
            context: Some(context.clone()),
            order_by: "main-route-quota-bucket-workflows-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = found.page_info.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = QuotaBucketWorkflowsTokenRecord {
        next_page_token: page_info.next_page_token,
        message: result.message,
        correlation_id: result.correlation_id,
        status: acme_kit::text::normalize(&result.status),
    };
    CarePlanQuerySink {
        client: careplans::care_plan_query_service_client::CarePlanQueryServiceClient::new(channel),
        context: context.clone(),
    }
    .write(record)
    .await
}

/// What `route_quota_bucket_workflows_token` keeps of each ListQuotaBucketWorkflows response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketWorkflowsTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `route_quota_bucket_workflows_token` hands each record.
trait QuotaBucketWorkflowsTokenWriter {
    fn write(&self, record: QuotaBucketWorkflowsTokenRecord) -> Delivery;
}

/// Sends records to `acme.careplans.v1.CarePlanQueryService`.
struct CarePlanQuerySink {
    client: careplans::care_plan_query_service_client::CarePlanQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl QuotaBucketWorkflowsTokenWriter for CarePlanQuerySink {
    fn write(&self, record: QuotaBucketWorkflowsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(careplans::SuggestCarePlansRequest {
                    filter: Some(careplans::suggest_care_plans_request::Filter {
                        owner_id: record.next_page_token,
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suggest_care_plans(request).await?;
            Ok(())
        })
    }
}
