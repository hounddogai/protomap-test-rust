//! `carry_fulfillments_id` reads `acme.fulfillment.v1.FulfillmentQueryService/CountFulfillments` and hands
//! what it keeps to `acme.ratelimiting.v1.RateLimitWorkflowService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient::new(channel.clone());
    let loaded = source
        .count_fulfillments(fulfillment::CountFulfillmentsRequest {
            context: Some(context.clone()),
            order_by: "main-carry-fulfillments-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = FulfillmentsIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
    };
    RateLimitWorkflowSink {
        client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(channel),
        context: context.clone(),
    }
    .deliver(record)
    .await
}

/// What `carry_fulfillments_id` keeps of each CountFulfillments response.
#[derive(Clone, Debug, Default)]
pub struct FulfillmentsIdRecord {
    pub correlation_id: String,
    pub message: String,
}

/// Where `carry_fulfillments_id` hands each record.
trait FulfillmentsIdTarget {
    fn deliver(&self, record: FulfillmentsIdRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl FulfillmentsIdTarget for RateLimitWorkflowSink {
    fn deliver(&self, record: FulfillmentsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::CancelRateLimitWorkflowRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        name: record.message,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.cancel_rate_limit_workflow(request).await?;
            Ok(())
        })
    }
}
