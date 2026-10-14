//! `forward_media_asset_name` reads `acme.media.v1.MediaAssetService/CreateMediaAsset` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::media::v1 as media;
use acme_proto::acme::reviews::v1 as reviews;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = media::media_asset_service_client::MediaAssetServiceClient::new(channel.clone());
    let fetched = source
        .create_media_asset(media::CreateMediaAssetRequest {
            context: Some(context.clone()),
            request_id: "nightly-forward-media-asset-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let media_asset = fetched.media_asset.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = MediaAssetNameRecord {
        name: media_asset.name,
        message: acme_kit::text::normalize(&result.message),
        code: media_asset.code,
    };
    let sinks: Vec<Box<dyn MediaAssetNameSink>> = vec![
        Box::new(FlagRolloutNotificationSink {
            client: featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(ProductReviewSink {
            client: reviews::product_review_service_client::ProductReviewServiceClient::new(channel),
        }),
    ];
    for sink in &sinks {
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `forward_media_asset_name` keeps of each CreateMediaAsset response.
#[derive(Clone, Debug, Default)]
pub struct MediaAssetNameRecord {
    pub name: String,
    pub message: String,
    pub code: String,
}

/// Where `forward_media_asset_name` hands each record.
trait MediaAssetNameSink {
    fn apply(&self, record: MediaAssetNameRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FlagRolloutNotificationService`.
struct FlagRolloutNotificationSink {
    client: featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl MediaAssetNameSink for FlagRolloutNotificationSink {
    fn apply(&self, record: MediaAssetNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::SendFlagRolloutTestNoticeRequest {
                    details: Some(featureflags::send_flag_rollout_test_notice_request::Details {
                        channel: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.send_flag_rollout_test_notice(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.reviews.v1.ProductReviewService`.
struct ProductReviewSink {
    client: reviews::product_review_service_client::ProductReviewServiceClient<Channel>,
}

impl MediaAssetNameSink for ProductReviewSink {
    fn apply(&self, record: MediaAssetNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_product_review(reviews::CreateProductReviewRequest {
                    request_id: record.message,
                    product_review: Some(reviews::ProductReview {
                        number: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
