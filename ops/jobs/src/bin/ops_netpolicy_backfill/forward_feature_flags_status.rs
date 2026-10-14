//! `forward_feature_flags_status` reads `acme.featureflags.v1.FeatureFlagSyncService/SyncFeatureFlags` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient::new(channel.clone());
    let found = source
        .sync_feature_flags(featureflags::SyncFeatureFlagsRequest {
            context: Some(context.clone()),
            feature_flag_id: "main-forward-feature-flags-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let feature_flag = found.feature_flag.unwrap_or_default();
    let record = FeatureFlagsStatusRecord {
        status: outcome.status,
        description: feature_flag.description,
    };
    act_on_feature_flags_status("certificate-validation", record, channel, context).await
}

/// What `forward_feature_flags_status` keeps of each SyncFeatureFlags response.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlagsStatusRecord {
    pub status: String,
    pub description: String,
}

/// Where `forward_feature_flags_status` hands each record.
trait FeatureFlagsStatusSink {
    fn push(&self, record: FeatureFlagsStatusRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigChangeWorkflowService`.
struct ConfigChangeWorkflowSink {
    client: configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient<Channel>,
}

impl FeatureFlagsStatusSink for ConfigChangeWorkflowSink {
    fn push(&self, record: FeatureFlagsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_config_change_workflow(configuration::RetryConfigChangeWorkflowRequest {
                    config_change: Some(configuration::ConfigChange {
                        config_change_id: record.status,
                        source: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.certificates.v1.CertificateValidationService`.
struct CertificateValidationSink {
    client: certificates::certificate_validation_service_client::CertificateValidationServiceClient<Channel>,
}

impl FeatureFlagsStatusSink for CertificateValidationSink {
    fn push(&self, record: FeatureFlagsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_certificate(certificates::NormalizeCertificateRequest {
                    certificate: Some(certificates::Certificate {
                        certificate_id: record.status,
                        notes: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.authorization.v1.PermissionCheckNotificationService`.
struct PermissionCheckNotificationSink {
    client:
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient<Channel>,
}

impl FeatureFlagsStatusSink for PermissionCheckNotificationSink {
    fn push(&self, record: FeatureFlagsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_permission_check_preferences(authorization::GetPermissionCheckPreferencesRequest {
                    permission_check_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_feature_flags_status(
    kind: &str,
    record: FeatureFlagsStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(FeatureFlagsStatusRecord) -> Delivery>> = BTreeMap::new();
    let config_change_workflow_channel = channel.clone();
    actions.insert(
        "config-change-workflow",
        Box::new(move |record| {
            ConfigChangeWorkflowSink {
                client: configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient::new(
                    config_change_workflow_channel.clone(),
                ),
            }
            .push(record)
        }),
    );
    let certificate_validation_channel = channel.clone();
    actions.insert(
        "certificate-validation",
        Box::new(move |record| {
            CertificateValidationSink {
                client: certificates::certificate_validation_service_client::CertificateValidationServiceClient::new(
                    certificate_validation_channel.clone(),
                ),
            }
            .push(record)
        }),
    );
    let permission_check_notification_channel = channel.clone();
    actions.insert("permission-check-notification", Box::new(move |record| PermissionCheckNotificationSink { client: authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(permission_check_notification_channel.clone()) }.push(record)));
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
