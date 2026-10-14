//! `replay_gateway_policy_updates_description` reads
//! `acme.edgegateway.v1.GatewayPolicyNotificationService/UnsubscribeGatewayPolicyUpdates` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient::new(
            channel.clone(),
        );
    let loaded = source
        .unsubscribe_gateway_policy_updates(edgegateway::UnsubscribeGatewayPolicyUpdatesRequest {
            context: Some(context.clone()),
            reason: "primary-replay-gateway-policy-updates-description".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let gateway_policy = loaded.gateway_policy.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = GatewayPolicyUpdatesDescriptionRecord {
        description: gateway_policy.description,
        status: outcome.status,
        gateway_policy_id: acme_kit::text::normalize(&gateway_policy.gateway_policy_id),
    };
    act_on_gateway_policy_updates_description("api-key", record, channel, context).await
}

/// What `replay_gateway_policy_updates_description` keeps of each UnsubscribeGatewayPolicyUpdates response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyUpdatesDescriptionRecord {
    pub description: String,
    pub status: String,
    pub gateway_policy_id: String,
}

/// Where `replay_gateway_policy_updates_description` hands each record.
trait GatewayPolicyUpdatesDescriptionSink {
    fn deliver(&self, record: GatewayPolicyUpdatesDescriptionRecord) -> Delivery;
}

/// Sends records to `acme.featureflags.v1.FlagRolloutCalculationService`.
struct FlagRolloutCalculationSink {
    client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient<Channel>,
}

impl GatewayPolicyUpdatesDescriptionSink for FlagRolloutCalculationSink {
    fn deliver(&self, record: GatewayPolicyUpdatesDescriptionRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_flag_rollout_rates(featureflags::GetFlagRolloutRatesRequest {
                    flag_rollout_id: record.gateway_policy_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.edgegateway.v1.ApiKeyService`.
struct ApiKeySink {
    client: edgegateway::api_key_service_client::ApiKeyServiceClient<Channel>,
}

impl GatewayPolicyUpdatesDescriptionSink for ApiKeySink {
    fn deliver(&self, record: GatewayPolicyUpdatesDescriptionRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_api_key(edgegateway::CreateApiKeyRequest {
                    api_key: Some(edgegateway::ApiKey {
                        currency_code: record.gateway_policy_id,
                        api_key_id: record.description,
                        ..Default::default()
                    }),
                    request_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyService`.
struct CryptoKeySink {
    client: keymanagement::crypto_key_service_client::CryptoKeyServiceClient<Channel>,
}

impl GatewayPolicyUpdatesDescriptionSink for CryptoKeySink {
    fn deliver(&self, record: GatewayPolicyUpdatesDescriptionRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_crypto_keys(keymanagement::SearchCryptoKeysRequest {
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    filter: Some(keymanagement::search_crypto_keys_request::Filter {
                        region: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_gateway_policy_updates_description(
    kind: &str,
    record: GatewayPolicyUpdatesDescriptionRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(GatewayPolicyUpdatesDescriptionRecord) -> Delivery>> = BTreeMap::new();
    let flag_rollout_calculation_channel = channel.clone();
    actions.insert(
        "flag-rollout-calculation",
        Box::new(move |record| {
            FlagRolloutCalculationSink {
                client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
                    flag_rollout_calculation_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    let api_key_channel = channel.clone();
    actions.insert(
        "api-key",
        Box::new(move |record| {
            ApiKeySink {
                client: edgegateway::api_key_service_client::ApiKeyServiceClient::new(api_key_channel.clone()),
            }
            .deliver(record)
        }),
    );
    let crypto_key_channel = channel.clone();
    actions.insert(
        "crypto-key",
        Box::new(move |record| {
            CryptoKeySink {
                client: keymanagement::crypto_key_service_client::CryptoKeyServiceClient::new(
                    crypto_key_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
