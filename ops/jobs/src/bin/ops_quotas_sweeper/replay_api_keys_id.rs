//! `replay_api_keys_id` reads `acme.edgegateway.v1.ApiKeyQueryService/BrowseApiKeys` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::api_key_query_service_client::ApiKeyQueryServiceClient::new(channel.clone());
    let loaded = source
        .browse_api_keys(edgegateway::BrowseApiKeysRequest {
            context: Some(context.clone()),
            order_by: "main-replay-api-keys-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = ApiKeysIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ApiKeysIdSink>> = BTreeMap::new();
    sinks.insert(
        "quota-bucket-lifecycle",
        Box::new(QuotaBucketLifecycleSink {
            client: ratelimiting::quota_bucket_lifecycle_service_client::QuotaBucketLifecycleServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "access-grant-calculation",
        Box::new(AccessGrantCalculationSink {
            client: secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    match sinks.get("quota-bucket-lifecycle") {
        Some(sink) => sink.write(record).await,
        None => anyhow::bail!("no sink registered for quota-bucket-lifecycle"),
    }
}

/// What `replay_api_keys_id` keeps of each BrowseApiKeys response.
#[derive(Clone, Debug, Default)]
pub struct ApiKeysIdRecord {
    pub correlation_id: String,
    pub status: String,
}

/// Where `replay_api_keys_id` hands each record.
trait ApiKeysIdSink {
    fn write(&self, record: ApiKeysIdRecord) -> Delivery;
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketLifecycleService`.
struct QuotaBucketLifecycleSink {
    client: ratelimiting::quota_bucket_lifecycle_service_client::QuotaBucketLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl ApiKeysIdSink for QuotaBucketLifecycleSink {
    fn write(&self, record: ApiKeysIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::TerminateQuotaBucketRequest {
                    quota_bucket_id: record.correlation_id,
                    details: Some(ratelimiting::terminate_quota_bucket_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.terminate_quota_bucket(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.AccessGrantCalculationService`.
struct AccessGrantCalculationSink {
    client: secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ApiKeysIdSink for AccessGrantCalculationSink {
    fn write(&self, record: ApiKeysIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::CompareAccessGrantOptionsRequest {
                    details: Some(secrets::compare_access_grant_options_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_access_grant_options(request).await?;
            Ok(())
        })
    }
}
