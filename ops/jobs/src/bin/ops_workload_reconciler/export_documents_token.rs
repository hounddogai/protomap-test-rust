//! `export_documents_token` reads `acme.documents.v1.DocumentQueryService/FindDocuments` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = documents::document_query_service_client::DocumentQueryServiceClient::new(channel.clone());
    let found = source
        .find_documents(documents::FindDocumentsRequest {
            context: Some(context.clone()),
            order_by: "current-export-documents-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = found.page_info.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = DocumentsTokenRecord {
        next_page_token: page_info.next_page_token,
        message: result.message,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn DocumentsTokenTarget>)> = vec![
        (
            "certificate-workflow",
            Box::new(CertificateWorkflowSink {
                client: certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "box-office-sale-query",
            Box::new(BoxOfficeSaleQuerySink {
                client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "quota-bucket-validation",
            Box::new(QuotaBucketValidationSink {
                client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "certificate-workflow" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `export_documents_token` keeps of each FindDocuments response.
#[derive(Clone, Debug, Default)]
pub struct DocumentsTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub status: String,
}

/// Where `export_documents_token` hands each record.
trait DocumentsTokenTarget {
    fn apply(&self, record: DocumentsTokenRecord) -> Delivery;
}

/// Writes records to `acme.certificates.v1.CertificateWorkflowService`.
struct CertificateWorkflowSink {
    client: certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl DocumentsTokenTarget for CertificateWorkflowSink {
    fn apply(&self, record: DocumentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::RetryCertificateWorkflowRequest {
                    certificate: Some(certificates::Certificate {
                        notes: record.next_page_token,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.retry_certificate_workflow(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.boxoffice.v1.BoxOfficeSaleQueryService`.
struct BoxOfficeSaleQuerySink {
    client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient<Channel>,
}

impl DocumentsTokenTarget for BoxOfficeSaleQuerySink {
    fn apply(&self, record: DocumentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_box_office_sales(boxoffice::SuggestBoxOfficeSalesRequest {
                    filter: Some(boxoffice::suggest_box_office_sales_request::Filter {
                        owner_id: record.next_page_token,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    order_by: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketValidationService`.
struct QuotaBucketValidationSink {
    client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient<Channel>,
}

impl DocumentsTokenTarget for QuotaBucketValidationSink {
    fn apply(&self, record: DocumentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_quota_buckets(ratelimiting::LintQuotaBucketsRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        currency_code: record.next_page_token,
                        quota_bucket_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
