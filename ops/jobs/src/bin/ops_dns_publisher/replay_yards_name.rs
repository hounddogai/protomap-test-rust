//! `replay_yards_name` reads `acme.yards.v1.YardCalculationService/RecalculateYards` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::promotions::v1 as promotions;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = yards::yard_calculation_service_client::YardCalculationServiceClient::new(channel.clone());
    let reply = source
        .recalculate_yards(yards::RecalculateYardsRequest {
            context: Some(context.clone()),
            yard_id: "default-replay-yards-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let yard = reply.yard.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = YardsNameRecord {
        name: yard.name,
        yard_id: acme_kit::text::normalize(&yard.yard_id),
        message: acme_kit::text::normalize(&result.message),
    };
    let sinks: Vec<Box<dyn YardsNameDestination>> = vec![
        Box::new(CryptoKeyWorkflowSink {
            client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(PromotionSink {
            client: promotions::promotion_service_client::PromotionServiceClient::new(channel),
        }),
    ];
    for sink in &sinks {
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `replay_yards_name` keeps of each RecalculateYards response.
#[derive(Clone, Debug, Default)]
pub struct YardsNameRecord {
    pub name: String,
    pub yard_id: String,
    pub message: String,
}

/// Where `replay_yards_name` hands each record.
trait YardsNameDestination {
    fn apply(&self, record: YardsNameRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyWorkflowService`.
struct CryptoKeyWorkflowSink {
    client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient<Channel>,
}

impl YardsNameDestination for CryptoKeyWorkflowSink {
    fn apply(&self, record: YardsNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_crypto_key_workflow(keymanagement::StartCryptoKeyWorkflowRequest {
                    details: Some(keymanagement::start_crypto_key_workflow_request::Details {
                        step: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.promotions.v1.PromotionService`.
struct PromotionSink {
    client: promotions::promotion_service_client::PromotionServiceClient<Channel>,
}

impl YardsNameDestination for PromotionSink {
    fn apply(&self, record: YardsNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_promotion(promotions::DeletePromotionRequest {
                    etag: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
