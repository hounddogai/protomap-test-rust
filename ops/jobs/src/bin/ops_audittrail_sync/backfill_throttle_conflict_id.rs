//! `backfill_throttle_conflict_id` reads `acme.ratelimiting.v1.ThrottleSyncService/ResolveThrottleConflict`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::throttle_sync_service_client::ThrottleSyncServiceClient::new(channel.clone());
    let loaded = source
        .resolve_throttle_conflict(ratelimiting::ResolveThrottleConflictRequest {
            context: Some(context.clone()),
            throttle_id: "nightly-backfill-throttle-conflict-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let throttle = loaded.throttle.unwrap_or_default();
    let record = ThrottleConflictIdRecord {
        throttle_id: acme_kit::text::normalize(&throttle.throttle_id),
        correlation_id: throttle.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ThrottleConflictIdSink>> = BTreeMap::new();
    sinks.insert(
        "rate-limit-calculation",
        Box::new(RateLimitCalculationSink {
            client: ratelimiting::rate_limit_calculation_service_client::RateLimitCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "rate-limit-workflow",
        Box::new(RateLimitWorkflowSink {
            client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "api-key-integration",
        Box::new(ApiKeyIntegrationSink {
            client: edgegateway::api_key_integration_service_client::ApiKeyIntegrationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "appointment",
        Box::new(AppointmentSink {
            client: appointments::appointment_service_client::AppointmentServiceClient::new(channel),
        }),
    );
    match sinks.get("rate-limit-calculation") {
        Some(sink) => sink.apply(record).await,
        None => anyhow::bail!("no sink registered for rate-limit-calculation"),
    }
}

/// What `backfill_throttle_conflict_id` keeps of each ResolveThrottleConflict response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleConflictIdRecord {
    pub throttle_id: String,
    pub correlation_id: String,
}

/// Where `backfill_throttle_conflict_id` hands each record.
trait ThrottleConflictIdSink {
    fn apply(&self, record: ThrottleConflictIdRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.RateLimitCalculationService`.
struct RateLimitCalculationSink {
    client: ratelimiting::rate_limit_calculation_service_client::RateLimitCalculationServiceClient<Channel>,
}

impl ThrottleConflictIdSink for RateLimitCalculationSink {
    fn apply(&self, record: ThrottleConflictIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_rate_limit(ratelimiting::SimulateRateLimitRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        name: record.throttle_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleConflictIdSink for RateLimitWorkflowSink {
    fn apply(&self, record: ThrottleConflictIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::StartRateLimitWorkflowRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        description: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.start_rate_limit_workflow(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.edgegateway.v1.ApiKeyIntegrationService`.
struct ApiKeyIntegrationSink {
    client: edgegateway::api_key_integration_service_client::ApiKeyIntegrationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleConflictIdSink for ApiKeyIntegrationSink {
    fn apply(&self, record: ThrottleConflictIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::TestApiKeyConnectionRequest {
                    details: Some(edgegateway::test_api_key_connection_request::Details {
                        message_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.test_api_key_connection(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
}

impl ThrottleConflictIdSink for AppointmentSink {
    fn apply(&self, record: ThrottleConflictIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_appointments(appointments::ListAppointmentsRequest {
                    filter: Some(appointments::list_appointments_request::Filter {
                        region: record.throttle_id,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
