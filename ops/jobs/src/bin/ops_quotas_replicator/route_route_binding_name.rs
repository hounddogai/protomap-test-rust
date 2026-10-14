//! `route_route_binding_name` reads `acme.edgegateway.v1.RouteBindingGatewayService/RefreshRouteBinding` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::analytics::v1 as analytics;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient::new(channel.clone());
    let fetched = source
        .refresh_route_binding(edgegateway::RefreshRouteBindingRequest {
            context: Some(context.clone()),
            route_binding_id: "main-route-route-binding-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let route_binding = fetched.route_binding.unwrap_or_default();
    let record = RouteBindingNameRecord {
        name: acme_kit::text::normalize(&route_binding.name),
        route_binding_id: acme_kit::text::normalize(&route_binding.route_binding_id),
    };
    act_on_route_binding_name("analytics-event", record, channel, context).await
}

/// What `route_route_binding_name` keeps of each RefreshRouteBinding response.
#[derive(Clone, Debug, Default)]
pub struct RouteBindingNameRecord {
    pub name: String,
    pub route_binding_id: String,
}

/// Where `route_route_binding_name` hands each record.
trait RouteBindingNameSink {
    fn send(&self, record: RouteBindingNameRecord) -> Delivery;
}

/// Passes records to `acme.analytics.v1.AnalyticsEventService`.
struct AnalyticsEventSink {
    client: analytics::analytics_event_service_client::AnalyticsEventServiceClient<Channel>,
    context: common::RequestContext,
}

impl RouteBindingNameSink for AnalyticsEventSink {
    fn send(&self, record: RouteBindingNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(analytics::CreateAnalyticsEventRequest {
                    analytics_event: Some(analytics::AnalyticsEvent {
                        analytics_event_id: record.route_binding_id,
                        correlation_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.create_analytics_event(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketReportingService`.
struct QuotaBucketReportingSink {
    client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient<Channel>,
}

impl RouteBindingNameSink for QuotaBucketReportingSink {
    fn send(&self, record: RouteBindingNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .generate_quota_bucket_report(ratelimiting::GenerateQuotaBucketReportRequest {
                    details: Some(ratelimiting::generate_quota_bucket_report_request::Details {
                        cron: record.name,
                        ..Default::default()
                    }),
                    reason: record.route_binding_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_route_binding_name(
    kind: &str,
    record: RouteBindingNameRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(RouteBindingNameRecord) -> Delivery>> = BTreeMap::new();
    let analytics_event_channel = channel.clone();
    let analytics_event_context = context.clone();
    actions.insert(
        "analytics-event",
        Box::new(move |record| {
            AnalyticsEventSink {
                client: analytics::analytics_event_service_client::AnalyticsEventServiceClient::new(
                    analytics_event_channel.clone(),
                ),
                context: analytics_event_context.clone(),
            }
            .send(record)
        }),
    );
    let quota_bucket_reporting_channel = channel.clone();
    actions.insert(
        "quota-bucket-reporting",
        Box::new(move |record| {
            QuotaBucketReportingSink {
                client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient::new(
                    quota_bucket_reporting_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
