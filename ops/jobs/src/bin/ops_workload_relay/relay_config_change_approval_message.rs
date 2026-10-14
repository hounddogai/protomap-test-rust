//! `relay_config_change_approval_message` reads
//! `acme.configuration.v1.ConfigChangeApprovalService/DelegateConfigChangeApproval` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient::new(channel.clone());
    let found = source
        .delegate_config_change_approval(configuration::DelegateConfigChangeApprovalRequest {
            context: Some(context.clone()),
            reason: "main-relay-config-change-approval-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let config_change = found.config_change.unwrap_or_default();
    let record = ConfigChangeApprovalMessageRecord {
        message: outcome.message,
        status: outcome.status,
        config_change_id: config_change.config_change_id,
        source: config_change.source,
    };
    let sinks: Vec<Box<dyn ConfigChangeApprovalMessageWriter>> = vec![
        Box::new(FeatureFlagSyncSink {
            client: featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient::new(channel.clone()),
        }),
        Box::new(AdmissionCalculationSink {
            client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[0].push(record).await
}

/// What `relay_config_change_approval_message` keeps of each DelegateConfigChangeApproval response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeApprovalMessageRecord {
    pub message: String,
    pub status: String,
    pub config_change_id: String,
    pub source: String,
}

/// Where `relay_config_change_approval_message` hands each record.
trait ConfigChangeApprovalMessageWriter {
    fn push(&self, record: ConfigChangeApprovalMessageRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FeatureFlagSyncService`.
struct FeatureFlagSyncSink {
    client: featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient<Channel>,
}

impl ConfigChangeApprovalMessageWriter for FeatureFlagSyncSink {
    fn push(&self, record: ConfigChangeApprovalMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resolve_feature_flag_conflict(featureflags::ResolveFeatureFlagConflictRequest {
                    details: Some(featureflags::resolve_feature_flag_conflict_request::Details {
                        checksum: record.source,
                        ..Default::default()
                    }),
                    reason: record.message,
                    feature_flag: Some(featureflags::FeatureFlag {
                        name: record.config_change_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.admissions.v1.AdmissionCalculationService`.
struct AdmissionCalculationSink {
    client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigChangeApprovalMessageWriter for AdmissionCalculationSink {
    fn push(&self, record: ConfigChangeApprovalMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(admissions::CompareAdmissionOptionsRequest {
                    details: Some(admissions::compare_admission_options_request::Details {
                        currency_code: record.status,
                        ..Default::default()
                    }),
                    reason: record.source,
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_admission_options(request).await?;
            Ok(())
        })
    }
}
