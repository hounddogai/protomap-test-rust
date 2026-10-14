//! Composition root of the keymanagement domain.

use acme_proto::acme::keymanagement::v1::crypto_key_approval_service_server::CryptoKeyApprovalServiceServer;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_server::CryptoKeyCalculationServiceServer;
use acme_proto::acme::keymanagement::v1::crypto_key_lifecycle_service_server::CryptoKeyLifecycleServiceServer;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_server::CryptoKeyQueryServiceServer;
use acme_proto::acme::keymanagement::v1::crypto_key_service_server::CryptoKeyServiceServer;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_server::CryptoKeyValidationServiceServer;
use acme_proto::acme::keymanagement::v1::crypto_key_workflow_service_server::CryptoKeyWorkflowServiceServer;
use acme_proto::acme::keymanagement::v1::key_ring_approval_service_server::KeyRingApprovalServiceServer;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_server::KeyRingCalculationServiceServer;
use acme_proto::acme::keymanagement::v1::key_ring_lifecycle_service_server::KeyRingLifecycleServiceServer;
use acme_proto::acme::keymanagement::v1::key_ring_query_service_server::KeyRingQueryServiceServer;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_server::KeyRingReportingServiceServer;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_server::KeyRingValidationServiceServer;
use acme_proto::acme::keymanagement::v1::key_ring_workflow_service_server::KeyRingWorkflowServiceServer;
use acme_proto::acme::keymanagement::v1::key_rotation_approval_service_server::KeyRotationApprovalServiceServer;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_server::KeyRotationCalculationServiceServer;
use acme_proto::acme::keymanagement::v1::key_rotation_lifecycle_service_server::KeyRotationLifecycleServiceServer;
use acme_proto::acme::keymanagement::v1::key_rotation_reporting_service_server::KeyRotationReportingServiceServer;
use acme_proto::acme::keymanagement::v1::key_rotation_validation_service_server::KeyRotationValidationServiceServer;
use acme_proto::acme::keymanagement::v1::key_rotation_workflow_service_server::KeyRotationWorkflowServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let config = acme_platform::config::Config::load(5105)?;
    let channel = config.channel()?;
    let journal = acme_platform::journal::Journal::connect(&config.database_url).await?;
    journal.migrate().await?;
    let repository = std::sync::Arc::new(acme_keymanagement::repository::KeymanagementRepository::new(journal));
    let auth = acme_platform::auth::IdentityLayer::new(channel.clone());
    tonic::transport::Server::builder()
        .layer(auth)
        .add_service(CryptoKeyApprovalServiceServer::with_interceptor(
            acme_keymanagement::services::crypto_key_approval_service::CryptoKeyApprovalHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(CryptoKeyCalculationServiceServer::with_interceptor(
            acme_keymanagement::services::crypto_key_calculation_service::CryptoKeyCalculationController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(CryptoKeyLifecycleServiceServer::with_interceptor(
            acme_keymanagement::services::crypto_key_lifecycle_service::CryptoKeyLifecycleApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(CryptoKeyQueryServiceServer::with_interceptor(
            acme_keymanagement::services::crypto_key_query_service::CryptoKeyQueryEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(CryptoKeyServiceServer::with_interceptor(
            acme_keymanagement::services::crypto_key_service::CryptoKeyHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(CryptoKeyValidationServiceServer::with_interceptor(
            acme_keymanagement::services::crypto_key_validation_service::CryptoKeyValidationController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(CryptoKeyWorkflowServiceServer::with_interceptor(
            acme_keymanagement::services::crypto_key_workflow_service::CryptoKeyWorkflowApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRingApprovalServiceServer::with_interceptor(
            acme_keymanagement::services::key_ring_approval_service::KeyRingApprovalEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRingCalculationServiceServer::with_interceptor(
            acme_keymanagement::services::key_ring_calculation_service::KeyRingCalculationHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRingLifecycleServiceServer::with_interceptor(
            acme_keymanagement::services::key_ring_lifecycle_service::KeyRingLifecycleController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRingQueryServiceServer::with_interceptor(
            acme_keymanagement::services::key_ring_query_service::KeyRingQueryApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRingReportingServiceServer::with_interceptor(
            acme_keymanagement::services::key_ring_reporting_service::KeyRingReportingEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRingValidationServiceServer::with_interceptor(
            acme_keymanagement::services::key_ring_validation_service::KeyRingValidationHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRingWorkflowServiceServer::with_interceptor(
            acme_keymanagement::services::key_ring_workflow_service::KeyRingWorkflowController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRotationApprovalServiceServer::with_interceptor(
            acme_keymanagement::services::key_rotation_approval_service::KeyRotationApprovalApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRotationCalculationServiceServer::with_interceptor(
            acme_keymanagement::services::key_rotation_calculation_service::KeyRotationCalculationEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRotationLifecycleServiceServer::with_interceptor(
            acme_keymanagement::services::key_rotation_lifecycle_service::KeyRotationLifecycleHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRotationReportingServiceServer::with_interceptor(
            acme_keymanagement::services::key_rotation_reporting_service::KeyRotationReportingController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRotationValidationServiceServer::with_interceptor(
            acme_keymanagement::services::key_rotation_validation_service::KeyRotationValidationApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(KeyRotationWorkflowServiceServer::with_interceptor(
            acme_keymanagement::services::key_rotation_workflow_service::KeyRotationWorkflowEndpoint::new(
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
