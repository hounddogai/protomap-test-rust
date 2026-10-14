//! `relay_documents_status` reads `acme.documents.v1.DocumentQueryService/CountDocuments` and hands what it
//! keeps to `acme.catalog.v1.ProductCalculationService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::catalog::v1 as catalog;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = documents::document_query_service_client::DocumentQueryServiceClient::new(channel.clone());
    let fetched = source
        .count_documents(documents::CountDocumentsRequest {
            context: Some(context.clone()),
            order_by: "nightly-relay-documents-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = DocumentsStatusRecord {
        status: result.status,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    ProductCalculationSink {
        client: catalog::product_calculation_service_client::ProductCalculationServiceClient::new(channel),
    }
    .deliver(record)
    .await
}

/// What `relay_documents_status` keeps of each CountDocuments response.
#[derive(Clone, Debug, Default)]
pub struct DocumentsStatusRecord {
    pub status: String,
    pub correlation_id: String,
}

/// Where `relay_documents_status` hands each record.
trait DocumentsStatusSink {
    fn deliver(&self, record: DocumentsStatusRecord) -> Delivery;
}

/// Sends records to `acme.catalog.v1.ProductCalculationService`.
struct ProductCalculationSink {
    client: catalog::product_calculation_service_client::ProductCalculationServiceClient<Channel>,
}

impl DocumentsStatusSink for ProductCalculationSink {
    fn deliver(&self, record: DocumentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_product(catalog::EstimateProductRequest {
                    reason: record.status,
                    product: Some(catalog::Product {
                        code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
