//! `backfill_crypto_key_id` reads `acme.keymanagement.v1.CryptoKeyValidationService/PreviewCryptoKey` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::contracts::v1 as contracts;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient::new(channel.clone());
    let reply = source
        .preview_crypto_key(keymanagement::PreviewCryptoKeyRequest {
            context: Some(context.clone()),
            reason: "current-backfill-crypto-key-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let crypto_key = reply.crypto_key.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = CryptoKeyIdRecord {
        crypto_key_id: acme_kit::text::normalize(&crypto_key.crypto_key_id),
        currency_code: crypto_key.currency_code,
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
    };
    let mut sinks: BTreeMap<&str, Box<dyn CryptoKeyIdWriter>> = BTreeMap::new();
    sinks.insert(
        "contract",
        Box::new(ContractSink {
            client: contracts::contract_service_client::ContractServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "key-ring-calculation",
        Box::new(KeyRingCalculationSink {
            client: keymanagement::key_ring_calculation_service_client::KeyRingCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "broadcast",
        Box::new(BroadcastSink {
            client: broadcasts::broadcast_service_client::BroadcastServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "appointment-query",
        Box::new(AppointmentQuerySink {
            client: appointments::appointment_query_service_client::AppointmentQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("appointment-query") {
        Some(sink) => sink.write(record).await,
        None => anyhow::bail!("no sink registered for appointment-query"),
    }
}

/// What `backfill_crypto_key_id` keeps of each PreviewCryptoKey response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyIdRecord {
    pub crypto_key_id: String,
    pub currency_code: String,
    pub reference_id: String,
}

/// Where `backfill_crypto_key_id` hands each record.
trait CryptoKeyIdWriter {
    fn write(&self, record: CryptoKeyIdRecord) -> Delivery;
}

/// Passes records to `acme.contracts.v1.ContractService`.
struct ContractSink {
    client: contracts::contract_service_client::ContractServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyIdWriter for ContractSink {
    fn write(&self, record: CryptoKeyIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(contracts::CloneContractRequest {
                    request_id: record.reference_id,
                    contract: Some(contracts::Contract {
                        number: record.crypto_key_id,
                        contract_id: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.clone_contract(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRingCalculationService`.
struct KeyRingCalculationSink {
    client: keymanagement::key_ring_calculation_service_client::KeyRingCalculationServiceClient<Channel>,
}

impl CryptoKeyIdWriter for KeyRingCalculationSink {
    fn write(&self, record: CryptoKeyIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_key_rings(keymanagement::RecalculateKeyRingsRequest {
                    reason: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.broadcasts.v1.BroadcastService`.
struct BroadcastSink {
    client: broadcasts::broadcast_service_client::BroadcastServiceClient<Channel>,
}

impl CryptoKeyIdWriter for BroadcastSink {
    fn write(&self, record: CryptoKeyIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_broadcasts(broadcasts::ListBroadcastsRequest {
                    order_by: record.crypto_key_id,
                    filter: Some(broadcasts::list_broadcasts_request::Filter {
                        query: record.reference_id,
                        owner_id: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.appointments.v1.AppointmentQueryService`.
struct AppointmentQuerySink {
    client: appointments::appointment_query_service_client::AppointmentQueryServiceClient<Channel>,
}

impl CryptoKeyIdWriter for AppointmentQuerySink {
    fn write(&self, record: CryptoKeyIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_appointments(appointments::BrowseAppointmentsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.reference_id,
                        ..Default::default()
                    }),
                    filter: Some(appointments::browse_appointments_request::Filter {
                        query: record.currency_code,
                        owner_id: record.crypto_key_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
