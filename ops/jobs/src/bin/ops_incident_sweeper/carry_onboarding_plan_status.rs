//! `carry_onboarding_plan_status` reads
//! `acme.onboarding.v1.OnboardingPlanCalculationService/QuoteOnboardingPlan` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::lineage::v1 as lineage;
use acme_proto::acme::onboarding::v1 as onboarding;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        onboarding::onboarding_plan_calculation_service_client::OnboardingPlanCalculationServiceClient::new(
            channel.clone(),
        );
    let reply = source
        .quote_onboarding_plan(onboarding::QuoteOnboardingPlanRequest {
            context: Some(context.clone()),
            reason: "default-carry-onboarding-plan-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let onboarding_plan = reply.onboarding_plan.unwrap_or_default();
    let record = OnboardingPlanStatusRecord {
        status: result.status,
        onboarding_plan_id: onboarding_plan.onboarding_plan_id,
    };
    let sinks: Vec<(&str, Box<dyn OnboardingPlanStatusDestination>)> = vec![
        (
            "secret-version-query",
            Box::new(SecretVersionQuerySink {
                client: secrets::secret_version_query_service_client::SecretVersionQueryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "lineage-node-query",
            Box::new(LineageNodeQuerySink {
                client: lineage::lineage_node_query_service_client::LineageNodeQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "config-change-validation",
            Box::new(ConfigChangeValidationSink {
                client:
                    configuration::config_change_validation_service_client::ConfigChangeValidationServiceClient::new(
                        channel,
                    ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "config-change-validation" {
            continue;
        }
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `carry_onboarding_plan_status` keeps of each QuoteOnboardingPlan response.
#[derive(Clone, Debug, Default)]
pub struct OnboardingPlanStatusRecord {
    pub status: String,
    pub onboarding_plan_id: String,
}

/// Where `carry_onboarding_plan_status` hands each record.
trait OnboardingPlanStatusDestination {
    fn send(&self, record: OnboardingPlanStatusRecord) -> Delivery;
}

/// Sends records to `acme.secrets.v1.SecretVersionQueryService`.
struct SecretVersionQuerySink {
    client: secrets::secret_version_query_service_client::SecretVersionQueryServiceClient<Channel>,
}

impl OnboardingPlanStatusDestination for SecretVersionQuerySink {
    fn send(&self, record: OnboardingPlanStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_secret_versions(secrets::BrowseSecretVersionsRequest {
                    filter: Some(secrets::browse_secret_versions_request::Filter {
                        region: record.onboarding_plan_id,
                        ..Default::default()
                    }),
                    order_by: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.lineage.v1.LineageNodeQueryService`.
struct LineageNodeQuerySink {
    client: lineage::lineage_node_query_service_client::LineageNodeQueryServiceClient<Channel>,
}

impl OnboardingPlanStatusDestination for LineageNodeQuerySink {
    fn send(&self, record: OnboardingPlanStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_lineage_node(lineage::LookupLineageNodeRequest {
                    lineage_node_id: record.onboarding_plan_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.ConfigChangeValidationService`.
struct ConfigChangeValidationSink {
    client: configuration::config_change_validation_service_client::ConfigChangeValidationServiceClient<Channel>,
}

impl OnboardingPlanStatusDestination for ConfigChangeValidationSink {
    fn send(&self, record: OnboardingPlanStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_config_change_eligibility(configuration::CheckConfigChangeEligibilityRequest {
                    details: Some(configuration::check_config_change_eligibility_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    config_change: Some(configuration::ConfigChange {
                        config_change_id: record.onboarding_plan_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
