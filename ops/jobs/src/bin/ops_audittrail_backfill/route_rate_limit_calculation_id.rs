//! `route_rate_limit_calculation_id` reads
//! `acme.ratelimiting.v1.RateLimitCalculationService/ExplainRateLimitCalculation` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::rate_limit_calculation_service_client::RateLimitCalculationServiceClient::new(channel.clone());
    let reply = source
        .explain_rate_limit_calculation(ratelimiting::ExplainRateLimitCalculationRequest {
            context: Some(context.clone()),
            rate_limit_id: "primary-route-rate-limit-calculation-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let rate_limit = reply.rate_limit.unwrap_or_default();
    let record = RateLimitCalculationIdRecord {
        reference_id: outcome.reference_id,
        name: rate_limit.name,
        message: outcome.message,
        status: acme_kit::text::normalize(&outcome.status),
    };
    dispatch_rate_limit_calculation_id("throttle-lifecycle", record, channel, &context).await
}

/// What `route_rate_limit_calculation_id` keeps of each ExplainRateLimitCalculation response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitCalculationIdRecord {
    pub reference_id: String,
    pub name: String,
    pub message: String,
    pub status: String,
}

/// Where `route_rate_limit_calculation_id` hands each record.
trait RateLimitCalculationIdSink {
    fn apply(&self, record: RateLimitCalculationIdRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.ThrottleLifecycleService`.
struct ThrottleLifecycleSink {
    client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl RateLimitCalculationIdSink for ThrottleLifecycleSink {
    fn apply(&self, record: RateLimitCalculationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ActivateThrottleRequest {
                    reason: record.message,
                    throttle_id: record.status,
                    details: Some(ratelimiting::activate_throttle_request::Details {
                        reason_code: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.activate_throttle(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.ThrottleSyncService`.
struct ThrottleSyncSink {
    client: ratelimiting::throttle_sync_service_client::ThrottleSyncServiceClient<Channel>,
    context: common::RequestContext,
}

impl RateLimitCalculationIdSink for ThrottleSyncSink {
    fn apply(&self, record: RateLimitCalculationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ListThrottleConflictsRequest {
                    filter: Some(ratelimiting::list_throttle_conflicts_request::Filter {
                        owner_id: record.status,
                        region: record.reference_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_throttle_conflicts(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagRolloutHistoryService`.
struct FlagRolloutHistorySink {
    client: featureflags::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient<Channel>,
    context: common::RequestContext,
}

impl RateLimitCalculationIdSink for FlagRolloutHistorySink {
    fn apply(&self, record: RateLimitCalculationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::RecordFlagRolloutVersionRequest {
                    flag_rollout_id: record.message,
                    details: Some(featureflags::record_flag_rollout_version_request::Details {
                        author_email: record.reference_id,
                        ..Default::default()
                    }),
                    reason: record.name,
                    ..Default::default()
                }),
                &context,
            )?;
            client.record_flag_rollout_version(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct RateLimitCalculationIdLogOnly;

impl RateLimitCalculationIdSink for RateLimitCalculationIdLogOnly {
    fn apply(&self, record: RateLimitCalculationIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.reference_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_rate_limit_calculation_id(
    kind: &str,
    record: RateLimitCalculationIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let throttle_lifecycle = ThrottleLifecycleSink {
        client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let throttle_sync = ThrottleSyncSink {
        client: ratelimiting::throttle_sync_service_client::ThrottleSyncServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let flag_rollout_history = FlagRolloutHistorySink {
        client: featureflags::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(RateLimitCalculationIdRecord) -> Delivery = match kind {
        "throttle-lifecycle" => &|record| throttle_lifecycle.apply(record),
        "throttle-sync" => &|record| throttle_sync.apply(record),
        "flag-rollout-history" => &|record| flag_rollout_history.apply(record),
        _ => &|record| RateLimitCalculationIdLogOnly.apply(record),
    };
    write(record).await
}
