//! Composition root of the ratelimiting domain.

use acme_proto::acme::ratelimiting::v1::quota_bucket_approval_service_server::QuotaBucketApprovalServiceServer;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_server::QuotaBucketCalculationServiceServer;
use acme_proto::acme::ratelimiting::v1::quota_bucket_lifecycle_service_server::QuotaBucketLifecycleServiceServer;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_server::QuotaBucketQueryServiceServer;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_server::QuotaBucketReportingServiceServer;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_server::QuotaBucketSyncServiceServer;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_server::QuotaBucketValidationServiceServer;
use acme_proto::acme::ratelimiting::v1::quota_bucket_workflow_service_server::QuotaBucketWorkflowServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_approval_service_server::RateLimitApprovalServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_server::RateLimitCalculationServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_lifecycle_service_server::RateLimitLifecycleServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_server::RateLimitQueryServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_server::RateLimitReportingServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_server::RateLimitServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_server::RateLimitSyncServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_server::RateLimitValidationServiceServer;
use acme_proto::acme::ratelimiting::v1::rate_limit_workflow_service_server::RateLimitWorkflowServiceServer;
use acme_proto::acme::ratelimiting::v1::throttle_approval_service_server::ThrottleApprovalServiceServer;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_server::ThrottleCalculationServiceServer;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_server::ThrottleImportServiceServer;
use acme_proto::acme::ratelimiting::v1::throttle_lifecycle_service_server::ThrottleLifecycleServiceServer;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_server::ThrottleReportingServiceServer;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_server::ThrottleSyncServiceServer;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_server::ThrottleValidationServiceServer;
use acme_proto::acme::ratelimiting::v1::throttle_workflow_service_server::ThrottleWorkflowServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let config = acme_platform::config::Config::load(5106)?;
    let channel = config.channel()?;
    let journal = acme_platform::journal::Journal::connect(&config.database_url).await?;
    journal.migrate().await?;
    let repository = std::sync::Arc::new(acme_ratelimiting::repository::RatelimitingRepository::new(journal));
    let auth = acme_platform::auth::IdentityLayer::new(channel.clone());
    tonic::transport::Server::builder()
        .layer(auth)
        .add_service(QuotaBucketApprovalServiceServer::with_interceptor(
            acme_ratelimiting::application::quota_bucket_approval_service::QuotaBucketApprovalHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(QuotaBucketCalculationServiceServer::with_interceptor(
            acme_ratelimiting::application::quota_bucket_calculation_service::QuotaBucketCalculationController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(QuotaBucketLifecycleServiceServer::with_interceptor(
            acme_ratelimiting::application::quota_bucket_lifecycle_service::QuotaBucketLifecycleApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(QuotaBucketQueryServiceServer::with_interceptor(
            acme_ratelimiting::application::quota_bucket_query_service::QuotaBucketQueryEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(QuotaBucketReportingServiceServer::with_interceptor(
            acme_ratelimiting::application::quota_bucket_reporting_service::QuotaBucketReportingHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(QuotaBucketSyncServiceServer::with_interceptor(
            acme_ratelimiting::application::quota_bucket_sync_service::QuotaBucketSyncController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(QuotaBucketValidationServiceServer::with_interceptor(
            acme_ratelimiting::application::quota_bucket_validation_service::QuotaBucketValidationApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(QuotaBucketWorkflowServiceServer::with_interceptor(
            acme_ratelimiting::application::quota_bucket_workflow_service::QuotaBucketWorkflowEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitApprovalServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_approval_service::RateLimitApprovalHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitCalculationServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_calculation_service::RateLimitCalculationController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitLifecycleServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_lifecycle_service::RateLimitLifecycleApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitQueryServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_query_service::RateLimitQueryEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitReportingServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_reporting_service::RateLimitReportingHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_service::RateLimitController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitSyncServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_sync_service::RateLimitSyncApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitValidationServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_validation_service::RateLimitValidationEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(RateLimitWorkflowServiceServer::with_interceptor(
            acme_ratelimiting::application::rate_limit_workflow_service::RateLimitWorkflowHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ThrottleApprovalServiceServer::with_interceptor(
            acme_ratelimiting::application::throttle_approval_service::ThrottleApprovalController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ThrottleCalculationServiceServer::with_interceptor(
            acme_ratelimiting::application::throttle_calculation_service::ThrottleCalculationApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ThrottleImportServiceServer::with_interceptor(
            acme_ratelimiting::application::throttle_import_service::ThrottleImportEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ThrottleLifecycleServiceServer::with_interceptor(
            acme_ratelimiting::application::throttle_lifecycle_service::ThrottleLifecycleHandler::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ThrottleReportingServiceServer::with_interceptor(
            acme_ratelimiting::application::throttle_reporting_service::ThrottleReportingController::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ThrottleSyncServiceServer::with_interceptor(
            acme_ratelimiting::application::throttle_sync_service::ThrottleSyncApi::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ThrottleValidationServiceServer::with_interceptor(
            acme_ratelimiting::application::throttle_validation_service::ThrottleValidationEndpoint::new(
                channel.clone(),
                repository.clone(),
            ),
            acme_platform::auth::require_identity,
        ))
        .add_service(ThrottleWorkflowServiceServer::with_interceptor(
            acme_ratelimiting::application::throttle_workflow_service::ThrottleWorkflowHandler::new(
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
