//! `settle_permission_check_test_notice_id` reads
//! `acme.authorization.v1.PermissionCheckNotificationService/SendPermissionCheckTestNotice` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::subscriptions::v1 as subscriptions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(
            channel.clone(),
        );
    let reply = source
        .send_permission_check_test_notice(authorization::SendPermissionCheckTestNoticeRequest {
            context: Some(context.clone()),
            reason: "current-settle-permission-check-test-notice-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let permission_check = reply.permission_check.unwrap_or_default();
    let record = PermissionCheckTestNoticeIdRecord {
        reference_id: outcome.reference_id,
        correlation_id: result.correlation_id,
        kind: permission_check.kind,
    };
    dispatch_permission_check_test_notice_id("feature-flag-export", record, channel, &context).await
}

/// What `settle_permission_check_test_notice_id` keeps of each SendPermissionCheckTestNotice response.
#[derive(Clone, Debug, Default)]
pub struct PermissionCheckTestNoticeIdRecord {
    pub reference_id: String,
    pub correlation_id: String,
    pub kind: String,
}

/// Where `settle_permission_check_test_notice_id` hands each record.
trait PermissionCheckTestNoticeIdWriter {
    fn push(&self, record: PermissionCheckTestNoticeIdRecord) -> Delivery;
}

/// Sends records to `acme.featureflags.v1.FeatureFlagExportService`.
struct FeatureFlagExportSink {
    client: featureflags::feature_flag_export_service_client::FeatureFlagExportServiceClient<Channel>,
}

impl PermissionCheckTestNoticeIdWriter for FeatureFlagExportSink {
    fn push(&self, record: PermissionCheckTestNoticeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_feature_flag_export(featureflags::GetFeatureFlagExportRequest {
                    feature_flag_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.subscriptions.v1.SubscriptionService`.
struct SubscriptionSink {
    client: subscriptions::subscription_service_client::SubscriptionServiceClient<Channel>,
}

impl PermissionCheckTestNoticeIdWriter for SubscriptionSink {
    fn push(&self, record: PermissionCheckTestNoticeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_subscription(subscriptions::UpdateSubscriptionRequest {
                    subscription: Some(subscriptions::Subscription {
                        subscription_id: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct PermissionCheckTestNoticeIdLogOnly;

impl PermissionCheckTestNoticeIdWriter for PermissionCheckTestNoticeIdLogOnly {
    fn push(&self, record: PermissionCheckTestNoticeIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.reference_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_permission_check_test_notice_id(
    kind: &str,
    record: PermissionCheckTestNoticeIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let feature_flag_export = FeatureFlagExportSink {
        client: featureflags::feature_flag_export_service_client::FeatureFlagExportServiceClient::new(channel.clone()),
    };
    let subscription = SubscriptionSink {
        client: subscriptions::subscription_service_client::SubscriptionServiceClient::new(channel),
    };
    let write: &dyn Fn(PermissionCheckTestNoticeIdRecord) -> Delivery = match kind {
        "feature-flag-export" => &|record| feature_flag_export.push(record),
        "subscription" => &|record| subscription.push(record),
        _ => &|record| PermissionCheckTestNoticeIdLogOnly.push(record),
    };
    write(record).await
}
