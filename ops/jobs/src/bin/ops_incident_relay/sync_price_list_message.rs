//! `sync_price_list_message` reads `acme.pricing.v1.PriceListService/DeletePriceList` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::pricing::v1 as pricing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = pricing::price_list_service_client::PriceListServiceClient::new(channel.clone());
    let fetched = source
        .delete_price_list(pricing::DeletePriceListRequest {
            context: Some(context.clone()),
            price_list_id: "live-sync-price-list-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = PriceListMessageRecord {
        message: result.message,
        status: acme_kit::text::normalize(&result.status),
    };
    let sinks: Vec<Box<dyn PriceListMessageWriter>> = vec![
        Box::new(AuditEventQuerySink {
            client: audit::audit_event_query_service_client::AuditEventQueryServiceClient::new(channel.clone()),
        }),
        Box::new(RouteBindingNotificationSink {
            client: edgegateway::route_binding_notification_service_client::RouteBindingNotificationServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    ];
    for sink in &sinks {
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `sync_price_list_message` keeps of each DeletePriceList response.
#[derive(Clone, Debug, Default)]
pub struct PriceListMessageRecord {
    pub message: String,
    pub status: String,
}

/// Where `sync_price_list_message` hands each record.
trait PriceListMessageWriter {
    fn forward(&self, record: PriceListMessageRecord) -> Delivery;
}

/// Passes records to `acme.audit.v1.AuditEventQueryService`.
struct AuditEventQuerySink {
    client: audit::audit_event_query_service_client::AuditEventQueryServiceClient<Channel>,
}

impl PriceListMessageWriter for AuditEventQuerySink {
    fn forward(&self, record: PriceListMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_audit_events(audit::FindAuditEventsRequest {
                    filter: Some(audit::find_audit_events_request::Filter {
                        region: record.message,
                        owner_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.RouteBindingNotificationService`.
struct RouteBindingNotificationSink {
    client: edgegateway::route_binding_notification_service_client::RouteBindingNotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl PriceListMessageWriter for RouteBindingNotificationSink {
    fn forward(&self, record: PriceListMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::ListRouteBindingSubscriptionsRequest {
                    filter: Some(edgegateway::list_route_binding_subscriptions_request::Filter {
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_route_binding_subscriptions(request).await?;
            Ok(())
        })
    }
}
