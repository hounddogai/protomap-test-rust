//! `relay_route_binding_status` reads `acme.edgegateway.v1.RouteBindingService/CloneRouteBinding` and hands
//! what it keeps to `acme.promotions.v1.PromotionService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::promotions::v1 as promotions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::route_binding_service_client::RouteBindingServiceClient::new(channel.clone());
    let loaded = source
        .clone_route_binding(edgegateway::CloneRouteBindingRequest {
            context: Some(context.clone()),
            request_id: "default-relay-route-binding-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let route_binding = loaded.route_binding.unwrap_or_default();
    let record = RouteBindingStatusRecord {
        status: result.status,
        message: result.message,
        description: route_binding.description,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    PromotionSink {
        client: promotions::promotion_service_client::PromotionServiceClient::new(channel),
        context: context.clone(),
    }
    .write(record)
    .await
}

/// What `relay_route_binding_status` keeps of each CloneRouteBinding response.
#[derive(Clone, Debug, Default)]
pub struct RouteBindingStatusRecord {
    pub status: String,
    pub message: String,
    pub description: String,
    pub correlation_id: String,
}

/// Where `relay_route_binding_status` hands each record.
trait RouteBindingStatusWriter {
    fn write(&self, record: RouteBindingStatusRecord) -> Delivery;
}

/// Passes records to `acme.promotions.v1.PromotionService`.
struct PromotionSink {
    client: promotions::promotion_service_client::PromotionServiceClient<Channel>,
    context: common::RequestContext,
}

impl RouteBindingStatusWriter for PromotionSink {
    fn write(&self, record: RouteBindingStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(promotions::ListPromotionsRequest {
                    filter: Some(promotions::list_promotions_request::Filter {
                        region: record.message,
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_promotions(request).await?;
            Ok(())
        })
    }
}
