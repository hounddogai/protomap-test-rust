//! `propagate_forum_id` reads `acme.community.v1.ForumService/GetForum` and hands what it keeps to the sink
//! of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::checkout::v1 as checkout;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = community::forum_service_client::ForumServiceClient::new(channel.clone());
    let loaded = source
        .get_forum(community::GetForumRequest {
            context: Some(context.clone()),
            forum_id: "live-propagate-forum-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let forum = loaded.forum.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = ForumIdRecord {
        parent_id: forum.parent_id,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        name: acme_kit::text::normalize(&forum.name),
    };
    act_on_forum_id("checkout-session-query", record, channel, context).await
}

/// What `propagate_forum_id` keeps of each GetForum response.
#[derive(Clone, Debug, Default)]
pub struct ForumIdRecord {
    pub parent_id: String,
    pub correlation_id: String,
    pub name: String,
}

/// Where `propagate_forum_id` hands each record.
trait ForumIdSink {
    fn deliver(&self, record: ForumIdRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FlagRolloutValidationService`.
struct FlagRolloutValidationSink {
    client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ForumIdSink for FlagRolloutValidationSink {
    fn deliver(&self, record: ForumIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::LintFlagRolloutsRequest {
                    flag_rollout: Some(featureflags::FlagRollout {
                        title: record.correlation_id,
                        flag_rollout_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_flag_rollouts(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.deliveries.v1.DeliveryQueryService`.
struct DeliveryQuerySink {
    client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient<Channel>,
}

impl ForumIdSink for DeliveryQuerySink {
    fn deliver(&self, record: ForumIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_deliveries(deliveries::BrowseDeliveriesRequest {
                    order_by: record.correlation_id,
                    filter: Some(deliveries::browse_deliveries_request::Filter {
                        owner_id: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.checkout.v1.CheckoutSessionQueryService`.
struct CheckoutSessionQuerySink {
    client: checkout::checkout_session_query_service_client::CheckoutSessionQueryServiceClient<Channel>,
}

impl ForumIdSink for CheckoutSessionQuerySink {
    fn deliver(&self, record: ForumIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_checkout_sessions(checkout::CountCheckoutSessionsRequest {
                    filter: Some(checkout::count_checkout_sessions_request::Filter {
                        owner_id: record.parent_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.ApiKeyService`.
struct ApiKeySink {
    client: edgegateway::api_key_service_client::ApiKeyServiceClient<Channel>,
}

impl ForumIdSink for ApiKeySink {
    fn deliver(&self, record: ForumIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_api_key(edgegateway::CloneApiKeyRequest {
                    request_id: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_forum_id(
    kind: &str,
    record: ForumIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(ForumIdRecord) -> Delivery>> = BTreeMap::new();
    let flag_rollout_validation_channel = channel.clone();
    let flag_rollout_validation_context = context.clone();
    actions.insert(
        "flag-rollout-validation",
        Box::new(move |record| {
            FlagRolloutValidationSink {
                client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient::new(
                    flag_rollout_validation_channel.clone(),
                ),
                context: flag_rollout_validation_context.clone(),
            }
            .deliver(record)
        }),
    );
    let delivery_query_channel = channel.clone();
    actions.insert(
        "delivery-query",
        Box::new(move |record| {
            DeliveryQuerySink {
                client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(
                    delivery_query_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    let checkout_session_query_channel = channel.clone();
    actions.insert(
        "checkout-session-query",
        Box::new(move |record| {
            CheckoutSessionQuerySink {
                client: checkout::checkout_session_query_service_client::CheckoutSessionQueryServiceClient::new(
                    checkout_session_query_channel.clone(),
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
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
