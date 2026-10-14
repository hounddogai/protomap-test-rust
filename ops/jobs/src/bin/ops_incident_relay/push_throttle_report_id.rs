//! `push_throttle_report_id` reads `acme.ratelimiting.v1.ThrottleReportingService/DeleteThrottleReport` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient::new(channel.clone());
    let found = source
        .delete_throttle_report(ratelimiting::DeleteThrottleReportRequest {
            context: Some(context.clone()),
            etag: "main-push-throttle-report-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let record = ThrottleReportIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: result.status,
        message: result.message,
    };
    let sinks: Vec<Box<dyn ThrottleReportIdSink>> = vec![
        Box::new(CryptoKeyCalculationSink { client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(channel.clone()) }),
        Box::new(QuotaBucketReportingSink { client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient::new(channel.clone()) }),
        Box::new(PermissionCheckNotificationSink { client: authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(channel.clone()) }),
        Box::new(FeatureFlagQuerySink { client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(channel) }),
    ];
    sinks[1].deliver(record).await
}

/// What `push_throttle_report_id` keeps of each DeleteThrottleReport response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleReportIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub message: String,
}

/// Where `push_throttle_report_id` hands each record.
trait ThrottleReportIdSink {
    fn deliver(&self, record: ThrottleReportIdRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
}

impl ThrottleReportIdSink for CryptoKeyCalculationSink {
    fn deliver(&self, record: ThrottleReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_crypto_key_rates(keymanagement::GetCryptoKeyRatesRequest {
                    crypto_key_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketReportingService`.
struct QuotaBucketReportingSink {
    client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient<Channel>,
}

impl ThrottleReportIdSink for QuotaBucketReportingSink {
    fn deliver(&self, record: ThrottleReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .schedule_quota_bucket_report(ratelimiting::ScheduleQuotaBucketReportRequest {
                    reason: record.message,
                    details: Some(ratelimiting::schedule_quota_bucket_report_request::Details {
                        format: record.correlation_id,
                        cron: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.authorization.v1.PermissionCheckNotificationService`.
struct PermissionCheckNotificationSink {
    client:
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient<Channel>,
}

impl ThrottleReportIdSink for PermissionCheckNotificationSink {
    fn deliver(&self, record: ThrottleReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_permission_check_subscriptions(authorization::ListPermissionCheckSubscriptionsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FeatureFlagQueryService`.
struct FeatureFlagQuerySink {
    client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient<Channel>,
}

impl ThrottleReportIdSink for FeatureFlagQuerySink {
    fn deliver(&self, record: ThrottleReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_feature_flag(featureflags::PeekFeatureFlagRequest {
                    feature_flag_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
