//! `settle_quota_buckets_message` reads `acme.ratelimiting.v1.QuotaBucketQueryService/FindQuotaBuckets` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::messaging::v1 as messaging;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::quota_bucket_query_service_client::QuotaBucketQueryServiceClient::new(channel.clone());
    let found = source
        .find_quota_buckets(ratelimiting::FindQuotaBucketsRequest {
            context: Some(context.clone()),
            order_by: "default-settle-quota-buckets-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let record = QuotaBucketsMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
        status: result.status,
    };
    dispatch_quota_buckets_message("conversation-calculation", record, channel, &context).await
}

/// What `settle_quota_buckets_message` keeps of each FindQuotaBuckets response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketsMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `settle_quota_buckets_message` hands each record.
trait QuotaBucketsMessageWriter {
    fn forward(&self, record: QuotaBucketsMessageRecord) -> Delivery;
}

/// Hands records to `acme.fleet.v1.VehicleService`.
struct VehicleSink {
    client: fleet::vehicle_service_client::VehicleServiceClient<Channel>,
}

impl QuotaBucketsMessageWriter for VehicleSink {
    fn forward(&self, record: QuotaBucketsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_vehicle(fleet::CreateVehicleRequest {
                    request_id: record.correlation_id,
                    vehicle: Some(fleet::Vehicle {
                        code: record.status,
                        vehicle_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FeatureFlagValidationService`.
struct FeatureFlagValidationSink {
    client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient<Channel>,
}

impl QuotaBucketsMessageWriter for FeatureFlagValidationSink {
    fn forward(&self, record: QuotaBucketsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_feature_flags(featureflags::LintFeatureFlagsRequest {
                    details: Some(featureflags::lint_feature_flags_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    feature_flag: Some(featureflags::FeatureFlag {
                        description: record.correlation_id,
                        ..Default::default()
                    }),
                    feature_flag_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketSyncService`.
struct QuotaBucketSyncSink {
    client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient<Channel>,
}

impl QuotaBucketsMessageWriter for QuotaBucketSyncSink {
    fn forward(&self, record: QuotaBucketsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_quota_bucket_sync_state(ratelimiting::GetQuotaBucketSyncStateRequest {
                    quota_bucket_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.messaging.v1.ConversationCalculationService`.
struct ConversationCalculationSink {
    client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl QuotaBucketsMessageWriter for ConversationCalculationSink {
    fn forward(&self, record: QuotaBucketsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(messaging::RecalculateConversationsRequest {
                    conversation: Some(messaging::Conversation {
                        conversation_id: record.status,
                        ..Default::default()
                    }),
                    details: Some(messaging::recalculate_conversations_request::Details {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.recalculate_conversations(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct QuotaBucketsMessageLogOnly;

impl QuotaBucketsMessageWriter for QuotaBucketsMessageLogOnly {
    fn forward(&self, record: QuotaBucketsMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_quota_buckets_message(
    kind: &str,
    record: QuotaBucketsMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let vehicle = VehicleSink {
        client: fleet::vehicle_service_client::VehicleServiceClient::new(channel.clone()),
    };
    let feature_flag_validation = FeatureFlagValidationSink {
        client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient::new(
            channel.clone(),
        ),
    };
    let quota_bucket_sync = QuotaBucketSyncSink {
        client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient::new(channel.clone()),
    };
    let conversation_calculation = ConversationCalculationSink {
        client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(QuotaBucketsMessageRecord) -> Delivery = match kind {
        "vehicle" => &|record| vehicle.forward(record),
        "feature-flag-validation" => &|record| feature_flag_validation.forward(record),
        "quota-bucket-sync" => &|record| quota_bucket_sync.forward(record),
        "conversation-calculation" => &|record| conversation_calculation.forward(record),
        _ => &|record| QuotaBucketsMessageLogOnly.forward(record),
    };
    write(record).await
}
