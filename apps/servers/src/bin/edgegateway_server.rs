//! Composition root of the edgegateway domain.

use acme_proto::acme::edgegateway::v1::api_key_integration_service_server::ApiKeyIntegrationServiceServer;
use acme_proto::acme::edgegateway::v1::api_key_notification_service_server::ApiKeyNotificationServiceServer;
use acme_proto::acme::edgegateway::v1::api_key_query_service_server::ApiKeyQueryServiceServer;
use acme_proto::acme::edgegateway::v1::api_key_service_server::ApiKeyServiceServer;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_server::ApiKeyWorkflowServiceServer;
use acme_proto::acme::edgegateway::v1::gateway_policy_integration_service_server::GatewayPolicyIntegrationServiceServer;
use acme_proto::acme::edgegateway::v1::gateway_policy_notification_service_server::GatewayPolicyNotificationServiceServer;
use acme_proto::acme::edgegateway::v1::gateway_policy_query_service_server::GatewayPolicyQueryServiceServer;
use acme_proto::acme::edgegateway::v1::gateway_policy_reporting_service_server::GatewayPolicyReportingServiceServer;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_server::GatewayPolicyWorkflowServiceServer;
use acme_proto::acme::edgegateway::v1::route_binding_gateway_service_server::RouteBindingGatewayServiceServer;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_server::RouteBindingIntegrationServiceServer;
use acme_proto::acme::edgegateway::v1::route_binding_notification_service_server::RouteBindingNotificationServiceServer;
use acme_proto::acme::edgegateway::v1::route_binding_query_service_server::RouteBindingQueryServiceServer;
use acme_proto::acme::edgegateway::v1::route_binding_service_server::RouteBindingServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let config = acme_platform::config::Config::load(5103)?;
    let channel = config.channel()?;
    let journal = acme_platform::journal::Journal::connect(&config.database_url).await?;
    journal.migrate().await?;
    let repository = std::sync::Arc::new(acme_edgegateway::repository::EdgegatewayRepository::new(journal));
    let auth = acme_platform::auth::IdentityLayer::new(channel.clone());
    tonic::transport::Server::builder()
        .layer(auth)
        .add_service(ApiKeyIntegrationServiceServer::with_interceptor(
            acme_edgegateway::transport::api_key_integration_service::ApiKeyIntegrationApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ApiKeyNotificationServiceServer::with_interceptor(
            acme_edgegateway::transport::api_key_notification_service::ApiKeyNotificationEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ApiKeyQueryServiceServer::with_interceptor(
            acme_edgegateway::transport::api_key_query_service::ApiKeyQueryHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ApiKeyServiceServer::with_interceptor(
            acme_edgegateway::transport::api_key_service::ApiKeyController::new(channel.clone(), repository.clone()),
            acme_platform::auth::require_identity,
        ))
        .add_service(ApiKeyWorkflowServiceServer::with_interceptor(
            acme_edgegateway::transport::api_key_workflow_service::ApiKeyWorkflowApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(GatewayPolicyIntegrationServiceServer::with_interceptor(
            acme_edgegateway::transport::gateway_policy_integration_service::GatewayPolicyIntegrationEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(GatewayPolicyNotificationServiceServer::with_interceptor(
            acme_edgegateway::transport::gateway_policy_notification_service::GatewayPolicyNotificationHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(GatewayPolicyQueryServiceServer::with_interceptor(
            acme_edgegateway::transport::gateway_policy_query_service::GatewayPolicyQueryController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(GatewayPolicyReportingServiceServer::with_interceptor(
            acme_edgegateway::transport::gateway_policy_reporting_service::GatewayPolicyReportingApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(GatewayPolicyWorkflowServiceServer::with_interceptor(
            acme_edgegateway::transport::gateway_policy_workflow_service::GatewayPolicyWorkflowEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RouteBindingGatewayServiceServer::with_interceptor(
            acme_edgegateway::transport::route_binding_gateway_service::RouteBindingGatewayHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RouteBindingIntegrationServiceServer::with_interceptor(
            acme_edgegateway::transport::route_binding_integration_service::RouteBindingIntegrationController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RouteBindingNotificationServiceServer::with_interceptor(
            acme_edgegateway::transport::route_binding_notification_service::RouteBindingNotificationApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RouteBindingQueryServiceServer::with_interceptor(
            acme_edgegateway::transport::route_binding_query_service::RouteBindingQueryEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RouteBindingServiceServer::with_interceptor(
            acme_edgegateway::transport::route_binding_service::RouteBindingHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .serve_with_shutdown(config.listen, async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                tracing::error!(%error, "shutdown signal failed");
            }
        })
        .await?;
    Ok(())
}
