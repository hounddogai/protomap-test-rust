//! `hand_off_route_binding_page_name` reads
//! `acme.edgegateway.v1.RouteBindingGatewayService/ComposeRouteBindingPage` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::subscriptions::v1 as subscriptions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient::new(channel.clone());
    let fetched = source
        .compose_route_binding_page(edgegateway::ComposeRouteBindingPageRequest {
            context: Some(context.clone()),
            route_binding_id: "current-hand-off-route-binding-page-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let route_binding = fetched.route_binding.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = OffRouteBindingPageNameRecord {
        name: acme_kit::text::normalize(&route_binding.name),
        description: acme_kit::text::normalize(&route_binding.description),
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffRouteBindingPageNameWriter>> = BTreeMap::new();
    sinks.insert(
        "document",
        Box::new(DocumentSink {
            client: documents::document_service_client::DocumentServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "key-rotation-workflow",
        Box::new(KeyRotationWorkflowSink {
            client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "api-key-workflow",
        Box::new(ApiKeyWorkflowSink {
            client: edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "subscription",
        Box::new(SubscriptionSink {
            client: subscriptions::subscription_service_client::SubscriptionServiceClient::new(channel),
        }),
    );
    match sinks.get("document") {
        Some(sink) => sink.write(record).await,
        None => anyhow::bail!("no sink registered for document"),
    }
}

/// What `hand_off_route_binding_page_name` keeps of each ComposeRouteBindingPage response.
#[derive(Clone, Debug, Default)]
pub struct OffRouteBindingPageNameRecord {
    pub name: String,
    pub description: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `hand_off_route_binding_page_name` hands each record.
trait OffRouteBindingPageNameWriter {
    fn write(&self, record: OffRouteBindingPageNameRecord) -> Delivery;
}

/// Writes records to `acme.documents.v1.DocumentService`.
struct DocumentSink {
    client: documents::document_service_client::DocumentServiceClient<Channel>,
}

impl OffRouteBindingPageNameWriter for DocumentSink {
    fn write(&self, record: OffRouteBindingPageNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_document(documents::GetDocumentRequest {
                    document_id: record.description,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffRouteBindingPageNameWriter for KeyRotationWorkflowSink {
    fn write(&self, record: OffRouteBindingPageNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::CancelKeyRotationWorkflowRequest {
                    key_rotation: Some(keymanagement::KeyRotation {
                        key_rotation_id: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(keymanagement::cancel_key_rotation_workflow_request::Details {
                        step: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.cancel_key_rotation_workflow(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.edgegateway.v1.ApiKeyWorkflowService`.
struct ApiKeyWorkflowSink {
    client: edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient<Channel>,
}

impl OffRouteBindingPageNameWriter for ApiKeyWorkflowSink {
    fn write(&self, record: OffRouteBindingPageNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .signal_api_key_workflow(edgegateway::SignalApiKeyWorkflowRequest {
                    details: Some(edgegateway::signal_api_key_workflow_request::Details {
                        initiated_by: record.description,
                        ..Default::default()
                    }),
                    reason: record.message,
                    api_key: Some(edgegateway::ApiKey {
                        currency_code: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.subscriptions.v1.SubscriptionService`.
struct SubscriptionSink {
    client: subscriptions::subscription_service_client::SubscriptionServiceClient<Channel>,
}

impl OffRouteBindingPageNameWriter for SubscriptionSink {
    fn write(&self, record: OffRouteBindingPageNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_subscription(subscriptions::DeleteSubscriptionRequest {
                    reason: record.correlation_id,
                    etag: record.name,
                    subscription_id: record.description,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
