//! `propagate_environment_workflow_id` reads
//! `acme.configuration.v1.EnvironmentWorkflowService/PauseEnvironmentWorkflow` and hands what it keeps to
//! `acme.boxoffice.v1.BoxOfficeSaleQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(channel.clone());
    let reply = source
        .pause_environment_workflow(configuration::PauseEnvironmentWorkflowRequest {
            context: Some(context.clone()),
            reason: "primary-propagate-environment-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let environment = reply.environment.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = EnvironmentWorkflowIdRecord {
        environment_id: environment.environment_id,
        message: acme_kit::text::normalize(&result.message),
        code: environment.code,
        reference_id: outcome.reference_id,
    };
    BoxOfficeSaleQuerySink {
        client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(channel),
    }
    .push(record)
    .await
}

/// What `propagate_environment_workflow_id` keeps of each PauseEnvironmentWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentWorkflowIdRecord {
    pub environment_id: String,
    pub message: String,
    pub code: String,
    pub reference_id: String,
}

/// Where `propagate_environment_workflow_id` hands each record.
trait EnvironmentWorkflowIdDestination {
    fn push(&self, record: EnvironmentWorkflowIdRecord) -> Delivery;
}

/// Passes records to `acme.boxoffice.v1.BoxOfficeSaleQueryService`.
struct BoxOfficeSaleQuerySink {
    client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient<Channel>,
}

impl EnvironmentWorkflowIdDestination for BoxOfficeSaleQuerySink {
    fn push(&self, record: EnvironmentWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_box_office_sales(boxoffice::FindBoxOfficeSalesRequest {
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    order_by: record.environment_id,
                    filter: Some(boxoffice::find_box_office_sales_request::Filter {
                        region: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
