//! `copy_crypto_keys_token` reads `acme.keymanagement.v1.CryptoKeyQueryService/ResolveCryptoKeys` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient::new(channel.clone());
    let found = source
        .resolve_crypto_keys(keymanagement::ResolveCryptoKeysRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = found.page_info.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = CryptoKeysTokenRecord {
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        status: acme_kit::text::normalize(&result.status),
    };
    let sinks: Vec<(&str, Box<dyn CryptoKeysTokenTarget>)> = vec![
        (
            "key-rotation-approval",
            Box::new(KeyRotationApprovalSink {
                client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "seller",
            Box::new(SellerSink {
                client: sellers::seller_service_client::SellerServiceClient::new(channel.clone()),
            }),
        ),
        (
            "flag-rollout-calculation",
            Box::new(FlagRolloutCalculationSink {
                client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
                    channel,
                ),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "flag-rollout-calculation" {
            continue;
        }
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `copy_crypto_keys_token` keeps of each ResolveCryptoKeys response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeysTokenRecord {
    pub next_page_token: String,
    pub status: String,
}

/// Where `copy_crypto_keys_token` hands each record.
trait CryptoKeysTokenTarget {
    fn write(&self, record: CryptoKeysTokenRecord) -> Delivery;
}

/// Sends records to `acme.keymanagement.v1.KeyRotationApprovalService`.
struct KeyRotationApprovalSink {
    client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeysTokenTarget for KeyRotationApprovalSink {
    fn write(&self, record: CryptoKeysTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::CommentKeyRotationApprovalRequest {
                    key_rotation_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.comment_key_rotation_approval(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.sellers.v1.SellerService`.
struct SellerSink {
    client: sellers::seller_service_client::SellerServiceClient<Channel>,
}

impl CryptoKeysTokenTarget for SellerSink {
    fn write(&self, record: CryptoKeysTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_seller(sellers::CloneSellerRequest {
                    request_id: record.status,
                    seller: Some(sellers::Seller {
                        seller_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagRolloutCalculationService`.
struct FlagRolloutCalculationSink {
    client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeysTokenTarget for FlagRolloutCalculationSink {
    fn write(&self, record: CryptoKeysTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::GetFlagRolloutRatesRequest {
                    flag_rollout_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_flag_rollout_rates(request).await?;
            Ok(())
        })
    }
}
