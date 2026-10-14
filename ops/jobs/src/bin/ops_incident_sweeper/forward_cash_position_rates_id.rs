//! `forward_cash_position_rates_id` reads
//! `acme.treasury.v1.CashPositionCalculationService/GetCashPositionRates` and hands what it keeps to the sink
//! of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::treasury::v1 as treasury;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        treasury::cash_position_calculation_service_client::CashPositionCalculationServiceClient::new(channel.clone());
    let found = source
        .get_cash_position_rates(treasury::GetCashPositionRatesRequest {
            context: Some(context.clone()),
            cash_position_id: "primary-forward-cash-position-rates-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let cash_position = found.cash_position.unwrap_or_default();
    let record = CashPositionRatesIdRecord {
        correlation_id: result.correlation_id,
        message: acme_kit::text::normalize(&result.message),
        cash_position_id: acme_kit::text::normalize(&cash_position.cash_position_id),
    };
    act_on_cash_position_rates_id("flag-rollout-sync", record, channel, context).await
}

/// What `forward_cash_position_rates_id` keeps of each GetCashPositionRates response.
#[derive(Clone, Debug, Default)]
pub struct CashPositionRatesIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub cash_position_id: String,
}

/// Where `forward_cash_position_rates_id` hands each record.
trait CashPositionRatesIdSink {
    fn send(&self, record: CashPositionRatesIdRecord) -> Delivery;
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyLifecycleService`.
struct CryptoKeyLifecycleSink {
    client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl CashPositionRatesIdSink for CryptoKeyLifecycleSink {
    fn send(&self, record: CashPositionRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ExpireCryptoKeyRequest {
                    crypto_key_id: record.cash_position_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.expire_crypto_key(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagRolloutSyncService`.
struct FlagRolloutSyncSink {
    client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient<Channel>,
}

impl CashPositionRatesIdSink for FlagRolloutSyncSink {
    fn send(&self, record: CashPositionRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resolve_flag_rollout_conflict(featureflags::ResolveFlagRolloutConflictRequest {
                    details: Some(featureflags::resolve_flag_rollout_conflict_request::Details {
                        cursor: record.correlation_id,
                        peer_region: record.message,
                        ..Default::default()
                    }),
                    reason: record.cash_position_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.KeyRingLifecycleService`.
struct KeyRingLifecycleSink {
    client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl CashPositionRatesIdSink for KeyRingLifecycleSink {
    fn send(&self, record: CashPositionRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ScheduleKeyRingExpiryRequest {
                    reason: record.cash_position_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.schedule_key_ring_expiry(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_cash_position_rates_id(
    kind: &str,
    record: CashPositionRatesIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(CashPositionRatesIdRecord) -> Delivery>> = BTreeMap::new();
    let crypto_key_lifecycle_channel = channel.clone();
    let crypto_key_lifecycle_context = context.clone();
    actions.insert(
        "crypto-key-lifecycle",
        Box::new(move |record| {
            CryptoKeyLifecycleSink {
                client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(
                    crypto_key_lifecycle_channel.clone(),
                ),
                context: crypto_key_lifecycle_context.clone(),
            }
            .send(record)
        }),
    );
    let flag_rollout_sync_channel = channel.clone();
    actions.insert(
        "flag-rollout-sync",
        Box::new(move |record| {
            FlagRolloutSyncSink {
                client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient::new(
                    flag_rollout_sync_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    let key_ring_lifecycle_channel = channel.clone();
    let key_ring_lifecycle_context = context.clone();
    actions.insert(
        "key-ring-lifecycle",
        Box::new(move |record| {
            KeyRingLifecycleSink {
                client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient::new(
                    key_ring_lifecycle_channel.clone(),
                ),
                context: key_ring_lifecycle_context.clone(),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
