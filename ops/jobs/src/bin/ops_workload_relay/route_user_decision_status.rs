//! `route_user_decision_status` reads `acme.identity.v1.UserValidationService/ExplainUserDecision` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::catalog::v1 as catalog;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = identity::user_validation_service_client::UserValidationServiceClient::new(channel.clone());
    let reply = source
        .explain_user_decision(identity::ExplainUserDecisionRequest {
            context: Some(context.clone()),
            user_id: "nightly-route-user-decision-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = UserDecisionStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
        outcome_status: outcome.status,
    };
    act_on_user_decision_status("fixed-asset-query", record, channel, context).await
}

/// What `route_user_decision_status` keeps of each ExplainUserDecision response.
#[derive(Clone, Debug, Default)]
pub struct UserDecisionStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub reference_id: String,
    pub outcome_status: String,
}

/// Where `route_user_decision_status` hands each record.
trait UserDecisionStatusTarget {
    fn forward(&self, record: UserDecisionStatusRecord) -> Delivery;
}

/// Hands records to `acme.catalog.v1.ProductCalculationService`.
struct ProductCalculationSink {
    client: catalog::product_calculation_service_client::ProductCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl UserDecisionStatusTarget for ProductCalculationSink {
    fn forward(&self, record: UserDecisionStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(catalog::CalculateProductRequest {
                    product: Some(catalog::Product {
                        product_id: record.reference_id,
                        ..Default::default()
                    }),
                    product_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.calculate_product(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.assets.v1.FixedAssetQueryService`.
struct FixedAssetQuerySink {
    client: assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient<Channel>,
}

impl UserDecisionStatusTarget for FixedAssetQuerySink {
    fn forward(&self, record: UserDecisionStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_fixed_assets(assets::SummarizeFixedAssetsRequest {
                    details: Some(assets::summarize_fixed_assets_request::Details {
                        format: record.status,
                        ..Default::default()
                    }),
                    reason: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_user_decision_status(
    kind: &str,
    record: UserDecisionStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(UserDecisionStatusRecord) -> Delivery>> = BTreeMap::new();
    let product_calculation_channel = channel.clone();
    let product_calculation_context = context.clone();
    actions.insert(
        "product-calculation",
        Box::new(move |record| {
            ProductCalculationSink {
                client: catalog::product_calculation_service_client::ProductCalculationServiceClient::new(
                    product_calculation_channel.clone(),
                ),
                context: product_calculation_context.clone(),
            }
            .forward(record)
        }),
    );
    let fixed_asset_query_channel = channel.clone();
    actions.insert(
        "fixed-asset-query",
        Box::new(move |record| {
            FixedAssetQuerySink {
                client: assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient::new(
                    fixed_asset_query_channel.clone(),
                ),
            }
            .forward(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
