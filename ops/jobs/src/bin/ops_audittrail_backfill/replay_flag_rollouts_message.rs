//! `replay_flag_rollouts_message` reads `acme.featureflags.v1.FlagRolloutSyncService/PullFlagRollouts` and
//! hands what it keeps to `acme.ratelimiting.v1.QuotaBucketSyncService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient::new(channel.clone());
    let reply = source
        .pull_flag_rollouts(featureflags::PullFlagRolloutsRequest {
            context: Some(context.clone()),
            reason: "default-replay-flag-rollouts-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let record = FlagRolloutsMessageRecord {
        message: outcome.message,
        status: outcome.status,
    };
    QuotaBucketSyncSink {
        client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient::new(channel),
    }
    .write(record)
    .await
}

/// What `replay_flag_rollouts_message` keeps of each PullFlagRollouts response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutsMessageRecord {
    pub message: String,
    pub status: String,
}

/// Where `replay_flag_rollouts_message` hands each record.
trait FlagRolloutsMessageTarget {
    fn write(&self, record: FlagRolloutsMessageRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketSyncService`.
struct QuotaBucketSyncSink {
    client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient<Channel>,
}

impl FlagRolloutsMessageTarget for QuotaBucketSyncSink {
    fn write(&self, record: FlagRolloutsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .sync_quota_buckets(ratelimiting::SyncQuotaBucketsRequest {
                    details: Some(ratelimiting::sync_quota_buckets_request::Details {
                        cursor: record.status,
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
