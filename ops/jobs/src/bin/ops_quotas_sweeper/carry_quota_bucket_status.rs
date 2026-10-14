//! `carry_quota_bucket_status` reads `acme.ratelimiting.v1.QuotaBucketLifecycleService/RenewQuotaBucket` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::quota_bucket_lifecycle_service_client::QuotaBucketLifecycleServiceClient::new(channel.clone());
    let loaded = source
        .renew_quota_bucket(ratelimiting::RenewQuotaBucketRequest {
            context: Some(context.clone()),
            reason: "current-carry-quota-bucket-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let quota_bucket = loaded.quota_bucket.unwrap_or_default();
    let record = QuotaBucketStatusRecord {
        status: result.status,
        quota_bucket_id: acme_kit::text::normalize(&quota_bucket.quota_bucket_id),
    };
    let sinks: Vec<Box<dyn QuotaBucketStatusSink>> = vec![
        Box::new(FanSessionSink {
            client: fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone()),
        }),
        Box::new(YardSink {
            client: yards::yard_service_client::YardServiceClient::new(channel),
        }),
    ];
    sinks[1].forward(record).await
}

/// What `carry_quota_bucket_status` keeps of each RenewQuotaBucket response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketStatusRecord {
    pub status: String,
    pub quota_bucket_id: String,
}

/// Where `carry_quota_bucket_status` hands each record.
trait QuotaBucketStatusSink {
    fn forward(&self, record: QuotaBucketStatusRecord) -> Delivery;
}

/// Writes records to `acme.fanportal.v1.FanSessionService`.
struct FanSessionSink {
    client: fanportal::fan_session_service_client::FanSessionServiceClient<Channel>,
}

impl QuotaBucketStatusSink for FanSessionSink {
    fn forward(&self, record: QuotaBucketStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_fan_session(fanportal::DeleteFanSessionRequest {
                    etag: record.quota_bucket_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.yards.v1.YardService`.
struct YardSink {
    client: yards::yard_service_client::YardServiceClient<Channel>,
}

impl QuotaBucketStatusSink for YardSink {
    fn forward(&self, record: QuotaBucketStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_yard(yards::CloneYardRequest {
                    yard: Some(yards::Yard {
                        name: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
