//! `route_video_visit_rates_id` reads `acme.telehealth.v1.VideoVisitCalculationService/GetVideoVisitRates`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient::new(channel.clone());
    let loaded = source
        .get_video_visit_rates(telehealth::GetVideoVisitRatesRequest {
            context: Some(context.clone()),
            video_visit_id: "current-route-video-visit-rates-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let video_visit = loaded.video_visit.unwrap_or_default();
    let record = VideoVisitRatesIdRecord {
        correlation_id: result.correlation_id,
        title: video_visit.title,
        status: result.status,
    };
    act_on_video_visit_rates_id("crypto-key-validation", record, channel, context).await
}

/// What `route_video_visit_rates_id` keeps of each GetVideoVisitRates response.
#[derive(Clone, Debug, Default)]
pub struct VideoVisitRatesIdRecord {
    pub correlation_id: String,
    pub title: String,
    pub status: String,
}

/// Where `route_video_visit_rates_id` hands each record.
trait VideoVisitRatesIdSink {
    fn deliver(&self, record: VideoVisitRatesIdRecord) -> Delivery;
}

/// Writes records to `acme.authorization.v1.AccessPolicyValidationService`.
struct AccessPolicyValidationSink {
    client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl VideoVisitRatesIdSink for AccessPolicyValidationSink {
    fn deliver(&self, record: VideoVisitRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::GetAccessPolicyRulesRequest {
                    access_policy_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_access_policy_rules(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketSyncService`.
struct QuotaBucketSyncSink {
    client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient<Channel>,
}

impl VideoVisitRatesIdSink for QuotaBucketSyncSink {
    fn deliver(&self, record: VideoVisitRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .push_quota_bucket(ratelimiting::PushQuotaBucketRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        currency_code: record.title,
                        ..Default::default()
                    }),
                    details: Some(ratelimiting::push_quota_bucket_request::Details {
                        cursor: record.status,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyValidationService`.
struct CryptoKeyValidationSink {
    client: keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient<Channel>,
}

impl VideoVisitRatesIdSink for CryptoKeyValidationSink {
    fn deliver(&self, record: VideoVisitRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .validate_crypto_key(keymanagement::ValidateCryptoKeyRequest {
                    reason: record.title,
                    crypto_key_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_video_visit_rates_id(
    kind: &str,
    record: VideoVisitRatesIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(VideoVisitRatesIdRecord) -> Delivery>> = BTreeMap::new();
    let access_policy_validation_channel = channel.clone();
    let access_policy_validation_context = context.clone();
    actions.insert(
        "access-policy-validation",
        Box::new(move |record| {
            AccessPolicyValidationSink {
                client:
                    authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
                        access_policy_validation_channel.clone(),
                    ),
                context: access_policy_validation_context.clone(),
            }
            .deliver(record)
        }),
    );
    let quota_bucket_sync_channel = channel.clone();
    actions.insert(
        "quota-bucket-sync",
        Box::new(move |record| {
            QuotaBucketSyncSink {
                client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient::new(
                    quota_bucket_sync_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    let crypto_key_validation_channel = channel.clone();
    actions.insert(
        "crypto-key-validation",
        Box::new(move |record| {
            CryptoKeyValidationSink {
                client: keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient::new(
                    crypto_key_validation_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
