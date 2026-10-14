//! `copy_prescription_id` reads `acme.prescriptions.v1.PrescriptionService/UpdatePrescription` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::prescriptions::v1 as prescriptions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = prescriptions::prescription_service_client::PrescriptionServiceClient::new(channel.clone());
    let loaded = source
        .update_prescription(prescriptions::UpdatePrescriptionRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let prescription = loaded.prescription.unwrap_or_default();
    let record = PrescriptionIdRecord {
        correlation_id: result.correlation_id,
        number: acme_kit::text::normalize(&prescription.number),
    };
    let sinks: Vec<Box<dyn PrescriptionIdDestination>> = vec![
        Box::new(FlagRolloutSyncSink {
            client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient::new(channel.clone()),
        }),
        Box::new(KeyRingQuerySink {
            client: keymanagement::key_ring_query_service_client::KeyRingQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(FeatureFlagHistorySink {
            client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(CryptoKeyCalculationSink {
            client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(
                channel,
            ),
        }),
    ];
    sinks[1].apply(record).await
}

/// What `copy_prescription_id` keeps of each UpdatePrescription response.
#[derive(Clone, Debug, Default)]
pub struct PrescriptionIdRecord {
    pub correlation_id: String,
    pub number: String,
}

/// Where `copy_prescription_id` hands each record.
trait PrescriptionIdDestination {
    fn apply(&self, record: PrescriptionIdRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FlagRolloutSyncService`.
struct FlagRolloutSyncSink {
    client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient<Channel>,
}

impl PrescriptionIdDestination for FlagRolloutSyncSink {
    fn apply(&self, record: PrescriptionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .sync_flag_rollouts(featureflags::SyncFlagRolloutsRequest {
                    details: Some(featureflags::sync_flag_rollouts_request::Details {
                        cursor: record.correlation_id,
                        checksum: record.number,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRingQueryService`.
struct KeyRingQuerySink {
    client: keymanagement::key_ring_query_service_client::KeyRingQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl PrescriptionIdDestination for KeyRingQuerySink {
    fn apply(&self, record: PrescriptionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::SummarizeKeyRingsRequest {
                    key_ring_id: record.number,
                    details: Some(keymanagement::summarize_key_rings_request::Details {
                        format: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_key_rings(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FeatureFlagHistoryService`.
struct FeatureFlagHistorySink {
    client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient<Channel>,
}

impl PrescriptionIdDestination for FeatureFlagHistorySink {
    fn apply(&self, record: PrescriptionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_feature_flag_version(featureflags::GetFeatureFlagVersionRequest {
                    feature_flag_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
}

impl PrescriptionIdDestination for CryptoKeyCalculationSink {
    fn apply(&self, record: PrescriptionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_crypto_key_options(keymanagement::CompareCryptoKeyOptionsRequest {
                    crypto_key: Some(keymanagement::CryptoKey {
                        crypto_key_id: record.correlation_id,
                        currency_code: record.number,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
