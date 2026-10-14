//! `settle_show_rates_id` reads `acme.shows.v1.ShowCalculationService/GetShowRates` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = shows::show_calculation_service_client::ShowCalculationServiceClient::new(channel.clone());
    let loaded = source
        .get_show_rates(shows::GetShowRatesRequest {
            context: Some(context.clone()),
            show_id: "current-settle-show-rates-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let show = loaded.show.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = ShowRatesIdRecord {
        show_id: acme_kit::text::normalize(&show.show_id),
        status: acme_kit::text::normalize(&result.status),
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    dispatch_show_rates_id("search-index-validation", record, channel, &context).await
}

/// What `settle_show_rates_id` keeps of each GetShowRates response.
#[derive(Clone, Debug, Default)]
pub struct ShowRatesIdRecord {
    pub show_id: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `settle_show_rates_id` hands each record.
trait ShowRatesIdWriter {
    fn write(&self, record: ShowRatesIdRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketWorkflowService`.
struct QuotaBucketWorkflowSink {
    client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient<Channel>,
}

impl ShowRatesIdWriter for QuotaBucketWorkflowSink {
    fn write(&self, record: ShowRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .inspect_quota_bucket_workflow(ratelimiting::InspectQuotaBucketWorkflowRequest {
                    quota_bucket_id: record.show_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.search.v1.SearchIndexValidationService`.
struct SearchIndexValidationSink {
    client: search::search_index_validation_service_client::SearchIndexValidationServiceClient<Channel>,
}

impl ShowRatesIdWriter for SearchIndexValidationSink {
    fn write(&self, record: ShowRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_search_index_decision(search::ExplainSearchIndexDecisionRequest {
                    details: Some(search::explain_search_index_decision_request::Details {
                        locale: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ShowRatesIdLogOnly;

impl ShowRatesIdWriter for ShowRatesIdLogOnly {
    fn write(&self, record: ShowRatesIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.show_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_show_rates_id(
    kind: &str,
    record: ShowRatesIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let quota_bucket_workflow = QuotaBucketWorkflowSink {
        client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient::new(
            channel.clone(),
        ),
    };
    let search_index_validation = SearchIndexValidationSink {
        client: search::search_index_validation_service_client::SearchIndexValidationServiceClient::new(channel),
    };
    let write: &dyn Fn(ShowRatesIdRecord) -> Delivery = match kind {
        "quota-bucket-workflow" => &|record| quota_bucket_workflow.write(record),
        "search-index-validation" => &|record| search_index_validation.write(record),
        _ => &|record| ShowRatesIdLogOnly.write(record),
    };
    write(record).await
}
