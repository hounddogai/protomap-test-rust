//! `sync_purchase_requisitions_status` reads
//! `acme.procurement.v1.PurchaseRequisitionQueryService/ResolvePurchaseRequisitions` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::procurement::v1 as procurement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient::new(
        channel.clone(),
    );
    let fetched = source
        .resolve_purchase_requisitions(procurement::ResolvePurchaseRequisitionsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let page_info = fetched.page_info.unwrap_or_default();
    let record = PurchaseRequisitionsStatusRecord {
        status: result.status,
        message: result.message,
        next_page_token: page_info.next_page_token,
        correlation_id: result.correlation_id,
    };
    dispatch_purchase_requisitions_status("patient-query", record, channel, &context).await
}

/// What `sync_purchase_requisitions_status` keeps of each ResolvePurchaseRequisitions response.
#[derive(Clone, Debug, Default)]
pub struct PurchaseRequisitionsStatusRecord {
    pub status: String,
    pub message: String,
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `sync_purchase_requisitions_status` hands each record.
trait PurchaseRequisitionsStatusTarget {
    fn push(&self, record: PurchaseRequisitionsStatusRecord) -> Delivery;
}

/// Passes records to `acme.patients.v1.PatientQueryService`.
struct PatientQuerySink {
    client: patients::patient_query_service_client::PatientQueryServiceClient<Channel>,
}

impl PurchaseRequisitionsStatusTarget for PatientQuerySink {
    fn push(&self, record: PurchaseRequisitionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_patient(patients::PeekPatientRequest {
                    patient_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.certificates.v1.CertificateService`.
struct CertificateSink {
    client: certificates::certificate_service_client::CertificateServiceClient<Channel>,
    context: common::RequestContext,
}

impl PurchaseRequisitionsStatusTarget for CertificateSink {
    fn push(&self, record: PurchaseRequisitionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::ListCertificatesRequest {
                    filter: Some(certificates::list_certificates_request::Filter {
                        query: record.next_page_token,
                        region: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_certificates(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct PurchaseRequisitionsStatusLogOnly;

impl PurchaseRequisitionsStatusTarget for PurchaseRequisitionsStatusLogOnly {
    fn push(&self, record: PurchaseRequisitionsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_purchase_requisitions_status(
    kind: &str,
    record: PurchaseRequisitionsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let patient_query = PatientQuerySink {
        client: patients::patient_query_service_client::PatientQueryServiceClient::new(channel.clone()),
    };
    let certificate = CertificateSink {
        client: certificates::certificate_service_client::CertificateServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(PurchaseRequisitionsStatusRecord) -> Delivery = match kind {
        "patient-query" => &|record| patient_query.push(record),
        "certificate" => &|record| certificate.push(record),
        _ => &|record| PurchaseRequisitionsStatusLogOnly.push(record),
    };
    write(record).await
}
