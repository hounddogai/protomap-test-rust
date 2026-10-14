//! `hand_off_broadcasts_message` reads `acme.broadcasts.v1.BroadcastQueryService/CountBroadcasts` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient::new(channel.clone());
    let fetched = source
        .count_broadcasts(broadcasts::CountBroadcastsRequest {
            context: Some(context.clone()),
            order_by: "current-hand-off-broadcasts-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = OffBroadcastsMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffBroadcastsMessageDestination>> = BTreeMap::new();
    sinks.insert(
        "access-grant-calculation",
        Box::new(AccessGrantCalculationSink {
            client: secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "quota-bucket-lifecycle",
        Box::new(QuotaBucketLifecycleSink {
            client: ratelimiting::quota_bucket_lifecycle_service_client::QuotaBucketLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "environment-approval",
        Box::new(EnvironmentApprovalSink {
            client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "throttle-calculation",
        Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("environment-approval") else {
        anyhow::bail!("no sink registered for environment-approval");
    };
    sink.forward(record).await
}

/// What `hand_off_broadcasts_message` keeps of each CountBroadcasts response.
#[derive(Clone, Debug, Default)]
pub struct OffBroadcastsMessageRecord {
    pub message: String,
    pub correlation_id: String,
}

/// Where `hand_off_broadcasts_message` hands each record.
trait OffBroadcastsMessageDestination {
    fn forward(&self, record: OffBroadcastsMessageRecord) -> Delivery;
}

/// Hands records to `acme.secrets.v1.AccessGrantCalculationService`.
struct AccessGrantCalculationSink {
    client: secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient<Channel>,
}

impl OffBroadcastsMessageDestination for AccessGrantCalculationSink {
    fn forward(&self, record: OffBroadcastsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_access_grant(secrets::EstimateAccessGrantRequest {
                    access_grant_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketLifecycleService`.
struct QuotaBucketLifecycleSink {
    client: ratelimiting::quota_bucket_lifecycle_service_client::QuotaBucketLifecycleServiceClient<Channel>,
}

impl OffBroadcastsMessageDestination for QuotaBucketLifecycleSink {
    fn forward(&self, record: OffBroadcastsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reinstate_quota_bucket(ratelimiting::ReinstateQuotaBucketRequest {
                    quota_bucket_id: record.message,
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.EnvironmentApprovalService`.
struct EnvironmentApprovalSink {
    client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffBroadcastsMessageDestination for EnvironmentApprovalSink {
    fn forward(&self, record: OffBroadcastsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::SubmitEnvironmentRequest {
                    reason: record.message,
                    environment: Some(configuration::Environment {
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.submit_environment(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
}

impl OffBroadcastsMessageDestination for ThrottleCalculationSink {
    fn forward(&self, record: OffBroadcastsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_throttle(ratelimiting::EstimateThrottleRequest {
                    throttle: Some(ratelimiting::Throttle {
                        throttle_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
