//! `hand_off_stock_items_status` reads `acme.inventory.v1.StockItemQueryService/ResolveStockItems` and hands
//! what it keeps to `acme.secrets.v1.AccessGrantWorkflowService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::inventory::v1 as inventory;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel.clone());
    let reply = source
        .resolve_stock_items(inventory::ResolveStockItemsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = OffStockItemsStatusRecord {
        status: result.status,
        message: result.message,
    };
    AccessGrantWorkflowSink {
        client: secrets::access_grant_workflow_service_client::AccessGrantWorkflowServiceClient::new(channel),
    }
    .push(record)
    .await
}

/// What `hand_off_stock_items_status` keeps of each ResolveStockItems response.
#[derive(Clone, Debug, Default)]
pub struct OffStockItemsStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `hand_off_stock_items_status` hands each record.
trait OffStockItemsStatusWriter {
    fn push(&self, record: OffStockItemsStatusRecord) -> Delivery;
}

/// Hands records to `acme.secrets.v1.AccessGrantWorkflowService`.
struct AccessGrantWorkflowSink {
    client: secrets::access_grant_workflow_service_client::AccessGrantWorkflowServiceClient<Channel>,
}

impl OffStockItemsStatusWriter for AccessGrantWorkflowSink {
    fn push(&self, record: OffStockItemsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_access_grant_workflow(secrets::AdvanceAccessGrantWorkflowRequest {
                    details: Some(secrets::advance_access_grant_workflow_request::Details {
                        initiated_by: record.status,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
