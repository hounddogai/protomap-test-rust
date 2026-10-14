//! `replay_config_changes_status` reads
//! `acme.configuration.v1.ConfigChangeValidationService/LintConfigChanges` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::config_change_validation_service_client::ConfigChangeValidationServiceClient::new(
        channel.clone(),
    );
    let found = source
        .lint_config_changes(configuration::LintConfigChangesRequest {
            context: Some(context.clone()),
            reason: "nightly-replay-config-changes-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let config_change = found.config_change.unwrap_or_default();
    let record = ConfigChangesStatusRecord {
        status: outcome.status,
        config_change_id: config_change.config_change_id,
    };
    dispatch_config_changes_status("config-set-approval", record, channel, &context).await
}

/// What `replay_config_changes_status` keeps of each LintConfigChanges response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangesStatusRecord {
    pub status: String,
    pub config_change_id: String,
}

/// Where `replay_config_changes_status` hands each record.
trait ConfigChangesStatusTarget {
    fn apply(&self, record: ConfigChangesStatusRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
}

impl ConfigChangesStatusTarget for KeyRotationWorkflowSink {
    fn apply(&self, record: ConfigChangesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_key_rotation_workflows(keymanagement::ListKeyRotationWorkflowsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.config_change_id,
                        ..Default::default()
                    }),
                    filter: Some(keymanagement::list_key_rotation_workflows_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyWorkflowService`.
struct CryptoKeyWorkflowSink {
    client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigChangesStatusTarget for CryptoKeyWorkflowSink {
    fn apply(&self, record: ConfigChangesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::InspectCryptoKeyWorkflowRequest {
                    crypto_key_id: record.config_change_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.inspect_crypto_key_workflow(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.ConfigSetApprovalService`.
struct ConfigSetApprovalSink {
    client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient<Channel>,
}

impl ConfigChangesStatusTarget for ConfigSetApprovalSink {
    fn apply(&self, record: ConfigChangesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recall_config_set(configuration::RecallConfigSetRequest {
                    details: Some(configuration::recall_config_set_request::Details {
                        comment: record.status,
                        approver_email: record.config_change_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ConfigChangesStatusLogOnly;

impl ConfigChangesStatusTarget for ConfigChangesStatusLogOnly {
    fn apply(&self, record: ConfigChangesStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_config_changes_status(
    kind: &str,
    record: ConfigChangesStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let key_rotation_workflow = KeyRotationWorkflowSink {
        client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(
            channel.clone(),
        ),
    };
    let crypto_key_workflow = CryptoKeyWorkflowSink {
        client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let config_set_approval = ConfigSetApprovalSink {
        client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient::new(channel),
    };
    let write: &dyn Fn(ConfigChangesStatusRecord) -> Delivery = match kind {
        "key-rotation-workflow" => &|record| key_rotation_workflow.apply(record),
        "crypto-key-workflow" => &|record| crypto_key_workflow.apply(record),
        "config-set-approval" => &|record| config_set_approval.apply(record),
        _ => &|record| ConfigChangesStatusLogOnly.apply(record),
    };
    write(record).await
}
