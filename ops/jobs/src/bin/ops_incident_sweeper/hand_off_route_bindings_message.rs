//! `hand_off_route_bindings_message` reads
//! `acme.edgegateway.v1.RouteBindingQueryService/ResolveRouteBindings` and hands what it keeps to the sink of
//! its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::route_binding_query_service_client::RouteBindingQueryServiceClient::new(channel.clone());
    let reply = source
        .resolve_route_bindings(edgegateway::ResolveRouteBindingsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = OffRouteBindingsMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
    };
    deliver_off_route_bindings_message("permission-check-history", record, channel, &context).await
}

/// What `hand_off_route_bindings_message` keeps of each ResolveRouteBindings response.
#[derive(Clone, Debug, Default)]
pub struct OffRouteBindingsMessageRecord {
    pub message: String,
    pub correlation_id: String,
}

/// Where `hand_off_route_bindings_message` hands each record.
trait OffRouteBindingsMessageWriter {
    fn push(&self, record: OffRouteBindingsMessageRecord) -> Delivery;
}

/// Writes records to `acme.deliveries.v1.DeliveryQueryService`.
struct DeliveryQuerySink {
    client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient<Channel>,
}

impl OffRouteBindingsMessageWriter for DeliveryQuerySink {
    fn push(&self, record: OffRouteBindingsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_deliveries(deliveries::SummarizeDeliveriesRequest {
                    delivery_id: record.message,
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.broadcasts.v1.BroadcastQueryService`.
struct BroadcastQuerySink {
    client: broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient<Channel>,
}

impl OffRouteBindingsMessageWriter for BroadcastQuerySink {
    fn push(&self, record: OffRouteBindingsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_broadcast(broadcasts::LookupBroadcastRequest {
                    broadcast_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.authorization.v1.PermissionCheckHistoryService`.
struct PermissionCheckHistorySink {
    client: authorization::permission_check_history_service_client::PermissionCheckHistoryServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffRouteBindingsMessageWriter for PermissionCheckHistorySink {
    fn push(&self, record: OffRouteBindingsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::RevertPermissionCheckRequest {
                    permission_check: Some(authorization::PermissionCheck {
                        source: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.revert_permission_check(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct OffRouteBindingsMessageLogOnly;

impl OffRouteBindingsMessageWriter for OffRouteBindingsMessageLogOnly {
    fn push(&self, record: OffRouteBindingsMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_off_route_bindings_message(
    kind: &str,
    record: OffRouteBindingsMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn OffRouteBindingsMessageWriter> = match kind {
        "delivery-query" => Box::new(DeliveryQuerySink {
            client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel.clone()),
        }),
        "broadcast-query" => Box::new(BroadcastQuerySink {
            client: broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient::new(channel.clone()),
        }),
        "permission-check-history" => Box::new(PermissionCheckHistorySink {
            client: authorization::permission_check_history_service_client::PermissionCheckHistoryServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
        _ => Box::new(OffRouteBindingsMessageLogOnly),
    };
    sink.push(record).await
}
