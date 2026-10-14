//! `forward_vehicle_status` reads `acme.fleet.v1.VehicleService/DeleteVehicle` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fleet::vehicle_service_client::VehicleServiceClient::new(channel.clone());
    let loaded = source
        .delete_vehicle(fleet::DeleteVehicleRequest {
            context: Some(context.clone()),
            vehicle_id: "live-forward-vehicle-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = VehicleStatusRecord {
        status: result.status,
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn VehicleStatusDestination>> = BTreeMap::new();
    sinks.insert(
        "throttle-reporting",
        Box::new(ThrottleReportingSink {
            client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "quota-bucket-workflow",
        Box::new(QuotaBucketWorkflowSink {
            client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "environment-validation",
        Box::new(EnvironmentValidationSink {
            client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "rate-limit-sync",
        Box::new(RateLimitSyncSink {
            client: ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient::new(channel),
        }),
    );
    match sinks.get("environment-validation") {
        Some(sink) => sink.send(record).await,
        None => anyhow::bail!("no sink registered for environment-validation"),
    }
}

/// What `forward_vehicle_status` keeps of each DeleteVehicle response.
#[derive(Clone, Debug, Default)]
pub struct VehicleStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `forward_vehicle_status` hands each record.
trait VehicleStatusDestination {
    fn send(&self, record: VehicleStatusRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.ThrottleReportingService`.
struct ThrottleReportingSink {
    client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient<Channel>,
}

impl VehicleStatusDestination for ThrottleReportingSink {
    fn send(&self, record: VehicleStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .share_throttle_report(ratelimiting::ShareThrottleReportRequest {
                    details: Some(ratelimiting::share_throttle_report_request::Details {
                        template_id: record.message,
                        channel: record.correlation_id,
                        ..Default::default()
                    }),
                    throttle_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketWorkflowService`.
struct QuotaBucketWorkflowSink {
    client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl VehicleStatusDestination for QuotaBucketWorkflowSink {
    fn send(&self, record: VehicleStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ListQuotaBucketWorkflowsRequest {
                    filter: Some(ratelimiting::list_quota_bucket_workflows_request::Filter {
                        query: record.message,
                        region: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_quota_bucket_workflows(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.EnvironmentValidationService`.
struct EnvironmentValidationSink {
    client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient<Channel>,
}

impl VehicleStatusDestination for EnvironmentValidationSink {
    fn send(&self, record: VehicleStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_environment(configuration::NormalizeEnvironmentRequest {
                    reason: record.message,
                    environment: Some(configuration::Environment {
                        environment_id: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(configuration::normalize_environment_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.RateLimitSyncService`.
struct RateLimitSyncSink {
    client: ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient<Channel>,
}

impl VehicleStatusDestination for RateLimitSyncSink {
    fn send(&self, record: VehicleStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .sync_rate_limits(ratelimiting::SyncRateLimitsRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        rate_limit_id: record.message,
                        ..Default::default()
                    }),
                    details: Some(ratelimiting::sync_rate_limits_request::Details {
                        resolution: record.correlation_id,
                        peer_region: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
