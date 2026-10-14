//! `mirror_search_index_code` reads `acme.search.v1.SearchIndexValidationService/ValidateSearchIndex` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::routing::v1 as routing;
use acme_proto::acme::search::v1 as search;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        search::search_index_validation_service_client::SearchIndexValidationServiceClient::new(channel.clone());
    let fetched = source
        .validate_search_index(search::ValidateSearchIndexRequest {
            context: Some(context.clone()),
            search_index_id: "current-mirror-search-index-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let search_index = fetched.search_index.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = SearchIndexCodeRecord {
        code: search_index.code,
        message: acme_kit::text::normalize(&result.message),
    };
    let sinks: Vec<Box<dyn SearchIndexCodeWriter>> = vec![
        Box::new(ExperimentCalculationSink {
            client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(LedgerAccountSink {
            client: ledger::ledger_account_service_client::LedgerAccountServiceClient::new(channel.clone()),
        }),
        Box::new(RouteQuerySink {
            client: routing::route_query_service_client::RouteQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(ConfigSetReportingSink {
            client: configuration::config_set_reporting_service_client::ConfigSetReportingServiceClient::new(channel),
        }),
    ];
    sinks[0].forward(record).await
}

/// What `mirror_search_index_code` keeps of each ValidateSearchIndex response.
#[derive(Clone, Debug, Default)]
pub struct SearchIndexCodeRecord {
    pub code: String,
    pub message: String,
}

/// Where `mirror_search_index_code` hands each record.
trait SearchIndexCodeWriter {
    fn forward(&self, record: SearchIndexCodeRecord) -> Delivery;
}

/// Sends records to `acme.experiments.v1.ExperimentCalculationService`.
struct ExperimentCalculationSink {
    client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SearchIndexCodeWriter for ExperimentCalculationSink {
    fn forward(&self, record: SearchIndexCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(experiments::QuoteExperimentRequest {
                    details: Some(experiments::quote_experiment_request::Details {
                        currency_code: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.quote_experiment(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ledger.v1.LedgerAccountService`.
struct LedgerAccountSink {
    client: ledger::ledger_account_service_client::LedgerAccountServiceClient<Channel>,
}

impl SearchIndexCodeWriter for LedgerAccountSink {
    fn forward(&self, record: SearchIndexCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_ledger_account(ledger::GetLedgerAccountRequest {
                    ledger_account_id: record.code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.routing.v1.RouteQueryService`.
struct RouteQuerySink {
    client: routing::route_query_service_client::RouteQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl SearchIndexCodeWriter for RouteQuerySink {
    fn forward(&self, record: SearchIndexCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(routing::SuggestRoutesRequest {
                    order_by: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.suggest_routes(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigSetReportingService`.
struct ConfigSetReportingSink {
    client: configuration::config_set_reporting_service_client::ConfigSetReportingServiceClient<Channel>,
}

impl SearchIndexCodeWriter for ConfigSetReportingSink {
    fn forward(&self, record: SearchIndexCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_config_set_totals(configuration::CountConfigSetTotalsRequest {
                    filter: Some(configuration::count_config_set_totals_request::Filter {
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
