//! `route_crypto_key_lifecycle_id` reads
//! `acme.keymanagement.v1.CryptoKeyLifecycleService/GetCryptoKeyLifecycle` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::treasury::v1 as treasury;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(channel.clone());
    let reply = source
        .get_crypto_key_lifecycle(keymanagement::GetCryptoKeyLifecycleRequest {
            context: Some(context.clone()),
            crypto_key_id: "main-route-crypto-key-lifecycle-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let crypto_key = reply.crypto_key.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = CryptoKeyLifecycleIdRecord {
        crypto_key_id: crypto_key.crypto_key_id,
        currency_code: crypto_key.currency_code,
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
    };
    dispatch_crypto_key_lifecycle_id("cash-position-calculation", record, channel, &context).await
}

/// What `route_crypto_key_lifecycle_id` keeps of each GetCryptoKeyLifecycle response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyLifecycleIdRecord {
    pub crypto_key_id: String,
    pub currency_code: String,
    pub status: String,
    pub message: String,
}

/// Where `route_crypto_key_lifecycle_id` hands each record.
trait CryptoKeyLifecycleIdWriter {
    fn forward(&self, record: CryptoKeyLifecycleIdRecord) -> Delivery;
}

/// Passes records to `acme.edgegateway.v1.RouteBindingIntegrationService`.
struct RouteBindingIntegrationSink {
    client: edgegateway::route_binding_integration_service_client::RouteBindingIntegrationServiceClient<Channel>,
}

impl CryptoKeyLifecycleIdWriter for RouteBindingIntegrationSink {
    fn forward(&self, record: CryptoKeyLifecycleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .replay_route_binding_events(edgegateway::ReplayRouteBindingEventsRequest {
                    route_binding_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.treasury.v1.CashPositionCalculationService`.
struct CashPositionCalculationSink {
    client: treasury::cash_position_calculation_service_client::CashPositionCalculationServiceClient<Channel>,
}

impl CryptoKeyLifecycleIdWriter for CashPositionCalculationSink {
    fn forward(&self, record: CryptoKeyLifecycleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_cash_position_calculation(treasury::ExplainCashPositionCalculationRequest {
                    cash_position: Some(treasury::CashPosition {
                        cash_position_id: record.status,
                        currency_code: record.currency_code,
                        ..Default::default()
                    }),
                    cash_position_id: record.crypto_key_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.CryptoKeyWorkflowService`.
struct CryptoKeyWorkflowSink {
    client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient<Channel>,
}

impl CryptoKeyLifecycleIdWriter for CryptoKeyWorkflowSink {
    fn forward(&self, record: CryptoKeyLifecycleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pause_crypto_key_workflow(keymanagement::PauseCryptoKeyWorkflowRequest {
                    details: Some(keymanagement::pause_crypto_key_workflow_request::Details {
                        initiated_by: record.status,
                        step: record.currency_code,
                        ..Default::default()
                    }),
                    crypto_key: Some(keymanagement::CryptoKey {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct CryptoKeyLifecycleIdLogOnly;

impl CryptoKeyLifecycleIdWriter for CryptoKeyLifecycleIdLogOnly {
    fn forward(&self, record: CryptoKeyLifecycleIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.crypto_key_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_crypto_key_lifecycle_id(
    kind: &str,
    record: CryptoKeyLifecycleIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let route_binding_integration = RouteBindingIntegrationSink {
        client: edgegateway::route_binding_integration_service_client::RouteBindingIntegrationServiceClient::new(
            channel.clone(),
        ),
    };
    let cash_position_calculation = CashPositionCalculationSink {
        client: treasury::cash_position_calculation_service_client::CashPositionCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let crypto_key_workflow = CryptoKeyWorkflowSink {
        client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(channel),
    };
    let write: &dyn Fn(CryptoKeyLifecycleIdRecord) -> Delivery = match kind {
        "route-binding-integration" => &|record| route_binding_integration.forward(record),
        "cash-position-calculation" => &|record| cash_position_calculation.forward(record),
        "crypto-key-workflow" => &|record| crypto_key_workflow.forward(record),
        _ => &|record| CryptoKeyLifecycleIdLogOnly.forward(record),
    };
    write(record).await
}
