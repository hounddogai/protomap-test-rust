//! `settle_invoice_batch_id` reads `acme.invoicing.v1.InvoiceBatchService/CreateInvoiceBatch` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::invoicing::v1 as invoicing;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel.clone());
    let loaded = source
        .create_invoice_batch(invoicing::CreateInvoiceBatchRequest {
            context: Some(context.clone()),
            request_id: "nightly-settle-invoice-batch-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let invoice_batch = loaded.invoice_batch.unwrap_or_default();
    let record = InvoiceBatchIdRecord {
        correlation_id: result.correlation_id,
        number: invoice_batch.number,
    };
    let sinks: Vec<Box<dyn InvoiceBatchIdDestination>> = vec![
        Box::new(CryptoKeyLifecycleSink {
            client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(ConcessionStandCalculationSink {
            client:
                concessions::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient::new(
                    channel.clone(),
                ),
            context: context.clone(),
        }),
        Box::new(EnvironmentSyncSink {
            client: configuration::environment_sync_service_client::EnvironmentSyncServiceClient::new(channel),
        }),
    ];
    for sink in &sinks {
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `settle_invoice_batch_id` keeps of each CreateInvoiceBatch response.
#[derive(Clone, Debug, Default)]
pub struct InvoiceBatchIdRecord {
    pub correlation_id: String,
    pub number: String,
}

/// Where `settle_invoice_batch_id` hands each record.
trait InvoiceBatchIdDestination {
    fn write(&self, record: InvoiceBatchIdRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.CryptoKeyLifecycleService`.
struct CryptoKeyLifecycleSink {
    client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient<Channel>,
}

impl InvoiceBatchIdDestination for CryptoKeyLifecycleSink {
    fn write(&self, record: InvoiceBatchIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reinstate_crypto_key(keymanagement::ReinstateCryptoKeyRequest {
                    reason: record.number,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.concessions.v1.ConcessionStandCalculationService`.
struct ConcessionStandCalculationSink {
    client: concessions::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl InvoiceBatchIdDestination for ConcessionStandCalculationSink {
    fn write(&self, record: InvoiceBatchIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(concessions::RecalculateConcessionStandsRequest {
                    concession_stand: Some(concessions::ConcessionStand {
                        code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.recalculate_concession_stands(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.EnvironmentSyncService`.
struct EnvironmentSyncSink {
    client: configuration::environment_sync_service_client::EnvironmentSyncServiceClient<Channel>,
}

impl InvoiceBatchIdDestination for EnvironmentSyncSink {
    fn write(&self, record: InvoiceBatchIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .push_environment(configuration::PushEnvironmentRequest {
                    environment: Some(configuration::Environment {
                        environment_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
