//! `mirror_crypto_key_message` reads `acme.keymanagement.v1.CryptoKeyLifecycleService/ReinstateCryptoKey` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(channel.clone());
    let reply = source
        .reinstate_crypto_key(keymanagement::ReinstateCryptoKeyRequest {
            context: Some(context.clone()),
            crypto_key_id: "primary-mirror-crypto-key-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let record = CryptoKeyMessageRecord {
        message: acme_kit::text::normalize(&outcome.message),
        reference_id: outcome.reference_id,
    };
    act_on_crypto_key_message("rate-limit", record, channel, context).await
}

/// What `mirror_crypto_key_message` keeps of each ReinstateCryptoKey response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyMessageRecord {
    pub message: String,
    pub reference_id: String,
}

/// Where `mirror_crypto_key_message` hands each record.
trait CryptoKeyMessageTarget {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.RateLimitService`.
struct RateLimitSink {
    client: ratelimiting::rate_limit_service_client::RateLimitServiceClient<Channel>,
}

impl CryptoKeyMessageTarget for RateLimitSink {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_rate_limit(ratelimiting::UpdateRateLimitRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        name: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.RouteBindingGatewayService`.
struct RouteBindingGatewaySink {
    client: edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyMessageTarget for RouteBindingGatewaySink {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::FetchRouteBindingViewRequest {
                    route_binding: Some(edgegateway::RouteBinding {
                        name: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.fetch_route_binding_view(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.assets.v1.FixedAssetCalculationService`.
struct FixedAssetCalculationSink {
    client: assets::fixed_asset_calculation_service_client::FixedAssetCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyMessageTarget for FixedAssetCalculationSink {
    fn write(&self, record: CryptoKeyMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(assets::EstimateFixedAssetRequest {
                    reason: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.estimate_fixed_asset(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_crypto_key_message(
    kind: &str,
    record: CryptoKeyMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(CryptoKeyMessageRecord) -> Delivery>> = BTreeMap::new();
    let rate_limit_channel = channel.clone();
    actions.insert(
        "rate-limit",
        Box::new(move |record| {
            RateLimitSink {
                client: ratelimiting::rate_limit_service_client::RateLimitServiceClient::new(
                    rate_limit_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let route_binding_gateway_channel = channel.clone();
    let route_binding_gateway_context = context.clone();
    actions.insert(
        "route-binding-gateway",
        Box::new(move |record| {
            RouteBindingGatewaySink {
                client: edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient::new(
                    route_binding_gateway_channel.clone(),
                ),
                context: route_binding_gateway_context.clone(),
            }
            .write(record)
        }),
    );
    let fixed_asset_calculation_channel = channel.clone();
    let fixed_asset_calculation_context = context.clone();
    actions.insert(
        "fixed-asset-calculation",
        Box::new(move |record| {
            FixedAssetCalculationSink {
                client: assets::fixed_asset_calculation_service_client::FixedAssetCalculationServiceClient::new(
                    fixed_asset_calculation_channel.clone(),
                ),
                context: fixed_asset_calculation_context.clone(),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
