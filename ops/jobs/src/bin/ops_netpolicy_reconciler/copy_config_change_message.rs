//! `copy_config_change_message` reads `acme.configuration.v1.ConfigChangeValidationService/ScoreConfigChange`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::config_change_validation_service_client::ConfigChangeValidationServiceClient::new(
        channel.clone(),
    );
    let fetched = source
        .score_config_change(configuration::ScoreConfigChangeRequest {
            context: Some(context.clone()),
            reason: "current-copy-config-change-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let config_change = fetched.config_change.unwrap_or_default();
    let record = ConfigChangeMessageRecord {
        message: result.message,
        source: acme_kit::text::normalize(&config_change.source),
        correlation_id: result.correlation_id,
        status: result.status,
    };
    let mut sinks: BTreeMap<String, Box<dyn ConfigChangeMessageDestination>> = BTreeMap::new();
    sinks.insert(
        "flag-rollout-calculation".to_owned(),
        Box::new(FlagRolloutCalculationSink {
            client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "fixed-asset-query".to_owned(),
        Box::new(FixedAssetQuerySink {
            client: assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert("certificate-authority-workflow".to_owned(), Box::new(CertificateAuthorityWorkflowSink { client: certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient::new(channel) }));
    let target = std::env::var("COPY_CONFIG_CHANGE_MESSAGE_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.push(record).await,
        None => Ok(()),
    }
}

/// What `copy_config_change_message` keeps of each ScoreConfigChange response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeMessageRecord {
    pub message: String,
    pub source: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `copy_config_change_message` hands each record.
trait ConfigChangeMessageDestination {
    fn push(&self, record: ConfigChangeMessageRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FlagRolloutCalculationService`.
struct FlagRolloutCalculationSink {
    client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigChangeMessageDestination for FlagRolloutCalculationSink {
    fn push(&self, record: ConfigChangeMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::RecalculateFlagRolloutsRequest {
                    flag_rollout: Some(featureflags::FlagRollout {
                        title: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.recalculate_flag_rollouts(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.assets.v1.FixedAssetQueryService`.
struct FixedAssetQuerySink {
    client: assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient<Channel>,
}

impl ConfigChangeMessageDestination for FixedAssetQuerySink {
    fn push(&self, record: ConfigChangeMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_fixed_assets(assets::CountFixedAssetsRequest {
                    filter: Some(assets::count_fixed_assets_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateAuthorityWorkflowService`.
struct CertificateAuthorityWorkflowSink {
    client:
        certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient<Channel>,
}

impl ConfigChangeMessageDestination for CertificateAuthorityWorkflowSink {
    fn push(&self, record: ConfigChangeMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .inspect_certificate_authority_workflow(certificates::InspectCertificateAuthorityWorkflowRequest {
                    certificate_authority_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
