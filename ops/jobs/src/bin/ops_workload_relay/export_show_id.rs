//! `export_show_id` reads `acme.shows.v1.ShowService/GetShow` and hands what it keeps to the sink of its
//! kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::onboarding::v1 as onboarding;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = shows::show_service_client::ShowServiceClient::new(channel.clone());
    let found = source
        .get_show(shows::GetShowRequest {
            context: Some(context.clone()),
            show_id: "primary-export-show-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let show = found.show.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = ShowIdRecord {
        show_id: show.show_id,
        message: result.message,
        correlation_id: result.correlation_id,
    };
    dispatch_show_id("onboarding-plan-calculation", record, channel, &context).await
}

/// What `export_show_id` keeps of each GetShow response.
#[derive(Clone, Debug, Default)]
pub struct ShowIdRecord {
    pub show_id: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `export_show_id` hands each record.
trait ShowIdWriter {
    fn deliver(&self, record: ShowIdRecord) -> Delivery;
}

/// Sends records to `acme.onboarding.v1.OnboardingPlanCalculationService`.
struct OnboardingPlanCalculationSink {
    client: onboarding::onboarding_plan_calculation_service_client::OnboardingPlanCalculationServiceClient<Channel>,
}

impl ShowIdWriter for OnboardingPlanCalculationSink {
    fn deliver(&self, record: ShowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_onboarding_plan(onboarding::CalculateOnboardingPlanRequest {
                    onboarding_plan: Some(onboarding::OnboardingPlan {
                        number: record.message,
                        ..Default::default()
                    }),
                    details: Some(onboarding::calculate_onboarding_plan_request::Details {
                        currency_code: record.show_id,
                        ..Default::default()
                    }),
                    onboarding_plan_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FeatureFlagQueryService`.
struct FeatureFlagQuerySink {
    client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient<Channel>,
}

impl ShowIdWriter for FeatureFlagQuerySink {
    fn deliver(&self, record: ShowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_feature_flags(featureflags::SummarizeFeatureFlagsRequest {
                    details: Some(featureflags::summarize_feature_flags_request::Details {
                        cron: record.show_id,
                        format: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.RateLimitSyncService`.
struct RateLimitSyncSink {
    client: ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient<Channel>,
}

impl ShowIdWriter for RateLimitSyncSink {
    fn deliver(&self, record: ShowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .push_rate_limit(ratelimiting::PushRateLimitRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        name: record.message,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    details: Some(ratelimiting::push_rate_limit_request::Details {
                        cursor: record.show_id,
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
struct ShowIdLogOnly;

impl ShowIdWriter for ShowIdLogOnly {
    fn deliver(&self, record: ShowIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.show_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_show_id(
    kind: &str,
    record: ShowIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let onboarding_plan_calculation = OnboardingPlanCalculationSink {
        client: onboarding::onboarding_plan_calculation_service_client::OnboardingPlanCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let feature_flag_query = FeatureFlagQuerySink {
        client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(channel.clone()),
    };
    let rate_limit_sync = RateLimitSyncSink {
        client: ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient::new(channel),
    };
    let write: &dyn Fn(ShowIdRecord) -> Delivery = match kind {
        "onboarding-plan-calculation" => &|record| onboarding_plan_calculation.deliver(record),
        "feature-flag-query" => &|record| feature_flag_query.deliver(record),
        "rate-limit-sync" => &|record| rate_limit_sync.deliver(record),
        _ => &|record| ShowIdLogOnly.deliver(record),
    };
    write(record).await
}
