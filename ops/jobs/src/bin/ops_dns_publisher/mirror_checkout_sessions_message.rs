//! `mirror_checkout_sessions_message` reads
//! `acme.checkout.v1.CheckoutSessionQueryService/BrowseCheckoutSessions` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::checkout::v1 as checkout;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        checkout::checkout_session_query_service_client::CheckoutSessionQueryServiceClient::new(channel.clone());
    let reply = source
        .browse_checkout_sessions(checkout::BrowseCheckoutSessionsRequest {
            context: Some(context.clone()),
            order_by: "current-mirror-checkout-sessions-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = CheckoutSessionsMessageRecord {
        message: result.message,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn CheckoutSessionsMessageDestination>)> = vec![
        (
            "access-grant-approval",
            Box::new(AccessGrantApprovalSink {
                client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "video-visit-calculation",
            Box::new(VideoVisitCalculationSink {
                client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "fixed-asset-calculation",
            Box::new(FixedAssetCalculationSink {
                client: assets::fixed_asset_calculation_service_client::FixedAssetCalculationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "rate-limit-workflow",
            Box::new(RateLimitWorkflowSink {
                client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "fixed-asset-calculation" {
            continue;
        }
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `mirror_checkout_sessions_message` keeps of each BrowseCheckoutSessions response.
#[derive(Clone, Debug, Default)]
pub struct CheckoutSessionsMessageRecord {
    pub message: String,
    pub status: String,
}

/// Where `mirror_checkout_sessions_message` hands each record.
trait CheckoutSessionsMessageDestination {
    fn write(&self, record: CheckoutSessionsMessageRecord) -> Delivery;
}

/// Passes records to `acme.secrets.v1.AccessGrantApprovalService`.
struct AccessGrantApprovalSink {
    client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient<Channel>,
}

impl CheckoutSessionsMessageDestination for AccessGrantApprovalSink {
    fn write(&self, record: CheckoutSessionsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .comment_access_grant_approval(secrets::CommentAccessGrantApprovalRequest {
                    details: Some(secrets::comment_access_grant_approval_request::Details {
                        approver_email: record.message,
                        ..Default::default()
                    }),
                    access_grant: Some(secrets::AccessGrant {
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

/// Passes records to `acme.telehealth.v1.VideoVisitCalculationService`.
struct VideoVisitCalculationSink {
    client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient<Channel>,
}

impl CheckoutSessionsMessageDestination for VideoVisitCalculationSink {
    fn write(&self, record: CheckoutSessionsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_video_visit_rates(telehealth::GetVideoVisitRatesRequest {
                    video_visit_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.assets.v1.FixedAssetCalculationService`.
struct FixedAssetCalculationSink {
    client: assets::fixed_asset_calculation_service_client::FixedAssetCalculationServiceClient<Channel>,
}

impl CheckoutSessionsMessageDestination for FixedAssetCalculationSink {
    fn write(&self, record: CheckoutSessionsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_fixed_asset_rates(assets::GetFixedAssetRatesRequest {
                    fixed_asset_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl CheckoutSessionsMessageDestination for RateLimitWorkflowSink {
    fn write(&self, record: CheckoutSessionsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ListRateLimitWorkflowsRequest {
                    order_by: record.message,
                    filter: Some(ratelimiting::list_rate_limit_workflows_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_rate_limit_workflows(request).await?;
            Ok(())
        })
    }
}
