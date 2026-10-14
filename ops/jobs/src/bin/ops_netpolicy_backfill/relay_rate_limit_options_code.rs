//! `relay_rate_limit_options_code` reads
//! `acme.ratelimiting.v1.RateLimitCalculationService/CompareRateLimitOptions` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::pricing::v1 as pricing;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::rate_limit_calculation_service_client::RateLimitCalculationServiceClient::new(channel.clone());
    let reply = source
        .compare_rate_limit_options(ratelimiting::CompareRateLimitOptionsRequest {
            context: Some(context.clone()),
            reason: "current-relay-rate-limit-options-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = reply.breakdown.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = RateLimitOptionsCodeRecord {
        currency_code: breakdown.currency_code,
        message: result.message,
    };
    act_on_rate_limit_options_code("config-set", record, channel, context).await
}

/// What `relay_rate_limit_options_code` keeps of each CompareRateLimitOptions response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitOptionsCodeRecord {
    pub currency_code: String,
    pub message: String,
}

/// Where `relay_rate_limit_options_code` hands each record.
trait RateLimitOptionsCodeWriter {
    fn write(&self, record: RateLimitOptionsCodeRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigSetService`.
struct ConfigSetSink {
    client: configuration::config_set_service_client::ConfigSetServiceClient<Channel>,
}

impl RateLimitOptionsCodeWriter for ConfigSetSink {
    fn write(&self, record: RateLimitOptionsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_config_sets(configuration::SearchConfigSetsRequest {
                    order_by: record.currency_code,
                    filter: Some(configuration::search_config_sets_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.admissions.v1.AdmissionQueryService`.
struct AdmissionQuerySink {
    client: admissions::admission_query_service_client::AdmissionQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl RateLimitOptionsCodeWriter for AdmissionQuerySink {
    fn write(&self, record: RateLimitOptionsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(admissions::PeekAdmissionRequest {
                    admission_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_admission(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.pricing.v1.PriceListService`.
struct PriceListSink {
    client: pricing::price_list_service_client::PriceListServiceClient<Channel>,
}

impl RateLimitOptionsCodeWriter for PriceListSink {
    fn write(&self, record: RateLimitOptionsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_price_list(pricing::GetPriceListRequest {
                    price_list_id: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_rate_limit_options_code(
    kind: &str,
    record: RateLimitOptionsCodeRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(RateLimitOptionsCodeRecord) -> Delivery>> = BTreeMap::new();
    let config_set_channel = channel.clone();
    actions.insert(
        "config-set",
        Box::new(move |record| {
            ConfigSetSink {
                client: configuration::config_set_service_client::ConfigSetServiceClient::new(
                    config_set_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let admission_query_channel = channel.clone();
    let admission_query_context = context.clone();
    actions.insert(
        "admission-query",
        Box::new(move |record| {
            AdmissionQuerySink {
                client: admissions::admission_query_service_client::AdmissionQueryServiceClient::new(
                    admission_query_channel.clone(),
                ),
                context: admission_query_context.clone(),
            }
            .write(record)
        }),
    );
    let price_list_channel = channel.clone();
    actions.insert(
        "price-list",
        Box::new(move |record| {
            PriceListSink {
                client: pricing::price_list_service_client::PriceListServiceClient::new(price_list_channel.clone()),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
