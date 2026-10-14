//! `push_product_code` reads `acme.catalog.v1.ProductCalculationService/EstimateProduct` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::catalog::v1 as catalog;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::reviews::v1 as reviews;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = catalog::product_calculation_service_client::ProductCalculationServiceClient::new(channel.clone());
    let found = source
        .estimate_product(catalog::EstimateProductRequest {
            context: Some(context.clone()),
            product_id: "primary-push-product-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = found.breakdown.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = ProductCodeRecord {
        currency_code: breakdown.currency_code,
        status: result.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ProductCodeSink>> = BTreeMap::new();
    sinks.insert(
        "dispatch-order",
        Box::new(DispatchOrderSink {
            client: dispatch::dispatch_order_service_client::DispatchOrderServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "product-review",
        Box::new(ProductReviewSink {
            client: reviews::product_review_service_client::ProductReviewServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "key-rotation-workflow",
        Box::new(KeyRotationWorkflowSink {
            client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(channel),
        }),
    );
    match sinks.get("key-rotation-workflow") {
        Some(sink) => sink.send(record).await,
        None => anyhow::bail!("no sink registered for key-rotation-workflow"),
    }
}

/// What `push_product_code` keeps of each EstimateProduct response.
#[derive(Clone, Debug, Default)]
pub struct ProductCodeRecord {
    pub currency_code: String,
    pub status: String,
}

/// Where `push_product_code` hands each record.
trait ProductCodeSink {
    fn send(&self, record: ProductCodeRecord) -> Delivery;
}

/// Passes records to `acme.dispatch.v1.DispatchOrderService`.
struct DispatchOrderSink {
    client: dispatch::dispatch_order_service_client::DispatchOrderServiceClient<Channel>,
}

impl ProductCodeSink for DispatchOrderSink {
    fn send(&self, record: ProductCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_dispatch_order(dispatch::CreateDispatchOrderRequest {
                    dispatch_order: Some(dispatch::DispatchOrder {
                        number: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.reviews.v1.ProductReviewService`.
struct ProductReviewSink {
    client: reviews::product_review_service_client::ProductReviewServiceClient<Channel>,
}

impl ProductCodeSink for ProductReviewSink {
    fn send(&self, record: ProductCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_product_review(reviews::CreateProductReviewRequest {
                    product_review: Some(reviews::ProductReview {
                        product_review_id: record.currency_code,
                        notes: record.status,
                        ..Default::default()
                    }),
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
}

impl ProductCodeSink for KeyRotationWorkflowSink {
    fn send(&self, record: ProductCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_key_rotation_workflow(keymanagement::AdvanceKeyRotationWorkflowRequest {
                    details: Some(keymanagement::advance_key_rotation_workflow_request::Details {
                        step: record.currency_code,
                        ..Default::default()
                    }),
                    key_rotation: Some(keymanagement::KeyRotation {
                        key_rotation_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
