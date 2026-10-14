//! `export_product_reviews_token` reads `acme.reviews.v1.ProductReviewService/ListProductReviews` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::inventory::v1 as inventory;
use acme_proto::acme::reviews::v1 as reviews;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = reviews::product_review_service_client::ProductReviewServiceClient::new(channel.clone());
    let loaded = source
        .list_product_reviews(reviews::ListProductReviewsRequest {
            context: Some(context.clone()),
            order_by: "main-export-product-reviews-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = loaded.page_info.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = ProductReviewsTokenRecord {
        next_page_token: page_info.next_page_token,
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn ProductReviewsTokenSink>)> = vec![
        (
            "certificate-approval",
            Box::new(CertificateApprovalSink {
                client: certificates::certificate_approval_service_client::CertificateApprovalServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "stock-item-query",
            Box::new(StockItemQuerySink {
                client: inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "document-query",
            Box::new(DocumentQuerySink {
                client: documents::document_query_service_client::DocumentQueryServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "document-query" {
            continue;
        }
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `export_product_reviews_token` keeps of each ListProductReviews response.
#[derive(Clone, Debug, Default)]
pub struct ProductReviewsTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `export_product_reviews_token` hands each record.
trait ProductReviewsTokenSink {
    fn forward(&self, record: ProductReviewsTokenRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CertificateApprovalService`.
struct CertificateApprovalSink {
    client: certificates::certificate_approval_service_client::CertificateApprovalServiceClient<Channel>,
}

impl ProductReviewsTokenSink for CertificateApprovalSink {
    fn forward(&self, record: ProductReviewsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .escalate_certificate(certificates::EscalateCertificateRequest {
                    details: Some(certificates::escalate_certificate_request::Details {
                        approver_email: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.inventory.v1.StockItemQueryService`.
struct StockItemQuerySink {
    client: inventory::stock_item_query_service_client::StockItemQueryServiceClient<Channel>,
}

impl ProductReviewsTokenSink for StockItemQuerySink {
    fn forward(&self, record: ProductReviewsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_stock_item(inventory::PeekStockItemRequest {
                    stock_item_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.documents.v1.DocumentQueryService`.
struct DocumentQuerySink {
    client: documents::document_query_service_client::DocumentQueryServiceClient<Channel>,
}

impl ProductReviewsTokenSink for DocumentQuerySink {
    fn forward(&self, record: ProductReviewsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_documents(documents::FindDocumentsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
