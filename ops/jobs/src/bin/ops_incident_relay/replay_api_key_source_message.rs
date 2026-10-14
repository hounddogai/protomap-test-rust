//! `replay_api_key_source_message` reads
//! `acme.edgegateway.v1.ApiKeyIntegrationService/DisconnectApiKeySource` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::api_key_integration_service_client::ApiKeyIntegrationServiceClient::new(channel.clone());
    let reply = source
        .disconnect_api_key_source(edgegateway::DisconnectApiKeySourceRequest {
            context: Some(context.clone()),
            api_key_id: "primary-replay-api-key-source-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = ApiKeySourceMessageRecord {
        message: result.message,
        status: acme_kit::text::normalize(&result.status),
        correlation_id: result.correlation_id,
    };
    dispatch_api_key_source_message("certificate-validation", record, channel, &context).await
}

/// What `replay_api_key_source_message` keeps of each DisconnectApiKeySource response.
#[derive(Clone, Debug, Default)]
pub struct ApiKeySourceMessageRecord {
    pub message: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `replay_api_key_source_message` hands each record.
trait ApiKeySourceMessageSink {
    fn forward(&self, record: ApiKeySourceMessageRecord) -> Delivery;
}

/// Hands records to `acme.certificates.v1.CertificateValidationService`.
struct CertificateValidationSink {
    client: certificates::certificate_validation_service_client::CertificateValidationServiceClient<Channel>,
}

impl ApiKeySourceMessageSink for CertificateValidationSink {
    fn forward(&self, record: ApiKeySourceMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_certificate(certificates::NormalizeCertificateRequest {
                    certificate_id: record.correlation_id,
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagEvaluationSyncService`.
struct FlagEvaluationSyncSink {
    client: featureflags::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient<Channel>,
}

impl ApiKeySourceMessageSink for FlagEvaluationSyncSink {
    fn forward(&self, record: ApiKeySourceMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pull_flag_evaluations(featureflags::PullFlagEvaluationsRequest {
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        flag_evaluation_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
}

impl ApiKeySourceMessageSink for KeyRotationWorkflowSink {
    fn forward(&self, record: ApiKeySourceMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .cancel_key_rotation_workflow(keymanagement::CancelKeyRotationWorkflowRequest {
                    details: Some(keymanagement::cancel_key_rotation_workflow_request::Details {
                        step: record.status,
                        ..Default::default()
                    }),
                    key_rotation: Some(keymanagement::KeyRotation {
                        key_rotation_id: record.correlation_id,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.EnvironmentWorkflowService`.
struct EnvironmentWorkflowSink {
    client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient<Channel>,
}

impl ApiKeySourceMessageSink for EnvironmentWorkflowSink {
    fn forward(&self, record: ApiKeySourceMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_environment_workflow(configuration::StartEnvironmentWorkflowRequest {
                    environment_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ApiKeySourceMessageLogOnly;

impl ApiKeySourceMessageSink for ApiKeySourceMessageLogOnly {
    fn forward(&self, record: ApiKeySourceMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_api_key_source_message(
    kind: &str,
    record: ApiKeySourceMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let certificate_validation = CertificateValidationSink {
        client: certificates::certificate_validation_service_client::CertificateValidationServiceClient::new(
            channel.clone(),
        ),
    };
    let flag_evaluation_sync = FlagEvaluationSyncSink {
        client: featureflags::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient::new(
            channel.clone(),
        ),
    };
    let key_rotation_workflow = KeyRotationWorkflowSink {
        client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(
            channel.clone(),
        ),
    };
    let environment_workflow = EnvironmentWorkflowSink {
        client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(channel),
    };
    let write: &dyn Fn(ApiKeySourceMessageRecord) -> Delivery = match kind {
        "certificate-validation" => &|record| certificate_validation.forward(record),
        "flag-evaluation-sync" => &|record| flag_evaluation_sync.forward(record),
        "key-rotation-workflow" => &|record| key_rotation_workflow.forward(record),
        "environment-workflow" => &|record| environment_workflow.forward(record),
        _ => &|record| ApiKeySourceMessageLogOnly.forward(record),
    };
    write(record).await
}
