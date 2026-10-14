//! `hand_off_crypto_key_message` reads `acme.keymanagement.v1.CryptoKeyLifecycleService/SuspendCryptoKey` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(channel.clone());
    let loaded = source
        .suspend_crypto_key(keymanagement::SuspendCryptoKeyRequest {
            context: Some(context.clone()),
            reason: "main-hand-off-crypto-key-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = loaded.outcome.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = OffCryptoKeyMessageRecord {
        message: outcome.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: outcome.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffCryptoKeyMessageDestination>> = BTreeMap::new();
    sinks.insert(
        "gift-card",
        Box::new(GiftCardSink {
            client: giftcards::gift_card_service_client::GiftCardServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "throttle-import",
        Box::new(ThrottleImportSink {
            client: ratelimiting::throttle_import_service_client::ThrottleImportServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "feature-flag-query",
        Box::new(FeatureFlagQuerySink {
            client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("throttle-import") {
        Some(sink) => sink.push(record).await,
        None => anyhow::bail!("no sink registered for throttle-import"),
    }
}

/// What `hand_off_crypto_key_message` keeps of each SuspendCryptoKey response.
#[derive(Clone, Debug, Default)]
pub struct OffCryptoKeyMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `hand_off_crypto_key_message` hands each record.
trait OffCryptoKeyMessageDestination {
    fn push(&self, record: OffCryptoKeyMessageRecord) -> Delivery;
}

/// Hands records to `acme.giftcards.v1.GiftCardService`.
struct GiftCardSink {
    client: giftcards::gift_card_service_client::GiftCardServiceClient<Channel>,
}

impl OffCryptoKeyMessageDestination for GiftCardSink {
    fn push(&self, record: OffCryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_gift_card(giftcards::UpdateGiftCardRequest {
                    gift_card: Some(giftcards::GiftCard {
                        gift_card_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleImportService`.
struct ThrottleImportSink {
    client: ratelimiting::throttle_import_service_client::ThrottleImportServiceClient<Channel>,
}

impl OffCryptoKeyMessageDestination for ThrottleImportSink {
    fn push(&self, record: OffCryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_throttle_import(ratelimiting::RetryThrottleImportRequest {
                    details: Some(ratelimiting::retry_throttle_import_request::Details {
                        initiated_by: record.correlation_id,
                        ..Default::default()
                    }),
                    throttle: Some(ratelimiting::Throttle {
                        source: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FeatureFlagQueryService`.
struct FeatureFlagQuerySink {
    client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient<Channel>,
}

impl OffCryptoKeyMessageDestination for FeatureFlagQuerySink {
    fn push(&self, record: OffCryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_feature_flag(featureflags::LookupFeatureFlagRequest {
                    feature_flag_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
