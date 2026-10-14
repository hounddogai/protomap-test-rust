//! `propagate_yard_id` reads `acme.yards.v1.YardService/GetYard` and hands what it keeps to the sink of its
//! kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::warehousing::v1 as warehousing;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = yards::yard_service_client::YardServiceClient::new(channel.clone());
    let reply = source
        .get_yard(yards::GetYardRequest {
            context: Some(context.clone()),
            yard_id: "current-propagate-yard-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let yard = reply.yard.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = YardIdRecord {
        yard_id: yard.yard_id,
        correlation_id: result.correlation_id,
        code: yard.code,
    };
    act_on_yard_id("config-set-lifecycle", record, channel, context).await
}

/// What `propagate_yard_id` keeps of each GetYard response.
#[derive(Clone, Debug, Default)]
pub struct YardIdRecord {
    pub yard_id: String,
    pub correlation_id: String,
    pub code: String,
}

/// Where `propagate_yard_id` hands each record.
trait YardIdTarget {
    fn apply(&self, record: YardIdRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.ConfigSetLifecycleService`.
struct ConfigSetLifecycleSink {
    client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient<Channel>,
}

impl YardIdTarget for ConfigSetLifecycleSink {
    fn apply(&self, record: YardIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .expire_config_set(configuration::ExpireConfigSetRequest {
                    details: Some(configuration::expire_config_set_request::Details {
                        reason_code: record.code,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.warehousing.v1.WarehouseQueryService`.
struct WarehouseQuerySink {
    client: warehousing::warehouse_query_service_client::WarehouseQueryServiceClient<Channel>,
}

impl YardIdTarget for WarehouseQuerySink {
    fn apply(&self, record: YardIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_warehouses(warehousing::FindWarehousesRequest {
                    filter: Some(warehousing::find_warehouses_request::Filter {
                        query: record.correlation_id,
                        owner_id: record.code,
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
async fn act_on_yard_id(
    kind: &str,
    record: YardIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(YardIdRecord) -> Delivery>> = BTreeMap::new();
    let config_set_lifecycle_channel = channel.clone();
    actions.insert(
        "config-set-lifecycle",
        Box::new(move |record| {
            ConfigSetLifecycleSink {
                client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(
                    config_set_lifecycle_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let warehouse_query_channel = channel.clone();
    actions.insert(
        "warehouse-query",
        Box::new(move |record| {
            WarehouseQuerySink {
                client: warehousing::warehouse_query_service_client::WarehouseQueryServiceClient::new(
                    warehouse_query_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
