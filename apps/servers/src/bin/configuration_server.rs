//! Composition root of the configuration domain.

use acme_proto::acme::configuration::v1::config_change_approval_service_server::ConfigChangeApprovalServiceServer;
use acme_proto::acme::configuration::v1::config_change_calculation_service_server::ConfigChangeCalculationServiceServer;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_server::ConfigChangeLifecycleServiceServer;
use acme_proto::acme::configuration::v1::config_change_query_service_server::ConfigChangeQueryServiceServer;
use acme_proto::acme::configuration::v1::config_change_reporting_service_server::ConfigChangeReportingServiceServer;
use acme_proto::acme::configuration::v1::config_change_validation_service_server::ConfigChangeValidationServiceServer;
use acme_proto::acme::configuration::v1::config_change_workflow_service_server::ConfigChangeWorkflowServiceServer;
use acme_proto::acme::configuration::v1::config_set_approval_service_server::ConfigSetApprovalServiceServer;
use acme_proto::acme::configuration::v1::config_set_calculation_service_server::ConfigSetCalculationServiceServer;
use acme_proto::acme::configuration::v1::config_set_lifecycle_service_server::ConfigSetLifecycleServiceServer;
use acme_proto::acme::configuration::v1::config_set_query_service_server::ConfigSetQueryServiceServer;
use acme_proto::acme::configuration::v1::config_set_reporting_service_server::ConfigSetReportingServiceServer;
use acme_proto::acme::configuration::v1::config_set_service_server::ConfigSetServiceServer;
use acme_proto::acme::configuration::v1::config_set_validation_service_server::ConfigSetValidationServiceServer;
use acme_proto::acme::configuration::v1::config_set_workflow_service_server::ConfigSetWorkflowServiceServer;
use acme_proto::acme::configuration::v1::environment_approval_service_server::EnvironmentApprovalServiceServer;
use acme_proto::acme::configuration::v1::environment_calculation_service_server::EnvironmentCalculationServiceServer;
use acme_proto::acme::configuration::v1::environment_lifecycle_service_server::EnvironmentLifecycleServiceServer;
use acme_proto::acme::configuration::v1::environment_reporting_service_server::EnvironmentReportingServiceServer;
use acme_proto::acme::configuration::v1::environment_sync_service_server::EnvironmentSyncServiceServer;
use acme_proto::acme::configuration::v1::environment_validation_service_server::EnvironmentValidationServiceServer;
use acme_proto::acme::configuration::v1::environment_workflow_service_server::EnvironmentWorkflowServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let config = acme_platform::config::Config::load(5102)?;
    let channel = config.channel()?;
    let journal = acme_platform::journal::Journal::connect(&config.database_url).await?;
    journal.migrate().await?;
    let repository = std::sync::Arc::new(acme_configuration::repository::ConfigurationRepository::new(journal));
    let auth = acme_platform::auth::IdentityLayer::new(channel.clone());
    tonic::transport::Server::builder()
        .layer(auth)
        .add_service(ConfigChangeApprovalServiceServer::with_interceptor(
            acme_configuration::application::config_change_approval_service::ConfigChangeApprovalHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigChangeCalculationServiceServer::with_interceptor(
            acme_configuration::application::config_change_calculation_service::ConfigChangeCalculationController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigChangeLifecycleServiceServer::with_interceptor(
            acme_configuration::application::config_change_lifecycle_service::ConfigChangeLifecycleApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigChangeQueryServiceServer::with_interceptor(
            acme_configuration::application::config_change_query_service::ConfigChangeQueryEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigChangeReportingServiceServer::with_interceptor(
            acme_configuration::application::config_change_reporting_service::ConfigChangeReportingHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigChangeValidationServiceServer::with_interceptor(
            acme_configuration::application::config_change_validation_service::ConfigChangeValidationController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigChangeWorkflowServiceServer::with_interceptor(
            acme_configuration::application::config_change_workflow_service::ConfigChangeWorkflowApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigSetApprovalServiceServer::with_interceptor(
            acme_configuration::application::config_set_approval_service::ConfigSetApprovalEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigSetCalculationServiceServer::with_interceptor(
            acme_configuration::application::config_set_calculation_service::ConfigSetCalculationHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigSetLifecycleServiceServer::with_interceptor(
            acme_configuration::application::config_set_lifecycle_service::ConfigSetLifecycleController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigSetQueryServiceServer::with_interceptor(
            acme_configuration::application::config_set_query_service::ConfigSetQueryApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigSetReportingServiceServer::with_interceptor(
            acme_configuration::application::config_set_reporting_service::ConfigSetReportingEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigSetServiceServer::with_interceptor(
            acme_configuration::application::config_set_service::ConfigSetHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigSetValidationServiceServer::with_interceptor(
            acme_configuration::application::config_set_validation_service::ConfigSetValidationController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ConfigSetWorkflowServiceServer::with_interceptor(
            acme_configuration::application::config_set_workflow_service::ConfigSetWorkflowApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(EnvironmentApprovalServiceServer::with_interceptor(
            acme_configuration::application::environment_approval_service::EnvironmentApprovalEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(EnvironmentCalculationServiceServer::with_interceptor(
            acme_configuration::application::environment_calculation_service::EnvironmentCalculationHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(EnvironmentLifecycleServiceServer::with_interceptor(
            acme_configuration::application::environment_lifecycle_service::EnvironmentLifecycleController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(EnvironmentReportingServiceServer::with_interceptor(
            acme_configuration::application::environment_reporting_service::EnvironmentReportingApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(EnvironmentSyncServiceServer::with_interceptor(
            acme_configuration::application::environment_sync_service::EnvironmentSyncEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(EnvironmentValidationServiceServer::with_interceptor(
            acme_configuration::application::environment_validation_service::EnvironmentValidationHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(EnvironmentWorkflowServiceServer::with_interceptor(
            acme_configuration::application::environment_workflow_service::EnvironmentWorkflowController::new(
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
