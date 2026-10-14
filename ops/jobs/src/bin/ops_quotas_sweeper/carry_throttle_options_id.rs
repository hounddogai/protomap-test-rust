//! `carry_throttle_options_id` reads `acme.ratelimiting.v1.ThrottleCalculationService/CompareThrottleOptions`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::promotions::v1 as promotions;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel.clone());
    let found = source
        .compare_throttle_options(ratelimiting::CompareThrottleOptionsRequest {
            context: Some(context.clone()),
            throttle_id: "default-carry-throttle-options-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let throttle = found.throttle.unwrap_or_default();
    let record = ThrottleOptionsIdRecord {
        correlation_id: acme_kit::text::normalize(&throttle.correlation_id),
        source: acme_kit::text::normalize(&throttle.source),
    };
    let sinks: Vec<(&str, Box<dyn ThrottleOptionsIdWriter>)> = vec![
        (
            "config-set-calculation",
            Box::new(ConfigSetCalculationSink {
                client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "promotion",
            Box::new(PromotionSink {
                client: promotions::promotion_service_client::PromotionServiceClient::new(channel.clone()),
            }),
        ),
        (
            "environment-approval",
            Box::new(EnvironmentApprovalSink {
                client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient::new(
                    channel,
                ),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "promotion" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `carry_throttle_options_id` keeps of each CompareThrottleOptions response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleOptionsIdRecord {
    pub correlation_id: String,
    pub source: String,
}

/// Where `carry_throttle_options_id` hands each record.
trait ThrottleOptionsIdWriter {
    fn deliver(&self, record: ThrottleOptionsIdRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigSetCalculationService`.
struct ConfigSetCalculationSink {
    client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleOptionsIdWriter for ConfigSetCalculationSink {
    fn deliver(&self, record: ThrottleOptionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::CompareConfigSetOptionsRequest {
                    config_set: Some(configuration::ConfigSet {
                        config_set_id: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_config_set_options(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.promotions.v1.PromotionService`.
struct PromotionSink {
    client: promotions::promotion_service_client::PromotionServiceClient<Channel>,
}

impl ThrottleOptionsIdWriter for PromotionSink {
    fn deliver(&self, record: ThrottleOptionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_promotions(promotions::SearchPromotionsRequest {
                    order_by: record.source,
                    filter: Some(promotions::search_promotions_request::Filter {
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.EnvironmentApprovalService`.
struct EnvironmentApprovalSink {
    client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleOptionsIdWriter for EnvironmentApprovalSink {
    fn deliver(&self, record: ThrottleOptionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::DelegateEnvironmentApprovalRequest {
                    reason: record.correlation_id,
                    environment: Some(configuration::Environment {
                        code: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.delegate_environment_approval(request).await?;
            Ok(())
        })
    }
}
