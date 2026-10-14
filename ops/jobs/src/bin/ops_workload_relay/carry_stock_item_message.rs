//! `carry_stock_item_message` reads `acme.inventory.v1.StockItemQueryService/LookupStockItem` and hands what
//! it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::inventory::v1 as inventory;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel.clone());
    let loaded = source
        .lookup_stock_item(inventory::LookupStockItemRequest {
            context: Some(context.clone()),
            stock_item_id: "nightly-carry-stock-item-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let stock_item = loaded.stock_item.unwrap_or_default();
    let record = StockItemMessageRecord {
        message: result.message,
        stock_item_id: acme_kit::text::normalize(&stock_item.stock_item_id),
        name: stock_item.name,
    };
    act_on_stock_item_message("feature-flag-history", record, channel, context).await
}

/// What `carry_stock_item_message` keeps of each LookupStockItem response.
#[derive(Clone, Debug, Default)]
pub struct StockItemMessageRecord {
    pub message: String,
    pub stock_item_id: String,
    pub name: String,
}

/// Where `carry_stock_item_message` hands each record.
trait StockItemMessageWriter {
    fn send(&self, record: StockItemMessageRecord) -> Delivery;
}

/// Sends records to `acme.authorization.v1.AccessPolicyValidationService`.
struct AccessPolicyValidationSink {
    client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient<Channel>,
}

impl StockItemMessageWriter for AccessPolicyValidationSink {
    fn send(&self, record: StockItemMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_access_policy(authorization::ScoreAccessPolicyRequest {
                    details: Some(authorization::score_access_policy_request::Details {
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

/// Hands records to `acme.featureflags.v1.FeatureFlagHistoryService`.
struct FeatureFlagHistorySink {
    client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient<Channel>,
    context: common::RequestContext,
}

impl StockItemMessageWriter for FeatureFlagHistorySink {
    fn send(&self, record: StockItemMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::CompareFeatureFlagVersionsRequest {
                    reason: record.name,
                    details: Some(featureflags::compare_feature_flag_versions_request::Details {
                        author_email: record.stock_item_id,
                        ..Default::default()
                    }),
                    feature_flag_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_feature_flag_versions(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.AccessPolicyNotificationService`.
struct AccessPolicyNotificationSink {
    client: authorization::access_policy_notification_service_client::AccessPolicyNotificationServiceClient<Channel>,
}

impl StockItemMessageWriter for AccessPolicyNotificationSink {
    fn send(&self, record: StockItemMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .unsubscribe_access_policy_updates(authorization::UnsubscribeAccessPolicyUpdatesRequest {
                    details: Some(authorization::unsubscribe_access_policy_updates_request::Details {
                        channel: record.stock_item_id,
                        ..Default::default()
                    }),
                    reason: record.name,
                    access_policy_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_stock_item_message(
    kind: &str,
    record: StockItemMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(StockItemMessageRecord) -> Delivery>> = BTreeMap::new();
    let access_policy_validation_channel = channel.clone();
    actions.insert(
        "access-policy-validation",
        Box::new(move |record| {
            AccessPolicyValidationSink {
                client:
                    authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
                        access_policy_validation_channel.clone(),
                    ),
            }
            .send(record)
        }),
    );
    let feature_flag_history_channel = channel.clone();
    let feature_flag_history_context = context.clone();
    actions.insert(
        "feature-flag-history",
        Box::new(move |record| {
            FeatureFlagHistorySink {
                client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(
                    feature_flag_history_channel.clone(),
                ),
                context: feature_flag_history_context.clone(),
            }
            .send(record)
        }),
    );
    let access_policy_notification_channel = channel.clone();
    actions.insert("access-policy-notification", Box::new(move |record| AccessPolicyNotificationSink { client: authorization::access_policy_notification_service_client::AccessPolicyNotificationServiceClient::new(access_policy_notification_channel.clone()) }.send(record)));
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
