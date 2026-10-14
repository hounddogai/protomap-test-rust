//! `settle_config_set_workflow_id` reads
//! `acme.configuration.v1.ConfigSetWorkflowService/StartConfigSetWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient::new(channel.clone());
    let found = source
        .start_config_set_workflow(configuration::StartConfigSetWorkflowRequest {
            context: Some(context.clone()),
            reason: "main-settle-config-set-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = ConfigSetWorkflowIdRecord {
        reference_id: outcome.reference_id,
        status: result.status,
    };
    act_on_config_set_workflow_id("config-set-query", record, channel, context).await
}

/// What `settle_config_set_workflow_id` keeps of each StartConfigSetWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetWorkflowIdRecord {
    pub reference_id: String,
    pub status: String,
}

/// Where `settle_config_set_workflow_id` hands each record.
trait ConfigSetWorkflowIdSink {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.ConfigSetQueryService`.
struct ConfigSetQuerySink {
    client: configuration::config_set_query_service_client::ConfigSetQueryServiceClient<Channel>,
}

impl ConfigSetWorkflowIdSink for ConfigSetQuerySink {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_config_set(configuration::LookupConfigSetRequest {
                    config_set_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.secrets.v1.SecretVersionWorkflowService`.
struct SecretVersionWorkflowSink {
    client: secrets::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient<Channel>,
}

impl ConfigSetWorkflowIdSink for SecretVersionWorkflowSink {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resume_secret_version_workflow(secrets::ResumeSecretVersionWorkflowRequest {
                    secret_version: Some(secrets::SecretVersion {
                        kind: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.authorization.v1.AccessPolicyAdminService`.
struct AccessPolicyAdminSink {
    client: authorization::access_policy_admin_service_client::AccessPolicyAdminServiceClient<Channel>,
}

impl ConfigSetWorkflowIdSink for AccessPolicyAdminSink {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .transfer_access_policy(authorization::TransferAccessPolicyRequest {
                    details: Some(authorization::transfer_access_policy_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    access_policy_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_config_set_workflow_id(
    kind: &str,
    record: ConfigSetWorkflowIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(ConfigSetWorkflowIdRecord) -> Delivery>> = BTreeMap::new();
    let config_set_query_channel = channel.clone();
    actions.insert(
        "config-set-query",
        Box::new(move |record| {
            ConfigSetQuerySink {
                client: configuration::config_set_query_service_client::ConfigSetQueryServiceClient::new(
                    config_set_query_channel.clone(),
                ),
            }
            .push(record)
        }),
    );
    let secret_version_workflow_channel = channel.clone();
    actions.insert(
        "secret-version-workflow",
        Box::new(move |record| {
            SecretVersionWorkflowSink {
                client: secrets::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient::new(
                    secret_version_workflow_channel.clone(),
                ),
            }
            .push(record)
        }),
    );
    let access_policy_admin_channel = channel.clone();
    actions.insert(
        "access-policy-admin",
        Box::new(move |record| {
            AccessPolicyAdminSink {
                client: authorization::access_policy_admin_service_client::AccessPolicyAdminServiceClient::new(
                    access_policy_admin_channel.clone(),
                ),
            }
            .push(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
