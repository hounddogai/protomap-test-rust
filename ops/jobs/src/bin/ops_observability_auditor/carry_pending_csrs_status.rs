//! `carry_pending_csrs_status` reads `acme.certificates.v1.CsrApprovalService/ListPendingCsrs` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::csr_approval_service_client::CsrApprovalServiceClient::new(channel.clone());
    let reply = source
        .list_pending_csrs(certificates::ListPendingCsrsRequest {
            context: Some(context.clone()),
            order_by: "default-carry-pending-csrs-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = PendingCsrsStatusRecord {
        status: result.status,
        next_page_token: page_info.next_page_token,
        correlation_id: result.correlation_id,
        message: result.message,
    };
    dispatch_pending_csrs_status("rate-limit-query", record, channel, &context).await
}

/// What `carry_pending_csrs_status` keeps of each ListPendingCsrs response.
#[derive(Clone, Debug, Default)]
pub struct PendingCsrsStatusRecord {
    pub status: String,
    pub next_page_token: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `carry_pending_csrs_status` hands each record.
trait PendingCsrsStatusDestination {
    fn push(&self, record: PendingCsrsStatusRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FeatureFlagNotificationService`.
struct FeatureFlagNotificationSink {
    client: featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient<Channel>,
}

impl PendingCsrsStatusDestination for FeatureFlagNotificationSink {
    fn push(&self, record: PendingCsrsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .send_feature_flag_test_notice(featureflags::SendFeatureFlagTestNoticeRequest {
                    details: Some(featureflags::send_feature_flag_test_notice_request::Details {
                        channel: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.RateLimitQueryService`.
struct RateLimitQuerySink {
    client: ratelimiting::rate_limit_query_service_client::RateLimitQueryServiceClient<Channel>,
}

impl PendingCsrsStatusDestination for RateLimitQuerySink {
    fn push(&self, record: PendingCsrsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_rate_limit(ratelimiting::LookupRateLimitRequest {
                    rate_limit_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct PendingCsrsStatusLogOnly;

impl PendingCsrsStatusDestination for PendingCsrsStatusLogOnly {
    fn push(&self, record: PendingCsrsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_pending_csrs_status(
    kind: &str,
    record: PendingCsrsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let feature_flag_notification = FeatureFlagNotificationSink {
        client: featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient::new(
            channel.clone(),
        ),
    };
    let rate_limit_query = RateLimitQuerySink {
        client: ratelimiting::rate_limit_query_service_client::RateLimitQueryServiceClient::new(channel),
    };
    let write: &dyn Fn(PendingCsrsStatusRecord) -> Delivery = match kind {
        "feature-flag-notification" => &|record| feature_flag_notification.push(record),
        "rate-limit-query" => &|record| rate_limit_query.push(record),
        _ => &|record| PendingCsrsStatusLogOnly.push(record),
    };
    write(record).await
}
