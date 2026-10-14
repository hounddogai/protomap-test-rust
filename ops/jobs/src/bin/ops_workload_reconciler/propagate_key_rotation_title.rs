//! `propagate_key_rotation_title` reads
//! `acme.keymanagement.v1.KeyRotationLifecycleService/ActivateKeyRotation` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient::new(channel.clone());
    let reply = source
        .activate_key_rotation(keymanagement::ActivateKeyRotationRequest {
            context: Some(context.clone()),
            key_rotation_id: "current-propagate-key-rotation-title".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let key_rotation = reply.key_rotation.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = KeyRotationTitleRecord {
        title: key_rotation.title,
        correlation_id: result.correlation_id,
        reference_id: outcome.reference_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn KeyRotationTitleWriter>> = BTreeMap::new();
    sinks.insert(
        "route-binding",
        Box::new(RouteBindingSink {
            client: edgegateway::route_binding_service_client::RouteBindingServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "access-policy-validation",
        Box::new(AccessPolicyValidationSink {
            client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "notification",
        Box::new(NotificationSink {
            client: notifications::notification_service_client::NotificationServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "rate-limit-validation",
        Box::new(RateLimitValidationSink {
            client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    let Some(sink) = sinks.get("notification") else {
        anyhow::bail!("no sink registered for notification");
    };
    sink.write(record).await
}

/// What `propagate_key_rotation_title` keeps of each ActivateKeyRotation response.
#[derive(Clone, Debug, Default)]
pub struct KeyRotationTitleRecord {
    pub title: String,
    pub correlation_id: String,
    pub reference_id: String,
}

/// Where `propagate_key_rotation_title` hands each record.
trait KeyRotationTitleWriter {
    fn write(&self, record: KeyRotationTitleRecord) -> Delivery;
}

/// Sends records to `acme.edgegateway.v1.RouteBindingService`.
struct RouteBindingSink {
    client: edgegateway::route_binding_service_client::RouteBindingServiceClient<Channel>,
}

impl KeyRotationTitleWriter for RouteBindingSink {
    fn write(&self, record: KeyRotationTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_route_binding(edgegateway::UpdateRouteBindingRequest {
                    route_binding: Some(edgegateway::RouteBinding {
                        route_binding_id: record.title,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.authorization.v1.AccessPolicyValidationService`.
struct AccessPolicyValidationSink {
    client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient<Channel>,
}

impl KeyRotationTitleWriter for AccessPolicyValidationSink {
    fn write(&self, record: KeyRotationTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_access_policy_eligibility(authorization::CheckAccessPolicyEligibilityRequest {
                    reason: record.title,
                    access_policy: Some(authorization::AccessPolicy {
                        description: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.notifications.v1.NotificationService`.
struct NotificationSink {
    client: notifications::notification_service_client::NotificationServiceClient<Channel>,
}

impl KeyRotationTitleWriter for NotificationSink {
    fn write(&self, record: KeyRotationTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_notification(notifications::CreateNotificationRequest {
                    notification: Some(notifications::Notification {
                        notification_id: record.reference_id,
                        ..Default::default()
                    }),
                    request_id: record.title,
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
    context: common::RequestContext,
}

impl KeyRotationTitleWriter for RateLimitValidationSink {
    fn write(&self, record: KeyRotationTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::LintRateLimitsRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    rate_limit_id: record.title,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_rate_limits(request).await?;
            Ok(())
        })
    }
}
