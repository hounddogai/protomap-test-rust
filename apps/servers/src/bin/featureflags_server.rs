//! Composition root of the featureflags domain.

use acme_proto::acme::featureflags::v1::feature_flag_admin_service_server::FeatureFlagAdminServiceServer;
use acme_proto::acme::featureflags::v1::feature_flag_export_service_server::FeatureFlagExportServiceServer;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_server::FeatureFlagHistoryServiceServer;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_server::FeatureFlagNotificationServiceServer;
use acme_proto::acme::featureflags::v1::feature_flag_query_service_server::FeatureFlagQueryServiceServer;
use acme_proto::acme::featureflags::v1::feature_flag_service_server::FeatureFlagServiceServer;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_server::FeatureFlagSyncServiceServer;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_server::FeatureFlagValidationServiceServer;
use acme_proto::acme::featureflags::v1::flag_evaluation_admin_service_server::FlagEvaluationAdminServiceServer;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_server::FlagEvaluationCalculationServiceServer;
use acme_proto::acme::featureflags::v1::flag_evaluation_export_service_server::FlagEvaluationExportServiceServer;
use acme_proto::acme::featureflags::v1::flag_evaluation_history_service_server::FlagEvaluationHistoryServiceServer;
use acme_proto::acme::featureflags::v1::flag_evaluation_notification_service_server::FlagEvaluationNotificationServiceServer;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_server::FlagEvaluationSyncServiceServer;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_server::FlagEvaluationValidationServiceServer;
use acme_proto::acme::featureflags::v1::flag_rollout_admin_service_server::FlagRolloutAdminServiceServer;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_server::FlagRolloutCalculationServiceServer;
use acme_proto::acme::featureflags::v1::flag_rollout_export_service_server::FlagRolloutExportServiceServer;
use acme_proto::acme::featureflags::v1::flag_rollout_history_service_server::FlagRolloutHistoryServiceServer;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_server::FlagRolloutNotificationServiceServer;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_server::FlagRolloutQueryServiceServer;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_server::FlagRolloutSyncServiceServer;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_server::FlagRolloutValidationServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let config = acme_platform::config::Config::load(5104)?;
    let channel = config.channel()?;
    let journal = acme_platform::journal::Journal::connect(&config.database_url).await?;
    journal.migrate().await?;
    let repository = std::sync::Arc::new(acme_featureflags::repository::FeatureflagsRepository::new(journal));
    let auth = acme_platform::auth::IdentityLayer::new(channel.clone());
    tonic::transport::Server::builder().layer(auth)
.add_service(FeatureFlagAdminServiceServer::with_interceptor(
acme_featureflags::endpoints::feature_flag_admin_service::FeatureFlagAdminController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FeatureFlagExportServiceServer::with_interceptor(
acme_featureflags::endpoints::feature_flag_export_service::FeatureFlagExportApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FeatureFlagHistoryServiceServer::with_interceptor(
acme_featureflags::endpoints::feature_flag_history_service::FeatureFlagHistoryEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FeatureFlagNotificationServiceServer::with_interceptor(
acme_featureflags::endpoints::feature_flag_notification_service::FeatureFlagNotificationHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FeatureFlagQueryServiceServer::with_interceptor(
acme_featureflags::endpoints::feature_flag_query_service::FeatureFlagQueryController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FeatureFlagServiceServer::with_interceptor(
acme_featureflags::endpoints::feature_flag_service::FeatureFlagApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FeatureFlagSyncServiceServer::with_interceptor(
acme_featureflags::endpoints::feature_flag_sync_service::FeatureFlagSyncEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FeatureFlagValidationServiceServer::with_interceptor(
acme_featureflags::endpoints::feature_flag_validation_service::FeatureFlagValidationHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagEvaluationAdminServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_evaluation_admin_service::FlagEvaluationAdminController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagEvaluationCalculationServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_evaluation_calculation_service::FlagEvaluationCalculationApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagEvaluationExportServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_evaluation_export_service::FlagEvaluationExportEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagEvaluationHistoryServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_evaluation_history_service::FlagEvaluationHistoryHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagEvaluationNotificationServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_evaluation_notification_service::FlagEvaluationNotificationController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagEvaluationSyncServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_evaluation_sync_service::FlagEvaluationSyncApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagEvaluationValidationServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_evaluation_validation_service::FlagEvaluationValidationEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagRolloutAdminServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_rollout_admin_service::FlagRolloutAdminHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagRolloutCalculationServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_rollout_calculation_service::FlagRolloutCalculationController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagRolloutExportServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_rollout_export_service::FlagRolloutExportApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagRolloutHistoryServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_rollout_history_service::FlagRolloutHistoryEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagRolloutNotificationServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_rollout_notification_service::FlagRolloutNotificationHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagRolloutQueryServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_rollout_query_service::FlagRolloutQueryController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagRolloutSyncServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_rollout_sync_service::FlagRolloutSyncApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(FlagRolloutValidationServiceServer::with_interceptor(
acme_featureflags::endpoints::flag_rollout_validation_service::FlagRolloutValidationEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.serve_with_shutdown(config.listen, async {
if let Err(error) = tokio::signal::ctrl_c().await { tracing::error!(%error, "shutdown signal failed"); }
}).await?;
    Ok(())
}
