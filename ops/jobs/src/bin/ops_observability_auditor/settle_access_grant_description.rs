//! `settle_access_grant_description` reads
//! `acme.secrets.v1.AccessGrantCalculationService/SimulateAccessGrant` and hands what it keeps to the sink of
//! its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::procurement::v1 as procurement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient::new(channel.clone());
    let reply = source
        .simulate_access_grant(secrets::SimulateAccessGrantRequest {
            context: Some(context.clone()),
            reason: "live-settle-access-grant-description".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let access_grant = reply.access_grant.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = AccessGrantDescriptionRecord {
        description: access_grant.description,
        status: result.status,
        name: access_grant.name,
    };
    let mut sinks: BTreeMap<&str, Box<dyn AccessGrantDescriptionSink>> = BTreeMap::new();
    sinks.insert(
        "labeling-task-query",
        Box::new(LabelingTaskQuerySink {
            client: labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "purchase-requisition-query",
        Box::new(PurchaseRequisitionQuerySink {
            client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient::new(
                channel,
            ),
        }),
    );
    let Some(sink) = sinks.get("labeling-task-query") else {
        anyhow::bail!("no sink registered for labeling-task-query");
    };
    sink.send(record).await
}

/// What `settle_access_grant_description` keeps of each SimulateAccessGrant response.
#[derive(Clone, Debug, Default)]
pub struct AccessGrantDescriptionRecord {
    pub description: String,
    pub status: String,
    pub name: String,
}

/// Where `settle_access_grant_description` hands each record.
trait AccessGrantDescriptionSink {
    fn send(&self, record: AccessGrantDescriptionRecord) -> Delivery;
}

/// Writes records to `acme.labeling.v1.LabelingTaskQueryService`.
struct LabelingTaskQuerySink {
    client: labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl AccessGrantDescriptionSink for LabelingTaskQuerySink {
    fn send(&self, record: AccessGrantDescriptionRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(labeling::SuggestLabelingTasksRequest {
                    page: Some(common::PageRequest {
                        page_token: record.name,
                        ..Default::default()
                    }),
                    filter: Some(labeling::suggest_labeling_tasks_request::Filter {
                        region: record.status,
                        ..Default::default()
                    }),
                    order_by: record.description,
                    ..Default::default()
                }),
                &context,
            )?;
            client.suggest_labeling_tasks(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.procurement.v1.PurchaseRequisitionQueryService`.
struct PurchaseRequisitionQuerySink {
    client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient<Channel>,
}

impl AccessGrantDescriptionSink for PurchaseRequisitionQuerySink {
    fn send(&self, record: AccessGrantDescriptionRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_purchase_requisition(procurement::PeekPurchaseRequisitionRequest {
                    purchase_requisition_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
