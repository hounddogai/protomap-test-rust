//! `replay_flag_evaluation_message` reads
//! `acme.featureflags.v1.FlagEvaluationValidationService/ValidateFlagEvaluation` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::media::v1 as media;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient::new(
            channel.clone(),
        );
    let loaded = source
        .validate_flag_evaluation(featureflags::ValidateFlagEvaluationRequest {
            context: Some(context.clone()),
            reason: "default-replay-flag-evaluation-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = FlagEvaluationMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        outcome_message: outcome.message,
    };
    let sinks: Vec<Box<dyn FlagEvaluationMessageTarget>> = vec![
        Box::new(FeatureFlagQuerySink {
            client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(QuotaBucketValidationSink {
            client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(CsrLifecycleSink {
            client: certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(MediaAssetSink {
            client: media::media_asset_service_client::MediaAssetServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[3].write(record).await
}

/// What `replay_flag_evaluation_message` keeps of each ValidateFlagEvaluation response.
#[derive(Clone, Debug, Default)]
pub struct FlagEvaluationMessageRecord {
    pub message: String,
    pub outcome_message: String,
}

/// Where `replay_flag_evaluation_message` hands each record.
trait FlagEvaluationMessageTarget {
    fn write(&self, record: FlagEvaluationMessageRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FeatureFlagQueryService`.
struct FeatureFlagQuerySink {
    client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationMessageTarget for FeatureFlagQuerySink {
    fn write(&self, record: FlagEvaluationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::PeekFeatureFlagRequest {
                    feature_flag_id: record.outcome_message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_feature_flag(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketValidationService`.
struct QuotaBucketValidationSink {
    client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient<Channel>,
}

impl FlagEvaluationMessageTarget for QuotaBucketValidationSink {
    fn write(&self, record: FlagEvaluationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_quota_bucket(ratelimiting::NormalizeQuotaBucketRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        quota_bucket_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CsrLifecycleService`.
struct CsrLifecycleSink {
    client: certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationMessageTarget for CsrLifecycleSink {
    fn write(&self, record: FlagEvaluationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::SuspendCsrRequest {
                    reason: record.message,
                    details: Some(certificates::suspend_csr_request::Details {
                        reason_code: record.outcome_message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suspend_csr(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.media.v1.MediaAssetService`.
struct MediaAssetSink {
    client: media::media_asset_service_client::MediaAssetServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationMessageTarget for MediaAssetSink {
    fn write(&self, record: FlagEvaluationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(media::SearchMediaAssetsRequest {
                    order_by: record.message,
                    filter: Some(media::search_media_assets_request::Filter {
                        owner_id: record.outcome_message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_media_assets(request).await?;
            Ok(())
        })
    }
}
