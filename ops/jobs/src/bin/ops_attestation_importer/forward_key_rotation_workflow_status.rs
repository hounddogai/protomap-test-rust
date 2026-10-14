//! `forward_key_rotation_workflow_status` reads
//! `acme.keymanagement.v1.KeyRotationWorkflowService/ResumeKeyRotationWorkflow` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::fuel::v1 as fuel;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::labeling::v1 as labeling;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(channel.clone());
    let reply = source
        .resume_key_rotation_workflow(keymanagement::ResumeKeyRotationWorkflowRequest {
            context: Some(context.clone()),
            reason: "live-forward-key-rotation-workflow-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let key_rotation = reply.key_rotation.unwrap_or_default();
    let record = KeyRotationWorkflowStatusRecord {
        status: acme_kit::text::normalize(&outcome.status),
        reference_id: outcome.reference_id,
        message: outcome.message,
        title: key_rotation.title,
    };
    dispatch_key_rotation_workflow_status("environment-calculation", record, channel, &context).await
}

/// What `forward_key_rotation_workflow_status` keeps of each ResumeKeyRotationWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct KeyRotationWorkflowStatusRecord {
    pub status: String,
    pub reference_id: String,
    pub message: String,
    pub title: String,
}

/// Where `forward_key_rotation_workflow_status` hands each record.
trait KeyRotationWorkflowStatusDestination {
    fn write(&self, record: KeyRotationWorkflowStatusRecord) -> Delivery;
}

/// Writes records to `acme.configuration.v1.EnvironmentCalculationService`.
struct EnvironmentCalculationSink {
    client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl KeyRotationWorkflowStatusDestination for EnvironmentCalculationSink {
    fn write(&self, record: KeyRotationWorkflowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::SimulateEnvironmentRequest {
                    environment: Some(configuration::Environment {
                        environment_id: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.simulate_environment(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.labeling.v1.LabelingTaskService`.
struct LabelingTaskSink {
    client: labeling::labeling_task_service_client::LabelingTaskServiceClient<Channel>,
}

impl KeyRotationWorkflowStatusDestination for LabelingTaskSink {
    fn write(&self, record: KeyRotationWorkflowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_labeling_tasks(labeling::ListLabelingTasksRequest {
                    filter: Some(labeling::list_labeling_tasks_request::Filter {
                        query: record.message,
                        owner_id: record.title,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.fuel.v1.FuelCardQueryService`.
struct FuelCardQuerySink {
    client: fuel::fuel_card_query_service_client::FuelCardQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl KeyRotationWorkflowStatusDestination for FuelCardQuerySink {
    fn write(&self, record: KeyRotationWorkflowStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fuel::SuggestFuelCardsRequest {
                    order_by: record.message,
                    filter: Some(fuel::suggest_fuel_cards_request::Filter {
                        owner_id: record.title,
                        query: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suggest_fuel_cards(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct KeyRotationWorkflowStatusLogOnly;

impl KeyRotationWorkflowStatusDestination for KeyRotationWorkflowStatusLogOnly {
    fn write(&self, record: KeyRotationWorkflowStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_key_rotation_workflow_status(
    kind: &str,
    record: KeyRotationWorkflowStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let environment_calculation = EnvironmentCalculationSink {
        client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient::new(
            channel.clone(),
        ),
        context: context.clone(),
    };
    let labeling_task = LabelingTaskSink {
        client: labeling::labeling_task_service_client::LabelingTaskServiceClient::new(channel.clone()),
    };
    let fuel_card_query = FuelCardQuerySink {
        client: fuel::fuel_card_query_service_client::FuelCardQueryServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(KeyRotationWorkflowStatusRecord) -> Delivery = match kind {
        "environment-calculation" => &|record| environment_calculation.write(record),
        "labeling-task" => &|record| labeling_task.write(record),
        "fuel-card-query" => &|record| fuel_card_query.write(record),
        _ => &|record| KeyRotationWorkflowStatusLogOnly.write(record),
    };
    write(record).await
}
