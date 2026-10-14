//! `copy_get_documents_status` reads `acme.documents.v1.DocumentService/BatchGetDocuments` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::inventory::v1 as inventory;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = documents::document_service_client::DocumentServiceClient::new(channel.clone());
    let found = source
        .batch_get_documents(documents::BatchGetDocumentsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = GetDocumentsStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        next_page_token: page_info.next_page_token,
        message: acme_kit::text::normalize(&result.message),
    };
    let sinks: Vec<Box<dyn GetDocumentsStatusTarget>> = vec![
        Box::new(SellerSink {
            client: sellers::seller_service_client::SellerServiceClient::new(channel.clone()),
        }),
        Box::new(RateLimitWorkflowSink {
            client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(StockItemQuerySink {
            client: inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[2].forward(record).await
}

/// What `copy_get_documents_status` keeps of each BatchGetDocuments response.
#[derive(Clone, Debug, Default)]
pub struct GetDocumentsStatusRecord {
    pub status: String,
    pub next_page_token: String,
    pub message: String,
}

/// Where `copy_get_documents_status` hands each record.
trait GetDocumentsStatusTarget {
    fn forward(&self, record: GetDocumentsStatusRecord) -> Delivery;
}

/// Writes records to `acme.sellers.v1.SellerService`.
struct SellerSink {
    client: sellers::seller_service_client::SellerServiceClient<Channel>,
}

impl GetDocumentsStatusTarget for SellerSink {
    fn forward(&self, record: GetDocumentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_sellers(sellers::ListSellersRequest {
                    filter: Some(sellers::list_sellers_request::Filter {
                        owner_id: record.status,
                        region: record.next_page_token,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl GetDocumentsStatusTarget for RateLimitWorkflowSink {
    fn forward(&self, record: GetDocumentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ListRateLimitWorkflowsRequest {
                    filter: Some(ratelimiting::list_rate_limit_workflows_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_rate_limit_workflows(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.inventory.v1.StockItemQueryService`.
struct StockItemQuerySink {
    client: inventory::stock_item_query_service_client::StockItemQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl GetDocumentsStatusTarget for StockItemQuerySink {
    fn forward(&self, record: GetDocumentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(inventory::FindStockItemsRequest {
                    order_by: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.find_stock_items(request).await?;
            Ok(())
        })
    }
}
