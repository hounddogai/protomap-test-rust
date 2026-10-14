//! `sync_price_lists_status` reads `acme.pricing.v1.PriceListService/SearchPriceLists` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::pricing::v1 as pricing;
use acme_proto::acme::search::v1 as search;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = pricing::price_list_service_client::PriceListServiceClient::new(channel.clone());
    let loaded = source
        .search_price_lists(pricing::SearchPriceListsRequest {
            context: Some(context.clone()),
            order_by: "live-sync-price-lists-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = PriceListsStatusRecord {
        status: result.status,
        message: acme_kit::text::normalize(&result.message),
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
    };
    let mut sinks: BTreeMap<String, Box<dyn PriceListsStatusSink>> = BTreeMap::new();
    sinks.insert(
        "flag-evaluation-calculation".to_owned(),
        Box::new(FlagEvaluationCalculationSink {
            client:
                featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient::new(
                    channel.clone(),
                ),
        }),
    );
    sinks.insert(
        "config-change-query".to_owned(),
        Box::new(ConfigChangeQuerySink {
            client: configuration::config_change_query_service_client::ConfigChangeQueryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "search-index-query".to_owned(),
        Box::new(SearchIndexQuerySink {
            client: search::search_index_query_service_client::SearchIndexQueryServiceClient::new(channel),
        }),
    );
    let target = std::env::var("SYNC_PRICE_LISTS_STATUS_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.push(record).await,
        None => Ok(()),
    }
}

/// What `sync_price_lists_status` keeps of each SearchPriceLists response.
#[derive(Clone, Debug, Default)]
pub struct PriceListsStatusRecord {
    pub status: String,
    pub message: String,
    pub next_page_token: String,
}

/// Where `sync_price_lists_status` hands each record.
trait PriceListsStatusSink {
    fn push(&self, record: PriceListsStatusRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationCalculationService`.
struct FlagEvaluationCalculationSink {
    client: featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient<Channel>,
}

impl PriceListsStatusSink for FlagEvaluationCalculationSink {
    fn push(&self, record: PriceListsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_flag_evaluations(featureflags::RecalculateFlagEvaluationsRequest {
                    details: Some(featureflags::recalculate_flag_evaluations_request::Details {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        correlation_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigChangeQueryService`.
struct ConfigChangeQuerySink {
    client: configuration::config_change_query_service_client::ConfigChangeQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl PriceListsStatusSink for ConfigChangeQuerySink {
    fn push(&self, record: PriceListsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::PeekConfigChangeRequest {
                    config_change_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_config_change(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.search.v1.SearchIndexQueryService`.
struct SearchIndexQuerySink {
    client: search::search_index_query_service_client::SearchIndexQueryServiceClient<Channel>,
}

impl PriceListsStatusSink for SearchIndexQuerySink {
    fn push(&self, record: PriceListsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_search_indexes(search::BrowseSearchIndexesRequest {
                    order_by: record.next_page_token,
                    filter: Some(search::browse_search_indexes_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
