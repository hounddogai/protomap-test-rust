//! `copy_secret_workflow_id` reads `acme.secrets.v1.SecretWorkflowService/RetrySecretWorkflow` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::workplaces::v1 as workplaces;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = secrets::secret_workflow_service_client::SecretWorkflowServiceClient::new(channel.clone());
    let reply = source
        .retry_secret_workflow(secrets::RetrySecretWorkflowRequest {
            context: Some(context.clone()),
            reason: "nightly-copy-secret-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let secret = reply.secret.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = SecretWorkflowIdRecord {
        secret_id: secret.secret_id,
        status: acme_kit::text::normalize(&result.status),
        outcome_status: outcome.status,
        reference_id: outcome.reference_id,
    };
    let sinks: Vec<(&str, Box<dyn SecretWorkflowIdDestination>)> = vec![
        (
            "care-plan-calculation",
            Box::new(CarePlanCalculationSink {
                client: careplans::care_plan_calculation_service_client::CarePlanCalculationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "feature-flag-admin",
            Box::new(FeatureFlagAdminSink {
                client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "config-change-validation",
            Box::new(ConfigChangeValidationSink {
                client:
                    configuration::config_change_validation_service_client::ConfigChangeValidationServiceClient::new(
                        channel.clone(),
                    ),
                context: context.clone(),
            }),
        ),
        (
            "building-query",
            Box::new(BuildingQuerySink {
                client: workplaces::building_query_service_client::BuildingQueryServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "building-query" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `copy_secret_workflow_id` keeps of each RetrySecretWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct SecretWorkflowIdRecord {
    pub secret_id: String,
    pub status: String,
    pub outcome_status: String,
    pub reference_id: String,
}

/// Where `copy_secret_workflow_id` hands each record.
trait SecretWorkflowIdDestination {
    fn apply(&self, record: SecretWorkflowIdRecord) -> Delivery;
}

/// Writes records to `acme.careplans.v1.CarePlanCalculationService`.
struct CarePlanCalculationSink {
    client: careplans::care_plan_calculation_service_client::CarePlanCalculationServiceClient<Channel>,
}

impl SecretWorkflowIdDestination for CarePlanCalculationSink {
    fn apply(&self, record: SecretWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_care_plan(careplans::CalculateCarePlanRequest {
                    reason: record.reference_id,
                    care_plan: Some(careplans::CarePlan {
                        care_plan_id: record.secret_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FeatureFlagAdminService`.
struct FeatureFlagAdminSink {
    client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient<Channel>,
    context: common::RequestContext,
}

impl SecretWorkflowIdDestination for FeatureFlagAdminSink {
    fn apply(&self, record: SecretWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::UnlockFeatureFlagRequest {
                    details: Some(featureflags::unlock_feature_flag_request::Details {
                        reason_code: record.outcome_status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.unlock_feature_flag(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigChangeValidationService`.
struct ConfigChangeValidationSink {
    client: configuration::config_change_validation_service_client::ConfigChangeValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SecretWorkflowIdDestination for ConfigChangeValidationSink {
    fn apply(&self, record: SecretWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::ValidateConfigChangeRequest {
                    reason: record.status,
                    config_change: Some(configuration::ConfigChange {
                        config_change_id: record.secret_id,
                        ..Default::default()
                    }),
                    details: Some(configuration::validate_config_change_request::Details {
                        locale: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.validate_config_change(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.workplaces.v1.BuildingQueryService`.
struct BuildingQuerySink {
    client: workplaces::building_query_service_client::BuildingQueryServiceClient<Channel>,
}

impl SecretWorkflowIdDestination for BuildingQuerySink {
    fn apply(&self, record: SecretWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_buildings(workplaces::SuggestBuildingsRequest {
                    filter: Some(workplaces::suggest_buildings_request::Filter {
                        region: record.outcome_status,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.secret_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
