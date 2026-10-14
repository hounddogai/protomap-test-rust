//! `route_anomalies_message` reads `acme.anomalies.v1.AnomalyQueryService/BrowseAnomalies` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = anomalies::anomaly_query_service_client::AnomalyQueryServiceClient::new(channel.clone());
    let reply = source
        .browse_anomalies(anomalies::BrowseAnomaliesRequest {
            context: Some(context.clone()),
            order_by: "primary-route-anomalies-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = AnomaliesMessageRecord {
        message: result.message,
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        status: result.status,
        correlation_id: result.correlation_id,
    };
    act_on_anomalies_message("rate-limit-validation", record, channel, context).await
}

/// What `route_anomalies_message` keeps of each BrowseAnomalies response.
#[derive(Clone, Debug, Default)]
pub struct AnomaliesMessageRecord {
    pub message: String,
    pub next_page_token: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `route_anomalies_message` hands each record.
trait AnomaliesMessageTarget {
    fn write(&self, record: AnomaliesMessageRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigSetLifecycleService`.
struct ConfigSetLifecycleSink {
    client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient<Channel>,
}

impl AnomaliesMessageTarget for ConfigSetLifecycleSink {
    fn write(&self, record: AnomaliesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .expire_config_set(configuration::ExpireConfigSetRequest {
                    reason: record.status,
                    details: Some(configuration::expire_config_set_request::Details {
                        reason_code: record.message,
                        ..Default::default()
                    }),
                    config_set_id: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.edgegateway.v1.RouteBindingIntegrationService`.
struct RouteBindingIntegrationSink {
    client: edgegateway::route_binding_integration_service_client::RouteBindingIntegrationServiceClient<Channel>,
}

impl AnomaliesMessageTarget for RouteBindingIntegrationSink {
    fn write(&self, record: AnomaliesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_route_binding_sources(edgegateway::ListRouteBindingSourcesRequest {
                    order_by: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.RateLimitValidationService`.
struct RateLimitValidationSink {
    client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient<Channel>,
}

impl AnomaliesMessageTarget for RateLimitValidationSink {
    fn write(&self, record: AnomaliesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .preview_rate_limit(ratelimiting::PreviewRateLimitRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        description: record.status,
                        ..Default::default()
                    }),
                    rate_limit_id: record.correlation_id,
                    reason: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRotationLifecycleService`.
struct KeyRotationLifecycleSink {
    client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl AnomaliesMessageTarget for KeyRotationLifecycleSink {
    fn write(&self, record: AnomaliesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ScheduleKeyRotationExpiryRequest {
                    reason: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.schedule_key_rotation_expiry(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_anomalies_message(
    kind: &str,
    record: AnomaliesMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(AnomaliesMessageRecord) -> Delivery>> = BTreeMap::new();
    let config_set_lifecycle_channel = channel.clone();
    actions.insert(
        "config-set-lifecycle",
        Box::new(move |record| {
            ConfigSetLifecycleSink {
                client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(
                    config_set_lifecycle_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let route_binding_integration_channel = channel.clone();
    actions.insert(
        "route-binding-integration",
        Box::new(move |record| {
            RouteBindingIntegrationSink {
                client:
                    edgegateway::route_binding_integration_service_client::RouteBindingIntegrationServiceClient::new(
                        route_binding_integration_channel.clone(),
                    ),
            }
            .write(record)
        }),
    );
    let rate_limit_validation_channel = channel.clone();
    actions.insert(
        "rate-limit-validation",
        Box::new(move |record| {
            RateLimitValidationSink {
                client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(
                    rate_limit_validation_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let key_rotation_lifecycle_channel = channel.clone();
    let key_rotation_lifecycle_context = context.clone();
    actions.insert(
        "key-rotation-lifecycle",
        Box::new(move |record| {
            KeyRotationLifecycleSink {
                client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient::new(
                    key_rotation_lifecycle_channel.clone(),
                ),
                context: key_rotation_lifecycle_context.clone(),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
