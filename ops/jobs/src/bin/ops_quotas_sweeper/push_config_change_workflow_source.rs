//! `push_config_change_workflow_source` reads
//! `acme.configuration.v1.ConfigChangeWorkflowService/RetryConfigChangeWorkflow` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient::new(channel.clone());
    let found = source
        .retry_config_change_workflow(configuration::RetryConfigChangeWorkflowRequest {
            context: Some(context.clone()),
            config_change_id: "default-push-config-change-workflow-source".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_change = found.config_change.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = ConfigChangeWorkflowSourceRecord {
        source: acme_kit::text::normalize(&config_change.source),
        config_change_id: config_change.config_change_id,
        message: acme_kit::text::normalize(&outcome.message),
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn ConfigChangeWorkflowSourceSink>)> = vec![
        (
            "audit-event",
            Box::new(AuditEventSink {
                client: audit::audit_event_service_client::AuditEventServiceClient::new(channel.clone()),
            }),
        ),
        (
            "video-visit-calculation",
            Box::new(VideoVisitCalculationSink {
                client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "secret-version-validation",
            Box::new(SecretVersionValidationSink {
                client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "audit-event" {
            continue;
        }
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `push_config_change_workflow_source` keeps of each RetryConfigChangeWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeWorkflowSourceRecord {
    pub source: String,
    pub config_change_id: String,
    pub message: String,
    pub status: String,
}

/// Where `push_config_change_workflow_source` hands each record.
trait ConfigChangeWorkflowSourceSink {
    fn forward(&self, record: ConfigChangeWorkflowSourceRecord) -> Delivery;
}

/// Hands records to `acme.audit.v1.AuditEventService`.
struct AuditEventSink {
    client: audit::audit_event_service_client::AuditEventServiceClient<Channel>,
}

impl ConfigChangeWorkflowSourceSink for AuditEventSink {
    fn forward(&self, record: ConfigChangeWorkflowSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_audit_event(audit::DeleteAuditEventRequest {
                    audit_event_id: record.source,
                    reason: record.status,
                    etag: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.telehealth.v1.VideoVisitCalculationService`.
struct VideoVisitCalculationSink {
    client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient<Channel>,
}

impl ConfigChangeWorkflowSourceSink for VideoVisitCalculationSink {
    fn forward(&self, record: ConfigChangeWorkflowSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_video_visit_options(telehealth::CompareVideoVisitOptionsRequest {
                    video_visit: Some(telehealth::VideoVisit {
                        title: record.source,
                        video_visit_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.secrets.v1.SecretVersionValidationService`.
struct SecretVersionValidationSink {
    client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient<Channel>,
}

impl ConfigChangeWorkflowSourceSink for SecretVersionValidationSink {
    fn forward(&self, record: ConfigChangeWorkflowSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_secret_version_rules(secrets::GetSecretVersionRulesRequest {
                    secret_version_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
