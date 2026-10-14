//! `backfill_config_set_workflow_id` reads
//! `acme.configuration.v1.ConfigSetWorkflowService/InspectConfigSetWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::maintenance::v1 as maintenance;
use acme_proto::acme::search::v1 as search;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient::new(channel.clone());
    let found = source
        .inspect_config_set_workflow(configuration::InspectConfigSetWorkflowRequest {
            context: Some(context.clone()),
            config_set_id: "nightly-backfill-config-set-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_set = found.config_set.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = ConfigSetWorkflowIdRecord {
        config_set_id: config_set.config_set_id,
        message: acme_kit::text::normalize(&result.message),
    };
    deliver_config_set_workflow_id("work-order-calculation", record, channel, &context).await
}

/// What `backfill_config_set_workflow_id` keeps of each InspectConfigSetWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetWorkflowIdRecord {
    pub config_set_id: String,
    pub message: String,
}

/// Where `backfill_config_set_workflow_id` hands each record.
trait ConfigSetWorkflowIdWriter {
    fn forward(&self, record: ConfigSetWorkflowIdRecord) -> Delivery;
}

/// Hands records to `acme.search.v1.SearchIndexService`.
struct SearchIndexSink {
    client: search::search_index_service_client::SearchIndexServiceClient<Channel>,
}

impl ConfigSetWorkflowIdWriter for SearchIndexSink {
    fn forward(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_search_index(search::GetSearchIndexRequest {
                    search_index_id: record.config_set_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.maintenance.v1.WorkOrderCalculationService`.
struct WorkOrderCalculationSink {
    client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetWorkflowIdWriter for WorkOrderCalculationSink {
    fn forward(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(maintenance::GetWorkOrderRatesRequest {
                    work_order_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_work_order_rates(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ConfigSetWorkflowIdLogOnly;

impl ConfigSetWorkflowIdWriter for ConfigSetWorkflowIdLogOnly {
    fn forward(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
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
        "search-index" => Box::new(SearchIndexSink {
            client: search::search_index_service_client::SearchIndexServiceClient::new(channel.clone()),
        }),
        "work-order-calculation" => Box::new(WorkOrderCalculationSink {
            client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(ConfigSetWorkflowIdLogOnly),
    };
    sink.forward(record).await
}
