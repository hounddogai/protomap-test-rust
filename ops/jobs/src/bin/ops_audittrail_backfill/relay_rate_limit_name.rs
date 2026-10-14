//! `relay_rate_limit_name` reads `acme.ratelimiting.v1.RateLimitQueryService/LookupRateLimit` and hands what
//! it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::rate_limit_query_service_client::RateLimitQueryServiceClient::new(channel.clone());
    let reply = source
        .lookup_rate_limit(ratelimiting::LookupRateLimitRequest {
            context: Some(context.clone()),
            rate_limit_id: "live-relay-rate-limit-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let rate_limit = reply.rate_limit.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = RateLimitNameRecord {
        name: rate_limit.name,
        message: result.message,
        rate_limit_id: rate_limit.rate_limit_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn RateLimitNameDestination>> = BTreeMap::new();
    sinks.insert(
        "flag-rollout-validation",
        Box::new(FlagRolloutValidationSink {
            client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "environment-calculation",
        Box::new(EnvironmentCalculationSink {
            client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient::new(
                channel,
            ),
        }),
    );
    match sinks.get("flag-rollout-validation") {
        Some(sink) => sink.push(record).await,
        None => anyhow::bail!("no sink registered for flag-rollout-validation"),
    }
}

/// What `relay_rate_limit_name` keeps of each LookupRateLimit response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitNameRecord {
    pub name: String,
    pub message: String,
    pub rate_limit_id: String,
}

/// Where `relay_rate_limit_name` hands each record.
trait RateLimitNameDestination {
    fn push(&self, record: RateLimitNameRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FlagRolloutValidationService`.
struct FlagRolloutValidationSink {
    client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient<Channel>,
}

impl RateLimitNameDestination for FlagRolloutValidationSink {
    fn push(&self, record: RateLimitNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_flag_rollout_decision(featureflags::ExplainFlagRolloutDecisionRequest {
                    flag_rollout: Some(featureflags::FlagRollout {
                        title: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.EnvironmentCalculationService`.
struct EnvironmentCalculationSink {
    client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient<Channel>,
}

impl RateLimitNameDestination for EnvironmentCalculationSink {
    fn push(&self, record: RateLimitNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_environment(configuration::SimulateEnvironmentRequest {
                    environment: Some(configuration::Environment {
                        environment_id: record.name,
                        name: record.message,
                        code: record.rate_limit_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
