//! `replay_flag_evaluation_id` reads `acme.featureflags.v1.FlagEvaluationAdminService/PurgeFlagEvaluation`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::catalog::v1 as catalog;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(channel.clone());
    let found = source
        .purge_flag_evaluation(featureflags::PurgeFlagEvaluationRequest {
            context: Some(context.clone()),
            reason: "main-replay-flag-evaluation-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let record = FlagEvaluationIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn FlagEvaluationIdTarget>> = BTreeMap::new();
    sinks.insert(
        "rate-limit",
        Box::new(RateLimitSink {
            client: ratelimiting::rate_limit_service_client::RateLimitServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "care-plan-query",
        Box::new(CarePlanQuerySink {
            client: careplans::care_plan_query_service_client::CarePlanQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "product-calculation",
        Box::new(ProductCalculationSink {
            client: catalog::product_calculation_service_client::ProductCalculationServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "user-validation",
        Box::new(UserValidationSink {
            client: identity::user_validation_service_client::UserValidationServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("product-calculation") else {
        anyhow::bail!("no sink registered for product-calculation");
    };
    sink.deliver(record).await
}

/// What `replay_flag_evaluation_id` keeps of each PurgeFlagEvaluation response.
#[derive(Clone, Debug, Default)]
pub struct FlagEvaluationIdRecord {
    pub correlation_id: String,
    pub status: String,
}

/// Where `replay_flag_evaluation_id` hands each record.
trait FlagEvaluationIdTarget {
    fn deliver(&self, record: FlagEvaluationIdRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.RateLimitService`.
struct RateLimitSink {
    client: ratelimiting::rate_limit_service_client::RateLimitServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationIdTarget for RateLimitSink {
    fn deliver(&self, record: FlagEvaluationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::SearchRateLimitsRequest {
                    filter: Some(ratelimiting::search_rate_limits_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_rate_limits(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.careplans.v1.CarePlanQueryService`.
struct CarePlanQuerySink {
    client: careplans::care_plan_query_service_client::CarePlanQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationIdTarget for CarePlanQuerySink {
    fn deliver(&self, record: FlagEvaluationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(careplans::LookupCarePlanRequest {
                    care_plan_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_care_plan(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.catalog.v1.ProductCalculationService`.
struct ProductCalculationSink {
    client: catalog::product_calculation_service_client::ProductCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationIdTarget for ProductCalculationSink {
    fn deliver(&self, record: FlagEvaluationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(catalog::CalculateProductRequest {
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.calculate_product(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.identity.v1.UserValidationService`.
struct UserValidationSink {
    client: identity::user_validation_service_client::UserValidationServiceClient<Channel>,
}

impl FlagEvaluationIdTarget for UserValidationSink {
    fn deliver(&self, record: FlagEvaluationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_user_rules(identity::GetUserRulesRequest {
                    user_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
