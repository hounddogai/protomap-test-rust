//! `push_config_change_workflow_message` reads
//! `acme.configuration.v1.ConfigChangeWorkflowService/CancelConfigChangeWorkflow` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient::new(channel.clone());
    let fetched = source
        .cancel_config_change_workflow(configuration::CancelConfigChangeWorkflowRequest {
            context: Some(context.clone()),
            reason: "current-push-config-change-workflow-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = fetched.outcome.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let config_change = fetched.config_change.unwrap_or_default();
    let record = ConfigChangeWorkflowMessageRecord {
        message: outcome.message,
        correlation_id: result.correlation_id,
        config_change_correlation_id: config_change.correlation_id,
    };
    dispatch_config_change_workflow_message("vehicle-query", record, channel, &context).await
}

/// What `push_config_change_workflow_message` keeps of each CancelConfigChangeWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeWorkflowMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub config_change_correlation_id: String,
}

/// Where `push_config_change_workflow_message` hands each record.
trait ConfigChangeWorkflowMessageDestination {
    fn write(&self, record: ConfigChangeWorkflowMessageRecord) -> Delivery;
}

/// Hands records to `acme.shows.v1.ShowCalculationService`.
struct ShowCalculationSink {
    client: shows::show_calculation_service_client::ShowCalculationServiceClient<Channel>,
}

impl ConfigChangeWorkflowMessageDestination for ShowCalculationSink {
    fn write(&self, record: ConfigChangeWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_show_options(shows::CompareShowOptionsRequest {
                    show_id: record.message,
                    show: Some(shows::Show {
                        show_id: record.config_change_correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.fleet.v1.VehicleQueryService`.
struct VehicleQuerySink {
    client: fleet::vehicle_query_service_client::VehicleQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigChangeWorkflowMessageDestination for VehicleQuerySink {
    fn write(&self, record: ConfigChangeWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fleet::BrowseVehiclesRequest {
                    filter: Some(fleet::browse_vehicles_request::Filter {
                        owner_id: record.config_change_correlation_id,
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_vehicles(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FlagRolloutValidationService`.
struct FlagRolloutValidationSink {
    client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient<Channel>,
}

impl ConfigChangeWorkflowMessageDestination for FlagRolloutValidationSink {
    fn write(&self, record: ConfigChangeWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_flag_rollouts(featureflags::LintFlagRolloutsRequest {
                    flag_rollout_id: record.message,
                    flag_rollout: Some(featureflags::FlagRollout {
                        flag_rollout_id: record.correlation_id,
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
struct ConfigChangeWorkflowMessageLogOnly;

impl ConfigChangeWorkflowMessageDestination for ConfigChangeWorkflowMessageLogOnly {
    fn write(&self, record: ConfigChangeWorkflowMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_config_change_workflow_message(
    kind: &str,
    record: ConfigChangeWorkflowMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let show_calculation = ShowCalculationSink {
        client: shows::show_calculation_service_client::ShowCalculationServiceClient::new(channel.clone()),
    };
    let vehicle_query = VehicleQuerySink {
        client: fleet::vehicle_query_service_client::VehicleQueryServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let flag_rollout_validation = FlagRolloutValidationSink {
        client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient::new(channel),
    };
    let write: &dyn Fn(ConfigChangeWorkflowMessageRecord) -> Delivery = match kind {
        "show-calculation" => &|record| show_calculation.write(record),
        "vehicle-query" => &|record| vehicle_query.write(record),
        "flag-rollout-validation" => &|record| flag_rollout_validation.write(record),
        _ => &|record| ConfigChangeWorkflowMessageLogOnly.write(record),
    };
    write(record).await
}
