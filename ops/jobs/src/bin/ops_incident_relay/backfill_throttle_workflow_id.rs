//! `backfill_throttle_workflow_id` reads `acme.ratelimiting.v1.ThrottleWorkflowService/RetryThrottleWorkflow`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::search::v1 as search;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_workflow_service_client::ThrottleWorkflowServiceClient::new(channel.clone());
    let reply = source
        .retry_throttle_workflow(ratelimiting::RetryThrottleWorkflowRequest {
            context: Some(context.clone()),
            reason: "live-backfill-throttle-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let throttle = reply.throttle.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = ThrottleWorkflowIdRecord {
        correlation_id: throttle.correlation_id,
        source: throttle.source,
        message: result.message,
        outcome_message: outcome.message,
    };
    dispatch_throttle_workflow_id("search-index", record, channel, &context).await
}

/// What `backfill_throttle_workflow_id` keeps of each RetryThrottleWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleWorkflowIdRecord {
    pub correlation_id: String,
    pub source: String,
    pub message: String,
    pub outcome_message: String,
}

/// Where `backfill_throttle_workflow_id` hands each record.
trait ThrottleWorkflowIdDestination {
    fn send(&self, record: ThrottleWorkflowIdRecord) -> Delivery;
}

/// Sends records to `acme.search.v1.SearchIndexService`.
struct SearchIndexSink {
    client: search::search_index_service_client::SearchIndexServiceClient<Channel>,
}

impl ThrottleWorkflowIdDestination for SearchIndexSink {
    fn send(&self, record: ThrottleWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_search_index(search::UpdateSearchIndexRequest {
                    search_index: Some(search::SearchIndex {
                        name: record.correlation_id,
                        search_index_id: record.outcome_message,
                        code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.AccessPolicyQueryService`.
struct AccessPolicyQuerySink {
    client: authorization::access_policy_query_service_client::AccessPolicyQueryServiceClient<Channel>,
}

impl ThrottleWorkflowIdDestination for AccessPolicyQuerySink {
    fn send(&self, record: ThrottleWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_access_policies(authorization::CountAccessPoliciesRequest {
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    filter: Some(authorization::count_access_policies_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.EnvironmentCalculationService`.
struct EnvironmentCalculationSink {
    client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient<Channel>,
}

impl ThrottleWorkflowIdDestination for EnvironmentCalculationSink {
    fn send(&self, record: ThrottleWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_environment_options(configuration::CompareEnvironmentOptionsRequest {
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ThrottleWorkflowIdLogOnly;

impl ThrottleWorkflowIdDestination for ThrottleWorkflowIdLogOnly {
    fn send(&self, record: ThrottleWorkflowIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_throttle_workflow_id(
    kind: &str,
    record: ThrottleWorkflowIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let search_index = SearchIndexSink {
        client: search::search_index_service_client::SearchIndexServiceClient::new(channel.clone()),
    };
    let access_policy_query = AccessPolicyQuerySink {
        client: authorization::access_policy_query_service_client::AccessPolicyQueryServiceClient::new(channel.clone()),
    };
    let environment_calculation = EnvironmentCalculationSink {
        client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient::new(
            channel,
        ),
    };
    let write: &dyn Fn(ThrottleWorkflowIdRecord) -> Delivery = match kind {
        "search-index" => &|record| search_index.send(record),
        "access-policy-query" => &|record| access_policy_query.send(record),
        "environment-calculation" => &|record| environment_calculation.send(record),
        _ => &|record| ThrottleWorkflowIdLogOnly.send(record),
    };
    write(record).await
}
