//! `backfill_crypto_key_workflow_id` reads
//! `acme.keymanagement.v1.CryptoKeyWorkflowService/CancelCryptoKeyWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::reviews::v1 as reviews;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(channel.clone());
    let fetched = source
        .cancel_crypto_key_workflow(keymanagement::CancelCryptoKeyWorkflowRequest {
            context: Some(context.clone()),
            reason: "primary-backfill-crypto-key-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = CryptoKeyWorkflowIdRecord {
        correlation_id: result.correlation_id,
        message: acme_kit::text::normalize(&outcome.message),
    };
    deliver_crypto_key_workflow_id("labeling-task", record, channel, &context).await
}

/// What `backfill_crypto_key_workflow_id` keeps of each CancelCryptoKeyWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyWorkflowIdRecord {
    pub correlation_id: String,
    pub message: String,
}

/// Where `backfill_crypto_key_workflow_id` hands each record.
trait CryptoKeyWorkflowIdSink {
    fn send(&self, record: CryptoKeyWorkflowIdRecord) -> Delivery;
}

/// Writes records to `acme.reviews.v1.ProductReviewService`.
struct ProductReviewSink {
    client: reviews::product_review_service_client::ProductReviewServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyWorkflowIdSink for ProductReviewSink {
    fn send(&self, record: CryptoKeyWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(reviews::SearchProductReviewsRequest {
                    filter: Some(reviews::search_product_reviews_request::Filter {
                        region: record.message,
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_product_reviews(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketSyncService`.
struct QuotaBucketSyncSink {
    client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient<Channel>,
}

impl CryptoKeyWorkflowIdSink for QuotaBucketSyncSink {
    fn send(&self, record: CryptoKeyWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pull_quota_buckets(ratelimiting::PullQuotaBucketsRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        quota_bucket_id: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(ratelimiting::pull_quota_buckets_request::Details {
                        checksum: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.labeling.v1.LabelingTaskService`.
struct LabelingTaskSink {
    client: labeling::labeling_task_service_client::LabelingTaskServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyWorkflowIdSink for LabelingTaskSink {
    fn send(&self, record: CryptoKeyWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(labeling::DeleteLabelingTaskRequest {
                    etag: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.delete_labeling_task(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.identity.v1.UserService`.
struct UserSink {
    client: identity::user_service_client::UserServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyWorkflowIdSink for UserSink {
    fn send(&self, record: CryptoKeyWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(identity::ListUsersRequest {
                    filter: Some(identity::list_users_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_users(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct CryptoKeyWorkflowIdLogOnly;

impl CryptoKeyWorkflowIdSink for CryptoKeyWorkflowIdLogOnly {
    fn send(&self, record: CryptoKeyWorkflowIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_crypto_key_workflow_id(
    kind: &str,
    record: CryptoKeyWorkflowIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn CryptoKeyWorkflowIdSink> = match kind {
        "product-review" => Box::new(ProductReviewSink {
            client: reviews::product_review_service_client::ProductReviewServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "quota-bucket-sync" => Box::new(QuotaBucketSyncSink {
            client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient::new(channel.clone()),
        }),
        "labeling-task" => Box::new(LabelingTaskSink {
            client: labeling::labeling_task_service_client::LabelingTaskServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "user" => Box::new(UserSink {
            client: identity::user_service_client::UserServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(CryptoKeyWorkflowIdLogOnly),
    };
    sink.send(record).await
}
