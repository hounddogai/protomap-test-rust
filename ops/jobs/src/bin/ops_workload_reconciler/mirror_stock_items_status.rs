//! `mirror_stock_items_status` reads `acme.inventory.v1.StockItemQueryService/FindStockItems` and hands what
//! it keeps to `acme.ratelimiting.v1.QuotaBucketApprovalService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::inventory::v1 as inventory;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel.clone());
    let loaded = source
        .find_stock_items(inventory::FindStockItemsRequest {
            context: Some(context.clone()),
            order_by: "live-mirror-stock-items-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = StockItemsStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
    };
    QuotaBucketApprovalSink {
        client: ratelimiting::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `mirror_stock_items_status` keeps of each FindStockItems response.
#[derive(Clone, Debug, Default)]
pub struct StockItemsStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub next_page_token: String,
}

/// Where `mirror_stock_items_status` hands each record.
trait StockItemsStatusSink {
    fn forward(&self, record: StockItemsStatusRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.QuotaBucketApprovalService`.
struct QuotaBucketApprovalSink {
    client: ratelimiting::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient<Channel>,
}

impl StockItemsStatusSink for QuotaBucketApprovalSink {
    fn forward(&self, record: StockItemsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .comment_quota_bucket_approval(ratelimiting::CommentQuotaBucketApprovalRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        quota_bucket_id: record.next_page_token,
                        ..Default::default()
                    }),
                    reason: record.status,
                    details: Some(ratelimiting::comment_quota_bucket_approval_request::Details {
                        comment: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
