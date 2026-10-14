//! `export_fan_sessions_id` reads `acme.fanportal.v1.FanSessionQueryService/FindFanSessions` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fanportal::fan_session_query_service_client::FanSessionQueryServiceClient::new(channel.clone());
    let loaded = source
        .find_fan_sessions(fanportal::FindFanSessionsRequest {
            context: Some(context.clone()),
            order_by: "live-export-fan-sessions-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = FanSessionsIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        message: result.message,
        status: acme_kit::text::normalize(&result.status),
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
    };
    let sinks: Vec<Box<dyn FanSessionsIdWriter>> = vec![
        Box::new(RouteBindingGatewaySink {
            client: edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(GatewayPolicyNotificationSink {
            client:
                edgegateway::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient::new(
                    channel.clone(),
                ),
        }),
        Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[2].deliver(record).await
}

/// What `export_fan_sessions_id` keeps of each FindFanSessions response.
#[derive(Clone, Debug, Default)]
pub struct FanSessionsIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `export_fan_sessions_id` hands each record.
trait FanSessionsIdWriter {
    fn deliver(&self, record: FanSessionsIdRecord) -> Delivery;
}

/// Writes records to `acme.edgegateway.v1.RouteBindingGatewayService`.
struct RouteBindingGatewaySink {
    client: edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient<Channel>,
    context: common::RequestContext,
}

impl FanSessionsIdWriter for RouteBindingGatewaySink {
    fn deliver(&self, record: FanSessionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::TrackRouteBindingRequest {
                    details: Some(edgegateway::track_route_binding_request::Details {
                        session_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.track_route_binding(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.edgegateway.v1.GatewayPolicyNotificationService`.
struct GatewayPolicyNotificationSink {
    client: edgegateway::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient<Channel>,
}

impl FanSessionsIdWriter for GatewayPolicyNotificationSink {
    fn deliver(&self, record: FanSessionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .notify_gateway_policy_watchers(edgegateway::NotifyGatewayPolicyWatchersRequest {
                    gateway_policy_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl FanSessionsIdWriter for ThrottleCalculationSink {
    fn deliver(&self, record: FanSessionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::CompareThrottleOptionsRequest {
                    details: Some(ratelimiting::compare_throttle_options_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_throttle_options(request).await?;
            Ok(())
        })
    }
}
