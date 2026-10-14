//! `forward_environment_expiry_code` reads
//! `acme.configuration.v1.EnvironmentLifecycleService/ScheduleEnvironmentExpiry` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::drivers::v1 as drivers;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::promotions::v1 as promotions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient::new(channel.clone());
    let found = source
        .schedule_environment_expiry(configuration::ScheduleEnvironmentExpiryRequest {
            context: Some(context.clone()),
            environment_id: "live-forward-environment-expiry-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let environment = found.environment.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = EnvironmentExpiryCodeRecord {
        code: acme_kit::text::normalize(&environment.code),
        message: acme_kit::text::normalize(&result.message),
        name: environment.name,
        reference_id: outcome.reference_id,
    };
    dispatch_environment_expiry_code("promotion-calculation", record, channel, &context).await
}

/// What `forward_environment_expiry_code` keeps of each ScheduleEnvironmentExpiry response.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentExpiryCodeRecord {
    pub code: String,
    pub message: String,
    pub name: String,
    pub reference_id: String,
}

/// Where `forward_environment_expiry_code` hands each record.
trait EnvironmentExpiryCodeDestination {
    fn forward(&self, record: EnvironmentExpiryCodeRecord) -> Delivery;
}

/// Writes records to `acme.edgegateway.v1.ApiKeyWorkflowService`.
struct ApiKeyWorkflowSink {
    client: edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient<Channel>,
}

impl EnvironmentExpiryCodeDestination for ApiKeyWorkflowSink {
    fn forward(&self, record: EnvironmentExpiryCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pause_api_key_workflow(edgegateway::PauseApiKeyWorkflowRequest {
                    api_key: Some(edgegateway::ApiKey {
                        currency_code: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.drivers.v1.DriverService`.
struct DriverSink {
    client: drivers::driver_service_client::DriverServiceClient<Channel>,
    context: common::RequestContext,
}

impl EnvironmentExpiryCodeDestination for DriverSink {
    fn forward(&self, record: EnvironmentExpiryCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(drivers::GetDriverRequest {
                    driver_id: record.reference_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_driver(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.promotions.v1.PromotionCalculationService`.
struct PromotionCalculationSink {
    client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient<Channel>,
}

impl EnvironmentExpiryCodeDestination for PromotionCalculationSink {
    fn forward(&self, record: EnvironmentExpiryCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_promotion(promotions::SimulatePromotionRequest {
                    promotion: Some(promotions::Promotion {
                        description: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.edgegateway.v1.RouteBindingIntegrationService`.
struct RouteBindingIntegrationSink {
    client: edgegateway::route_binding_integration_service_client::RouteBindingIntegrationServiceClient<Channel>,
    context: common::RequestContext,
}

impl EnvironmentExpiryCodeDestination for RouteBindingIntegrationSink {
    fn forward(&self, record: EnvironmentExpiryCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::AcknowledgeRouteBindingEventRequest {
                    route_binding_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.acknowledge_route_binding_event(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct EnvironmentExpiryCodeLogOnly;

impl EnvironmentExpiryCodeDestination for EnvironmentExpiryCodeLogOnly {
    fn forward(&self, record: EnvironmentExpiryCodeRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.code);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_environment_expiry_code(
    kind: &str,
    record: EnvironmentExpiryCodeRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let api_key_workflow = ApiKeyWorkflowSink {
        client: edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient::new(channel.clone()),
    };
    let driver = DriverSink {
        client: drivers::driver_service_client::DriverServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let promotion_calculation = PromotionCalculationSink {
        client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let route_binding_integration = RouteBindingIntegrationSink {
        client: edgegateway::route_binding_integration_service_client::RouteBindingIntegrationServiceClient::new(
            channel,
        ),
        context: context.clone(),
    };
    let write: &dyn Fn(EnvironmentExpiryCodeRecord) -> Delivery = match kind {
        "api-key-workflow" => &|record| api_key_workflow.forward(record),
        "driver" => &|record| driver.forward(record),
        "promotion-calculation" => &|record| promotion_calculation.forward(record),
        "route-binding-integration" => &|record| route_binding_integration.forward(record),
        _ => &|record| EnvironmentExpiryCodeLogOnly.forward(record),
    };
    write(record).await
}
