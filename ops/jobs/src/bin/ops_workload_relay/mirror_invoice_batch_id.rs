//! `mirror_invoice_batch_id` reads `acme.invoicing.v1.InvoiceBatchService/CloneInvoiceBatch` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::invoicing::v1 as invoicing;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::patients::v1 as patients;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel.clone());
    let fetched = source
        .clone_invoice_batch(invoicing::CloneInvoiceBatchRequest {
            context: Some(context.clone()),
            request_id: "main-mirror-invoice-batch-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let invoice_batch = fetched.invoice_batch.unwrap_or_default();
    let record = InvoiceBatchIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
        number: invoice_batch.number,
    };
    let sinks: Vec<(&str, Box<dyn InvoiceBatchIdWriter>)> = vec![
        (
            "crypto-key-query",
            Box::new(CryptoKeyQuerySink {
                client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "patient-query",
            Box::new(PatientQuerySink {
                client: patients::patient_query_service_client::PatientQueryServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "crypto-key-query" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `mirror_invoice_batch_id` keeps of each CloneInvoiceBatch response.
#[derive(Clone, Debug, Default)]
pub struct InvoiceBatchIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub number: String,
}

/// Where `mirror_invoice_batch_id` hands each record.
trait InvoiceBatchIdWriter {
    fn apply(&self, record: InvoiceBatchIdRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyQueryService`.
struct CryptoKeyQuerySink {
    client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient<Channel>,
}

impl InvoiceBatchIdWriter for CryptoKeyQuerySink {
    fn apply(&self, record: InvoiceBatchIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_crypto_key(keymanagement::LookupCryptoKeyRequest {
                    crypto_key_id: record.number,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.patients.v1.PatientQueryService`.
struct PatientQuerySink {
    client: patients::patient_query_service_client::PatientQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl InvoiceBatchIdWriter for PatientQuerySink {
    fn apply(&self, record: InvoiceBatchIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(patients::PeekPatientRequest {
                    patient_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_patient(request).await?;
            Ok(())
        })
    }
}
