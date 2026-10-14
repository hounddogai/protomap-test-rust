//! `sync_rate_limit_name` reads `acme.ratelimiting.v1.RateLimitQueryService/PeekRateLimit` and hands what it
//! keeps to the sink of its kind.

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
    let loaded = source
        .peek_rate_limit(ratelimiting::PeekRateLimitRequest {
            context: Some(context.clone()),
            rate_limit_id: "current-sync-rate-limit-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let rate_limit = loaded.rate_limit.unwrap_or_default();
    let record = RateLimitNameRecord {
        name: rate_limit.name,
        rate_limit_id: rate_limit.rate_limit_id,
    };
    act_on_rate_limit_name("feature-flag-query", record, channel, context).await
}

/// What `sync_rate_limit_name` keeps of each PeekRateLimit response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitNameRecord {
    pub name: String,
    pub rate_limit_id: String,
}

/// Where `sync_rate_limit_name` hands each record.
trait RateLimitNameTarget {
    fn write(&self, record: RateLimitNameRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.EnvironmentWorkflowService`.
struct EnvironmentWorkflowSink {
    client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient<Channel>,
}

impl RateLimitNameTarget for EnvironmentWorkflowSink {
    fn write(&self, record: RateLimitNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resume_environment_workflow(configuration::ResumeEnvironmentWorkflowRequest {
                    environment: Some(configuration::Environment {
                        environment_id: record.name,
                        ..Default::default()
                    }),
                    details: Some(configuration::resume_environment_workflow_request::Details {
                        initiated_by: record.rate_limit_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FeatureFlagQueryService`.
struct FeatureFlagQuerySink {
    client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient<Channel>,
}

impl RateLimitNameTarget for FeatureFlagQuerySink {
    fn write(&self, record: RateLimitNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_feature_flags(featureflags::CountFeatureFlagsRequest {
                    filter: Some(featureflags::count_feature_flags_request::Filter {
                        owner_id: record.rate_limit_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.ThrottleValidationService`.
struct ThrottleValidationSink {
    client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient<Channel>,
}

impl RateLimitNameTarget for ThrottleValidationSink {
    fn write(&self, record: RateLimitNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_throttle(ratelimiting::ScoreThrottleRequest {
                    details: Some(ratelimiting::score_throttle_request::Details {
                        currency_code: record.name,
                        ..Default::default()
                    }),
                    throttle: Some(ratelimiting::Throttle {
                        throttle_id: record.rate_limit_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_rate_limit_name(
    kind: &str,
    record: RateLimitNameRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(RateLimitNameRecord) -> Delivery>> = BTreeMap::new();
    let environment_workflow_channel = channel.clone();
    actions.insert(
        "environment-workflow",
        Box::new(move |record| {
            EnvironmentWorkflowSink {
                client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(
                    environment_workflow_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let feature_flag_query_channel = channel.clone();
    actions.insert(
        "feature-flag-query",
        Box::new(move |record| {
            FeatureFlagQuerySink {
                client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(
                    feature_flag_query_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let throttle_validation_channel = channel.clone();
    actions.insert(
        "throttle-validation",
        Box::new(move |record| {
            ThrottleValidationSink {
                client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient::new(
                    throttle_validation_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
