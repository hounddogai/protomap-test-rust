//! `forward_role_binding_id` reads `acme.authorization.v1.RoleBindingAdminService/InspectRoleBinding` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient::new(channel.clone());
    let reply = source
        .inspect_role_binding(authorization::InspectRoleBindingRequest {
            context: Some(context.clone()),
            role_binding_id: "default-forward-role-binding-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = RoleBindingIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn RoleBindingIdTarget>> = BTreeMap::new();
    sinks.insert(
        "key-ring-workflow",
        Box::new(KeyRingWorkflowSink {
            client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "rate-limit-validation",
        Box::new(RateLimitValidationSink {
            client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "role-binding-query",
        Box::new(RoleBindingQuerySink {
            client: authorization::role_binding_query_service_client::RoleBindingQueryServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("rate-limit-validation") else {
        anyhow::bail!("no sink registered for rate-limit-validation");
    };
    sink.apply(record).await
}

/// What `forward_role_binding_id` keeps of each InspectRoleBinding response.
#[derive(Clone, Debug, Default)]
pub struct RoleBindingIdRecord {
    pub correlation_id: String,
    pub message: String,
}

/// Where `forward_role_binding_id` hands each record.
trait RoleBindingIdTarget {
    fn apply(&self, record: RoleBindingIdRecord) -> Delivery;
}

/// Writes records to `acme.keymanagement.v1.KeyRingWorkflowService`.
struct KeyRingWorkflowSink {
    client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient<Channel>,
}

impl RoleBindingIdTarget for KeyRingWorkflowSink {
    fn apply(&self, record: RoleBindingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .cancel_key_ring_workflow(keymanagement::CancelKeyRingWorkflowRequest {
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.RateLimitValidationService`.
struct RateLimitValidationSink {
    client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient<Channel>,
}

impl RoleBindingIdTarget for RateLimitValidationSink {
    fn apply(&self, record: RoleBindingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_rate_limits(ratelimiting::LintRateLimitsRequest {
                    reason: record.correlation_id,
                    rate_limit: Some(ratelimiting::RateLimit {
                        description: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.authorization.v1.RoleBindingQueryService`.
struct RoleBindingQuerySink {
    client: authorization::role_binding_query_service_client::RoleBindingQueryServiceClient<Channel>,
}

impl RoleBindingIdTarget for RoleBindingQuerySink {
    fn apply(&self, record: RoleBindingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_role_bindings(authorization::BrowseRoleBindingsRequest {
                    order_by: record.correlation_id,
                    filter: Some(authorization::browse_role_bindings_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
