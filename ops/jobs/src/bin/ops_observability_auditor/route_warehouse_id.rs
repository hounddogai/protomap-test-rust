//! `route_warehouse_id` reads `acme.warehousing.v1.WarehouseQueryService/PeekWarehouse` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = warehousing::warehouse_query_service_client::WarehouseQueryServiceClient::new(channel.clone());
    let loaded = source
        .peek_warehouse(warehousing::PeekWarehouseRequest {
            context: Some(context.clone()),
            warehouse_id: "primary-route-warehouse-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let warehouse = loaded.warehouse.unwrap_or_default();
    let record = WarehouseIdRecord {
        correlation_id: result.correlation_id,
        code: warehouse.code,
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
    };
    deliver_warehouse_id("experiment-calculation", record, channel, &context).await
}

/// What `route_warehouse_id` keeps of each PeekWarehouse response.
#[derive(Clone, Debug, Default)]
pub struct WarehouseIdRecord {
    pub correlation_id: String,
    pub code: String,
    pub status: String,
    pub message: String,
}

/// Where `route_warehouse_id` hands each record.
trait WarehouseIdWriter {
    fn apply(&self, record: WarehouseIdRecord) -> Delivery;
}

/// Passes records to `acme.edgegateway.v1.ApiKeyNotificationService`.
struct ApiKeyNotificationSink {
    client: edgegateway::api_key_notification_service_client::ApiKeyNotificationServiceClient<Channel>,
}

impl WarehouseIdWriter for ApiKeyNotificationSink {
    fn apply(&self, record: WarehouseIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .subscribe_api_key_updates(edgegateway::SubscribeApiKeyUpdatesRequest {
                    api_key_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.experiments.v1.ExperimentCalculationService`.
struct ExperimentCalculationSink {
    client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient<Channel>,
}

impl WarehouseIdWriter for ExperimentCalculationSink {
    fn apply(&self, record: WarehouseIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_experiment(experiments::EstimateExperimentRequest {
                    experiment: Some(experiments::Experiment {
                        title: record.message,
                        ..Default::default()
                    }),
                    reason: record.code,
                    details: Some(experiments::estimate_experiment_request::Details {
                        currency_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.edgegateway.v1.RouteBindingNotificationService`.
struct RouteBindingNotificationSink {
    client: edgegateway::route_binding_notification_service_client::RouteBindingNotificationServiceClient<Channel>,
}

impl WarehouseIdWriter for RouteBindingNotificationSink {
    fn apply(&self, record: WarehouseIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_route_binding_subscriptions(edgegateway::ListRouteBindingSubscriptionsRequest {
                    order_by: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.KeyRingQueryService`.
struct KeyRingQuerySink {
    client: keymanagement::key_ring_query_service_client::KeyRingQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl WarehouseIdWriter for KeyRingQuerySink {
    fn apply(&self, record: WarehouseIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::BrowseKeyRingsRequest {
                    order_by: record.code,
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_key_rings(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct WarehouseIdLogOnly;

impl WarehouseIdWriter for WarehouseIdLogOnly {
    fn apply(&self, record: WarehouseIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_warehouse_id(
    kind: &str,
    record: WarehouseIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn WarehouseIdWriter> = match kind {
        "api-key-notification" => Box::new(ApiKeyNotificationSink {
            client: edgegateway::api_key_notification_service_client::ApiKeyNotificationServiceClient::new(
                channel.clone(),
            ),
        }),
        "experiment-calculation" => Box::new(ExperimentCalculationSink {
            client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        "route-binding-notification" => Box::new(RouteBindingNotificationSink {
            client: edgegateway::route_binding_notification_service_client::RouteBindingNotificationServiceClient::new(
                channel.clone(),
            ),
        }),
        "key-ring-query" => Box::new(KeyRingQuerySink {
            client: keymanagement::key_ring_query_service_client::KeyRingQueryServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(WarehouseIdLogOnly),
    };
    sink.apply(record).await
}
