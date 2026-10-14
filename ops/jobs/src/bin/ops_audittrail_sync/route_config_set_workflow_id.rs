//! `route_config_set_workflow_id` reads
//! `acme.configuration.v1.ConfigSetWorkflowService/ResumeConfigSetWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient::new(channel.clone());
    let fetched = source
        .resume_config_set_workflow(configuration::ResumeConfigSetWorkflowRequest {
            context: Some(context.clone()),
            reason: "nightly-route-config-set-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_set = fetched.config_set.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = ConfigSetWorkflowIdRecord {
        config_set_id: config_set.config_set_id,
        status: outcome.status,
    };
    deliver_config_set_workflow_id("environment-sync", record, channel, &context).await
}

/// What `route_config_set_workflow_id` keeps of each ResumeConfigSetWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetWorkflowIdRecord {
    pub config_set_id: String,
    pub status: String,
}

/// Where `route_config_set_workflow_id` hands each record.
trait ConfigSetWorkflowIdWriter {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetWorkflowIdWriter for RateLimitWorkflowSink {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::InspectRateLimitWorkflowRequest {
                    rate_limit_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.inspect_rate_limit_workflow(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.ConfigSetCalculationService`.
struct ConfigSetCalculationSink {
    client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient<Channel>,
}

impl ConfigSetWorkflowIdWriter for ConfigSetCalculationSink {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_config_set(configuration::SimulateConfigSetRequest {
                    config_set: Some(configuration::ConfigSet {
                        config_set_id: record.config_set_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.EnvironmentSyncService`.
struct EnvironmentSyncSink {
    client: configuration::environment_sync_service_client::EnvironmentSyncServiceClient<Channel>,
}

impl ConfigSetWorkflowIdWriter for EnvironmentSyncSink {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .sync_environments(configuration::SyncEnvironmentsRequest {
                    environment: Some(configuration::Environment {
                        code: record.status,
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
struct ConfigSetWorkflowIdLogOnly;

impl ConfigSetWorkflowIdWriter for ConfigSetWorkflowIdLogOnly {
    fn push(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.config_set_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_config_set_workflow_id(
    kind: &str,
    record: ConfigSetWorkflowIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn ConfigSetWorkflowIdWriter> = match kind {
        "rate-limit-workflow" => Box::new(RateLimitWorkflowSink {
            client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "config-set-calculation" => Box::new(ConfigSetCalculationSink {
            client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        "environment-sync" => Box::new(EnvironmentSyncSink {
            client: configuration::environment_sync_service_client::EnvironmentSyncServiceClient::new(channel),
        }),
        _ => Box::new(ConfigSetWorkflowIdLogOnly),
    };
    sink.push(record).await
}
