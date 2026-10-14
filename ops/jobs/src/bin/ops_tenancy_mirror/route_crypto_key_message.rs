//! `route_crypto_key_message` reads `acme.keymanagement.v1.CryptoKeyService/UpdateCryptoKey` and hands what
//! it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::media::v1 as media;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = keymanagement::crypto_key_service_client::CryptoKeyServiceClient::new(channel.clone());
    let reply = source
        .update_crypto_key(keymanagement::UpdateCryptoKeyRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = CryptoKeyMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
    };
    act_on_crypto_key_message("feature-flag-validation", record, channel, context).await
}

/// What `route_crypto_key_message` keeps of each UpdateCryptoKey response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyMessageRecord {
    pub message: String,
    pub correlation_id: String,
}

/// Where `route_crypto_key_message` hands each record.
trait CryptoKeyMessageDestination {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery;
}

/// Writes records to `acme.media.v1.MediaAssetService`.
struct MediaAssetSink {
    client: media::media_asset_service_client::MediaAssetServiceClient<Channel>,
}

impl CryptoKeyMessageDestination for MediaAssetSink {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_media_asset(media::UpdateMediaAssetRequest {
                    media_asset: Some(media::MediaAsset {
                        media_asset_id: record.message,
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

/// Writes records to `acme.featureflags.v1.FeatureFlagValidationService`.
struct FeatureFlagValidationSink {
    client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient<Channel>,
}

impl CryptoKeyMessageDestination for FeatureFlagValidationSink {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_feature_flag(featureflags::NormalizeFeatureFlagRequest {
                    details: Some(featureflags::normalize_feature_flag_request::Details {
                        locale: record.correlation_id,
                        ..Default::default()
                    }),
                    feature_flag: Some(featureflags::FeatureFlag {
                        feature_flag_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.fulfillment.v1.FulfillmentQueryService`.
struct FulfillmentQuerySink {
    client: fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient<Channel>,
}

impl CryptoKeyMessageDestination for FulfillmentQuerySink {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_fulfillments(fulfillment::BrowseFulfillmentsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    order_by: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.KeyRotationApprovalService`.
struct KeyRotationApprovalSink {
    client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyMessageDestination for KeyRotationApprovalSink {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::EscalateKeyRotationRequest {
                    details: Some(keymanagement::escalate_key_rotation_request::Details {
                        comment: record.message,
                        approver_email: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.escalate_key_rotation(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_crypto_key_message(
    kind: &str,
    record: CryptoKeyMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(CryptoKeyMessageRecord) -> Delivery>> = BTreeMap::new();
    let media_asset_channel = channel.clone();
    actions.insert(
        "media-asset",
        Box::new(move |record| {
            MediaAssetSink {
                client: media::media_asset_service_client::MediaAssetServiceClient::new(media_asset_channel.clone()),
            }
            .write(record)
        }),
    );
    let feature_flag_validation_channel = channel.clone();
    actions.insert(
        "feature-flag-validation",
        Box::new(move |record| {
            FeatureFlagValidationSink {
                client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient::new(
                    feature_flag_validation_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let fulfillment_query_channel = channel.clone();
    actions.insert(
        "fulfillment-query",
        Box::new(move |record| {
            FulfillmentQuerySink {
                client: fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient::new(
                    fulfillment_query_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let key_rotation_approval_channel = channel.clone();
    let key_rotation_approval_context = context.clone();
    actions.insert(
        "key-rotation-approval",
        Box::new(move |record| {
            KeyRotationApprovalSink {
                client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient::new(
                    key_rotation_approval_channel.clone(),
                ),
                context: key_rotation_approval_context.clone(),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
