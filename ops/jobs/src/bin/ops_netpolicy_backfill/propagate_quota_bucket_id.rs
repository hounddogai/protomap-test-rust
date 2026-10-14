//! `propagate_quota_bucket_id` reads `acme.ratelimiting.v1.QuotaBucketCalculationService/QuoteQuotaBucket`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient::new(
        channel.clone(),
    );
    let reply = source
        .quote_quota_bucket(ratelimiting::QuoteQuotaBucketRequest {
            context: Some(context.clone()),
            quota_bucket_id: "live-propagate-quota-bucket-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let quota_bucket = reply.quota_bucket.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = QuotaBucketIdRecord {
        quota_bucket_id: quota_bucket.quota_bucket_id,
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn QuotaBucketIdTarget>> = BTreeMap::new();
    sinks.insert(
        "environment-sync",
        Box::new(EnvironmentSyncSink {
            client: configuration::environment_sync_service_client::EnvironmentSyncServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "chat-room-query",
        Box::new(ChatRoomQuerySink {
            client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "certificate-validation",
        Box::new(CertificateValidationSink {
            client: certificates::certificate_validation_service_client::CertificateValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "seller-query",
        Box::new(SellerQuerySink {
            client: sellers::seller_query_service_client::SellerQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("seller-query") {
        Some(sink) => sink.forward(record).await,
        None => anyhow::bail!("no sink registered for seller-query"),
    }
}

/// What `propagate_quota_bucket_id` keeps of each QuoteQuotaBucket response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketIdRecord {
    pub quota_bucket_id: String,
    pub status: String,
    pub message: String,
}

/// Where `propagate_quota_bucket_id` hands each record.
trait QuotaBucketIdTarget {
    fn forward(&self, record: QuotaBucketIdRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.EnvironmentSyncService`.
struct EnvironmentSyncSink {
    client: configuration::environment_sync_service_client::EnvironmentSyncServiceClient<Channel>,
}

impl QuotaBucketIdTarget for EnvironmentSyncSink {
    fn forward(&self, record: QuotaBucketIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pull_environments(configuration::PullEnvironmentsRequest {
                    details: Some(configuration::pull_environments_request::Details {
                        checksum: record.message,
                        resolution: record.quota_bucket_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.chat.v1.ChatRoomQueryService`.
struct ChatRoomQuerySink {
    client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient<Channel>,
}

impl QuotaBucketIdTarget for ChatRoomQuerySink {
    fn forward(&self, record: QuotaBucketIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_chat_rooms(chat::SummarizeChatRoomsRequest {
                    details: Some(chat::summarize_chat_rooms_request::Details {
                        format: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.certificates.v1.CertificateValidationService`.
struct CertificateValidationSink {
    client: certificates::certificate_validation_service_client::CertificateValidationServiceClient<Channel>,
}

impl QuotaBucketIdTarget for CertificateValidationSink {
    fn forward(&self, record: QuotaBucketIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_certificate(certificates::ScoreCertificateRequest {
                    details: Some(certificates::score_certificate_request::Details {
                        currency_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.sellers.v1.SellerQueryService`.
struct SellerQuerySink {
    client: sellers::seller_query_service_client::SellerQueryServiceClient<Channel>,
}

impl QuotaBucketIdTarget for SellerQuerySink {
    fn forward(&self, record: QuotaBucketIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_sellers(sellers::BrowseSellersRequest {
                    order_by: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
