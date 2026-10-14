//! `forward_flag_rollout_version_status` reads
//! `acme.featureflags.v1.FlagRolloutHistoryService/GetFlagRolloutVersion` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::promotions::v1 as promotions;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient::new(channel.clone());
    let fetched = source
        .get_flag_rollout_version(featureflags::GetFlagRolloutVersionRequest {
            context: Some(context.clone()),
            flag_rollout_id: "nightly-forward-flag-rollout-version-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let flag_rollout = fetched.flag_rollout.unwrap_or_default();
    let record = FlagRolloutVersionStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        flag_rollout_id: flag_rollout.flag_rollout_id,
        message: result.message,
        title: acme_kit::text::normalize(&flag_rollout.title),
    };
    dispatch_flag_rollout_version_status("promotion", record, channel, &context).await
}

/// What `forward_flag_rollout_version_status` keeps of each GetFlagRolloutVersion response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutVersionStatusRecord {
    pub status: String,
    pub flag_rollout_id: String,
    pub message: String,
    pub title: String,
}

/// Where `forward_flag_rollout_version_status` hands each record.
trait FlagRolloutVersionStatusTarget {
    fn forward(&self, record: FlagRolloutVersionStatusRecord) -> Delivery;
}

/// Sends records to `acme.promotions.v1.PromotionService`.
struct PromotionSink {
    client: promotions::promotion_service_client::PromotionServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagRolloutVersionStatusTarget for PromotionSink {
    fn forward(&self, record: FlagRolloutVersionStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(promotions::CreatePromotionRequest {
                    promotion: Some(promotions::Promotion {
                        promotion_id: record.flag_rollout_id,
                        description: record.status,
                        name: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.create_promotion(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.QuotaBucketValidationService`.
struct QuotaBucketValidationSink {
    client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient<Channel>,
}

impl FlagRolloutVersionStatusTarget for QuotaBucketValidationSink {
    fn forward(&self, record: FlagRolloutVersionStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .preview_quota_bucket(ratelimiting::PreviewQuotaBucketRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        quota_bucket_id: record.message,
                        currency_code: record.flag_rollout_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.CryptoKeyQueryService`.
struct CryptoKeyQuerySink {
    client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient<Channel>,
}

impl FlagRolloutVersionStatusTarget for CryptoKeyQuerySink {
    fn forward(&self, record: FlagRolloutVersionStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_crypto_keys(keymanagement::BrowseCryptoKeysRequest {
                    filter: Some(keymanagement::browse_crypto_keys_request::Filter {
                        query: record.title,
                        ..Default::default()
                    }),
                    order_by: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigChangeWorkflowService`.
struct ConfigChangeWorkflowSink {
    client: configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagRolloutVersionStatusTarget for ConfigChangeWorkflowSink {
    fn forward(&self, record: FlagRolloutVersionStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::ResumeConfigChangeWorkflowRequest {
                    reason: record.message,
                    details: Some(configuration::resume_config_change_workflow_request::Details {
                        step: record.flag_rollout_id,
                        ..Default::default()
                    }),
                    config_change: Some(configuration::ConfigChange {
                        config_change_id: record.title,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.resume_config_change_workflow(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct FlagRolloutVersionStatusLogOnly;

impl FlagRolloutVersionStatusTarget for FlagRolloutVersionStatusLogOnly {
    fn forward(&self, record: FlagRolloutVersionStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_flag_rollout_version_status(
    kind: &str,
    record: FlagRolloutVersionStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let promotion = PromotionSink {
        client: promotions::promotion_service_client::PromotionServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let quota_bucket_validation = QuotaBucketValidationSink {
        client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(
            channel.clone(),
        ),
    };
    let crypto_key_query = CryptoKeyQuerySink {
        client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient::new(channel.clone()),
    };
    let config_change_workflow = ConfigChangeWorkflowSink {
        client: configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(FlagRolloutVersionStatusRecord) -> Delivery = match kind {
        "promotion" => &|record| promotion.forward(record),
        "quota-bucket-validation" => &|record| quota_bucket_validation.forward(record),
        "crypto-key-query" => &|record| crypto_key_query.forward(record),
        "config-change-workflow" => &|record| config_change_workflow.forward(record),
        _ => &|record| FlagRolloutVersionStatusLogOnly.forward(record),
    };
    write(record).await
}
