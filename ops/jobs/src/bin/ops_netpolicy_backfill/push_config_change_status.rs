//! `push_config_change_status` reads
//! `acme.configuration.v1.ConfigChangeLifecycleService/ActivateConfigChange` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient::new(channel.clone());
    let fetched = source
        .activate_config_change(configuration::ActivateConfigChangeRequest {
            context: Some(context.clone()),
            config_change_id: "live-push-config-change-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let config_change = fetched.config_change.unwrap_or_default();
    let record = ConfigChangeStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        config_change_id: config_change.config_change_id,
        source: config_change.source,
        kind: acme_kit::text::normalize(&config_change.kind),
    };
    let mut sinks: BTreeMap<String, Box<dyn ConfigChangeStatusDestination>> = BTreeMap::new();
    sinks.insert(
        "access-policy-admin".to_owned(),
        Box::new(AccessPolicyAdminSink {
            client: authorization::access_policy_admin_service_client::AccessPolicyAdminServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "config-set-approval".to_owned(),
        Box::new(ConfigSetApprovalSink {
            client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient::new(channel),
        }),
    );
    let target = std::env::var("PUSH_CONFIG_CHANGE_STATUS_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.forward(record).await,
        None => Ok(()),
    }
}

/// What `push_config_change_status` keeps of each ActivateConfigChange response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeStatusRecord {
    pub status: String,
    pub config_change_id: String,
    pub source: String,
    pub kind: String,
}

/// Where `push_config_change_status` hands each record.
trait ConfigChangeStatusDestination {
    fn forward(&self, record: ConfigChangeStatusRecord) -> Delivery;
}

/// Writes records to `acme.authorization.v1.AccessPolicyAdminService`.
struct AccessPolicyAdminSink {
    client: authorization::access_policy_admin_service_client::AccessPolicyAdminServiceClient<Channel>,
}

impl ConfigChangeStatusDestination for AccessPolicyAdminSink {
    fn forward(&self, record: ConfigChangeStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lock_access_policy(authorization::LockAccessPolicyRequest {
                    reason: record.status,
                    access_policy_id: record.config_change_id,
                    details: Some(authorization::lock_access_policy_request::Details {
                        reason_code: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.ConfigSetApprovalService`.
struct ConfigSetApprovalSink {
    client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient<Channel>,
}

impl ConfigChangeStatusDestination for ConfigSetApprovalSink {
    fn forward(&self, record: ConfigChangeStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .submit_config_set(configuration::SubmitConfigSetRequest {
                    reason: record.config_change_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
