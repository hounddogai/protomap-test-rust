//! `copy_config_sets_message` reads `acme.configuration.v1.ConfigSetQueryService/CountConfigSets` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::config_set_query_service_client::ConfigSetQueryServiceClient::new(channel.clone());
    let reply = source
        .count_config_sets(configuration::CountConfigSetsRequest {
            context: Some(context.clone()),
            order_by: "default-copy-config-sets-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = ConfigSetsMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
    };
    act_on_config_sets_message("key-ring-lifecycle", record, channel, context).await
}

/// What `copy_config_sets_message` keeps of each CountConfigSets response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetsMessageRecord {
    pub message: String,
    pub correlation_id: String,
}

/// Where `copy_config_sets_message` hands each record.
trait ConfigSetsMessageSink {
    fn write(&self, record: ConfigSetsMessageRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.KeyRingLifecycleService`.
struct KeyRingLifecycleSink {
    client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient<Channel>,
}

impl ConfigSetsMessageSink for KeyRingLifecycleSink {
    fn write(&self, record: ConfigSetsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_key_ring(keymanagement::RenewKeyRingRequest {
                    key_ring_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.authorization.v1.RoleBindingValidationService`.
struct RoleBindingValidationSink {
    client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetsMessageSink for RoleBindingValidationSink {
    fn write(&self, record: ConfigSetsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::PreviewRoleBindingRequest {
                    role_binding: Some(authorization::RoleBinding {
                        notes: record.correlation_id,
                        ..Default::default()
                    }),
                    role_binding_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.preview_role_binding(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.edgegateway.v1.RouteBindingService`.
struct RouteBindingSink {
    client: edgegateway::route_binding_service_client::RouteBindingServiceClient<Channel>,
}

impl ConfigSetsMessageSink for RouteBindingSink {
    fn write(&self, record: ConfigSetsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_route_binding(edgegateway::GetRouteBindingRequest {
                    route_binding_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_config_sets_message(
    kind: &str,
    record: ConfigSetsMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(ConfigSetsMessageRecord) -> Delivery>> = BTreeMap::new();
    let key_ring_lifecycle_channel = channel.clone();
    actions.insert(
        "key-ring-lifecycle",
        Box::new(move |record| {
            KeyRingLifecycleSink {
                client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient::new(
                    key_ring_lifecycle_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let role_binding_validation_channel = channel.clone();
    let role_binding_validation_context = context.clone();
    actions.insert(
        "role-binding-validation",
        Box::new(move |record| {
            RoleBindingValidationSink {
                client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient::new(
                    role_binding_validation_channel.clone(),
                ),
                context: role_binding_validation_context.clone(),
            }
            .write(record)
        }),
    );
    let route_binding_channel = channel.clone();
    actions.insert(
        "route-binding",
        Box::new(move |record| {
            RouteBindingSink {
                client: edgegateway::route_binding_service_client::RouteBindingServiceClient::new(
                    route_binding_channel.clone(),
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
