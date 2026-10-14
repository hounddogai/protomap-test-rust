//! Composition root of the secrets domain.

use acme_proto::acme::secrets::v1::access_grant_approval_service_server::AccessGrantApprovalServiceServer;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_server::AccessGrantCalculationServiceServer;
use acme_proto::acme::secrets::v1::access_grant_validation_service_server::AccessGrantValidationServiceServer;
use acme_proto::acme::secrets::v1::access_grant_workflow_service_server::AccessGrantWorkflowServiceServer;
use acme_proto::acme::secrets::v1::secret_approval_service_server::SecretApprovalServiceServer;
use acme_proto::acme::secrets::v1::secret_query_service_server::SecretQueryServiceServer;
use acme_proto::acme::secrets::v1::secret_service_server::SecretServiceServer;
use acme_proto::acme::secrets::v1::secret_version_approval_service_server::SecretVersionApprovalServiceServer;
use acme_proto::acme::secrets::v1::secret_version_query_service_server::SecretVersionQueryServiceServer;
use acme_proto::acme::secrets::v1::secret_version_validation_service_server::SecretVersionValidationServiceServer;
use acme_proto::acme::secrets::v1::secret_version_workflow_service_server::SecretVersionWorkflowServiceServer;
use acme_proto::acme::secrets::v1::secret_workflow_service_server::SecretWorkflowServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let config = acme_platform::config::Config::load(5107)?;
    let channel = config.channel()?;
    let journal = acme_platform::journal::Journal::connect(&config.database_url).await?;
    journal.migrate().await?;
    let repository = std::sync::Arc::new(acme_secrets::repository::SecretsRepository::new(journal));
    let auth = acme_platform::auth::IdentityLayer::new(channel.clone());
    tonic::transport::Server::builder()
        .layer(auth)
        .add_service(AccessGrantApprovalServiceServer::with_interceptor(
            acme_secrets::transport::access_grant_approval_service::AccessGrantApprovalController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(AccessGrantCalculationServiceServer::with_interceptor(
            acme_secrets::transport::access_grant_calculation_service::AccessGrantCalculationApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(AccessGrantValidationServiceServer::with_interceptor(
            acme_secrets::transport::access_grant_validation_service::AccessGrantValidationEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(AccessGrantWorkflowServiceServer::with_interceptor(
            acme_secrets::transport::access_grant_workflow_service::AccessGrantWorkflowHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(SecretApprovalServiceServer::with_interceptor(
            acme_secrets::transport::secret_approval_service::SecretApprovalController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(SecretQueryServiceServer::with_interceptor(
            acme_secrets::transport::secret_query_service::SecretQueryApi::new(channel.clone(), repository.clone()),
            acme_platform::auth::require_identity,
        ))
        .add_service(SecretServiceServer::with_interceptor(
            acme_secrets::transport::secret_service::SecretEndpoint::new(channel.clone(), repository.clone()),
            acme_platform::auth::require_identity,
        ))
        .add_service(SecretVersionApprovalServiceServer::with_interceptor(
            acme_secrets::transport::secret_version_approval_service::SecretVersionApprovalHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(SecretVersionQueryServiceServer::with_interceptor(
            acme_secrets::transport::secret_version_query_service::SecretVersionQueryController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(SecretVersionValidationServiceServer::with_interceptor(
            acme_secrets::transport::secret_version_validation_service::SecretVersionValidationApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(SecretVersionWorkflowServiceServer::with_interceptor(
            acme_secrets::transport::secret_version_workflow_service::SecretVersionWorkflowEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(SecretWorkflowServiceServer::with_interceptor(
            acme_secrets::transport::secret_workflow_service::SecretWorkflowHandler::new(
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
