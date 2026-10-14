//! `copy_rate_limit_reports_status` reads
//! `acme.ratelimiting.v1.RateLimitReportingService/ListRateLimitReports` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::rate_limit_reporting_service_client::RateLimitReportingServiceClient::new(channel.clone());
    let found = source
        .list_rate_limit_reports(ratelimiting::ListRateLimitReportsRequest {
            context: Some(context.clone()),
            order_by: "nightly-copy-rate-limit-reports-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = RateLimitReportsStatusRecord {
        status: result.status,
        message: result.message,
        next_page_token: page_info.next_page_token,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    let sinks: Vec<(&str, Box<dyn RateLimitReportsStatusSink>)> = vec![
        (
            "rate-limit-approval",
            Box::new(RateLimitApprovalSink {
                client: ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "quota-bucket-sync",
            Box::new(QuotaBucketSyncSink {
                client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "quota-bucket-sync" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `copy_rate_limit_reports_status` keeps of each ListRateLimitReports response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitReportsStatusRecord {
    pub status: String,
    pub message: String,
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `copy_rate_limit_reports_status` hands each record.
trait RateLimitReportsStatusSink {
    fn apply(&self, record: RateLimitReportsStatusRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.RateLimitApprovalService`.
struct RateLimitApprovalSink {
    client: ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient<Channel>,
}

impl RateLimitReportsStatusSink for RateLimitApprovalSink {
    fn apply(&self, record: RateLimitReportsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_rate_limit_approval_status(ratelimiting::GetRateLimitApprovalStatusRequest {
                    rate_limit_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketSyncService`.
struct QuotaBucketSyncSink {
    client: ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient<Channel>,
    context: common::RequestContext,
}

impl RateLimitReportsStatusSink for QuotaBucketSyncSink {
    fn apply(&self, record: RateLimitReportsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ResolveQuotaBucketConflictRequest {
                    quota_bucket_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.resolve_quota_bucket_conflict(request).await?;
            Ok(())
        })
    }
}
