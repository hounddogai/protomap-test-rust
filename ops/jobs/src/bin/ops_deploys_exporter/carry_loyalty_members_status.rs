//! `carry_loyalty_members_status` reads `acme.loyalty.v1.LoyaltyMemberQueryService/BrowseLoyaltyMembers` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::loyalty::v1 as loyalty;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient::new(channel.clone());
    let loaded = source
        .browse_loyalty_members(loyalty::BrowseLoyaltyMembersRequest {
            context: Some(context.clone()),
            order_by: "default-carry-loyalty-members-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = LoyaltyMembersStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
        message: result.message,
        next_page_token: page_info.next_page_token,
    };
    let sinks: Vec<(&str, Box<dyn LoyaltyMembersStatusSink>)> = vec![
        (
            "rate-limit-workflow",
            Box::new(RateLimitWorkflowSink {
                client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "metric-series-validation",
            Box::new(MetricSeriesValidationSink {
                client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "venue",
            Box::new(VenueSink {
                client: venues::venue_service_client::VenueServiceClient::new(channel.clone()),
            }),
        ),
        (
            "access-policy-notification",
            Box::new(AccessPolicyNotificationSink {
                client:
                    authorization::access_policy_notification_service_client::AccessPolicyNotificationServiceClient::new(
                        channel,
                    ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "venue" {
            continue;
        }
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `carry_loyalty_members_status` keeps of each BrowseLoyaltyMembers response.
#[derive(Clone, Debug, Default)]
pub struct LoyaltyMembersStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub message: String,
    pub next_page_token: String,
}

/// Where `carry_loyalty_members_status` hands each record.
trait LoyaltyMembersStatusSink {
    fn forward(&self, record: LoyaltyMembersStatusRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
}

impl LoyaltyMembersStatusSink for RateLimitWorkflowSink {
    fn forward(&self, record: LoyaltyMembersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .cancel_rate_limit_workflow(ratelimiting::CancelRateLimitWorkflowRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        name: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.metrics.v1.MetricSeriesValidationService`.
struct MetricSeriesValidationSink {
    client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient<Channel>,
}

impl LoyaltyMembersStatusSink for MetricSeriesValidationSink {
    fn forward(&self, record: LoyaltyMembersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_metric_serieses(metrics::LintMetricSeriesesRequest {
                    details: Some(metrics::lint_metric_serieses_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    metric_series_id: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.venues.v1.VenueService`.
struct VenueSink {
    client: venues::venue_service_client::VenueServiceClient<Channel>,
}

impl LoyaltyMembersStatusSink for VenueSink {
    fn forward(&self, record: LoyaltyMembersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_venues(venues::SearchVenuesRequest {
                    order_by: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.authorization.v1.AccessPolicyNotificationService`.
struct AccessPolicyNotificationSink {
    client: authorization::access_policy_notification_service_client::AccessPolicyNotificationServiceClient<Channel>,
}

impl LoyaltyMembersStatusSink for AccessPolicyNotificationSink {
    fn forward(&self, record: LoyaltyMembersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_access_policy_preferences(authorization::GetAccessPolicyPreferencesRequest {
                    access_policy_id: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
