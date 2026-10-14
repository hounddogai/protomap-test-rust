//! `sync_secret_version_workflow_kind` reads
//! `acme.secrets.v1.SecretVersionWorkflowService/SignalSecretVersionWorkflow` and hands what it keeps to
//! `acme.keymanagement.v1.KeyRotationValidationService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient::new(channel.clone());
    let loaded = source
        .signal_secret_version_workflow(secrets::SignalSecretVersionWorkflowRequest {
            context: Some(context.clone()),
            secret_version_id: "current-sync-secret-version-workflow-kind".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let secret_version = loaded.secret_version.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = SecretVersionWorkflowKindRecord {
        kind: acme_kit::text::normalize(&secret_version.kind),
        message: result.message,
        status: result.status,
        outcome_status: outcome.status,
    };
    KeyRotationValidationSink {
        client: keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient::new(channel),
        context: context.clone(),
    }
    .deliver(record)
    .await
}

/// What `sync_secret_version_workflow_kind` keeps of each SignalSecretVersionWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct SecretVersionWorkflowKindRecord {
    pub kind: String,
    pub message: String,
    pub status: String,
    pub outcome_status: String,
}

/// Where `sync_secret_version_workflow_kind` hands each record.
trait SecretVersionWorkflowKindTarget {
    fn deliver(&self, record: SecretVersionWorkflowKindRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRotationValidationService`.
struct KeyRotationValidationSink {
    client: keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SecretVersionWorkflowKindTarget for KeyRotationValidationSink {
    fn deliver(&self, record: SecretVersionWorkflowKindRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::GetKeyRotationRulesRequest {
                    key_rotation_id: record.kind,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_key_rotation_rules(request).await?;
            Ok(())
        })
    }
}
