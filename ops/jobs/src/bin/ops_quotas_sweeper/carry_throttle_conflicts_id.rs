//! `carry_throttle_conflicts_id` reads `acme.ratelimiting.v1.ThrottleSyncService/ListThrottleConflicts` and
//! hands what it keeps to `acme.keymanagement.v1.CryptoKeyLifecycleService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::throttle_sync_service_client::ThrottleSyncServiceClient::new(channel.clone());
    let loaded = source
        .list_throttle_conflicts(ratelimiting::ListThrottleConflictsRequest {
            context: Some(context.clone()),
            order_by: "nightly-carry-throttle-conflicts-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = ThrottleConflictsIdRecord {
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
        message: result.message,
    };
    CryptoKeyLifecycleSink {
        client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(channel),
        context: context.clone(),
    }
    .write(record)
    .await
}

/// What `carry_throttle_conflicts_id` keeps of each ListThrottleConflicts response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleConflictsIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
    pub message: String,
}

/// Where `carry_throttle_conflicts_id` hands each record.
trait ThrottleConflictsIdSink {
    fn write(&self, record: ThrottleConflictsIdRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyLifecycleService`.
struct CryptoKeyLifecycleSink {
    client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleConflictsIdSink for CryptoKeyLifecycleSink {
    fn write(&self, record: ThrottleConflictsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ScheduleCryptoKeyExpiryRequest {
                    crypto_key_id: record.next_page_token,
                    reason: record.message,
                    crypto_key: Some(keymanagement::CryptoKey {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.schedule_crypto_key_expiry(request).await?;
            Ok(())
        })
    }
}
