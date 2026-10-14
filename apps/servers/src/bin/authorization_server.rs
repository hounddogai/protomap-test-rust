//! Composition root of the authorization domain.

use acme_proto::acme::authorization::v1::access_policy_admin_service_server::AccessPolicyAdminServiceServer;
use acme_proto::acme::authorization::v1::access_policy_history_service_server::AccessPolicyHistoryServiceServer;
use acme_proto::acme::authorization::v1::access_policy_notification_service_server::AccessPolicyNotificationServiceServer;
use acme_proto::acme::authorization::v1::access_policy_query_service_server::AccessPolicyQueryServiceServer;
use acme_proto::acme::authorization::v1::access_policy_service_server::AccessPolicyServiceServer;
use acme_proto::acme::authorization::v1::access_policy_validation_service_server::AccessPolicyValidationServiceServer;
use acme_proto::acme::authorization::v1::permission_check_admin_service_server::PermissionCheckAdminServiceServer;
use acme_proto::acme::authorization::v1::permission_check_export_service_server::PermissionCheckExportServiceServer;
use acme_proto::acme::authorization::v1::permission_check_history_service_server::PermissionCheckHistoryServiceServer;
use acme_proto::acme::authorization::v1::permission_check_notification_service_server::PermissionCheckNotificationServiceServer;
use acme_proto::acme::authorization::v1::permission_check_validation_service_server::PermissionCheckValidationServiceServer;
use acme_proto::acme::authorization::v1::role_binding_admin_service_server::RoleBindingAdminServiceServer;
use acme_proto::acme::authorization::v1::role_binding_export_service_server::RoleBindingExportServiceServer;
use acme_proto::acme::authorization::v1::role_binding_history_service_server::RoleBindingHistoryServiceServer;
use acme_proto::acme::authorization::v1::role_binding_notification_service_server::RoleBindingNotificationServiceServer;
use acme_proto::acme::authorization::v1::role_binding_query_service_server::RoleBindingQueryServiceServer;
use acme_proto::acme::authorization::v1::role_binding_validation_service_server::RoleBindingValidationServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let config = acme_platform::config::Config::load(5100)?;
    let channel = config.channel()?;
    let journal = acme_platform::journal::Journal::connect(&config.database_url).await?;
    journal.migrate().await?;
    let repository = std::sync::Arc::new(acme_authorization::repository::AuthorizationRepository::new(journal));
    let auth = acme_platform::auth::IdentityLayer::new(channel.clone());
    tonic::transport::Server::builder().layer(auth)
.add_service(AccessPolicyAdminServiceServer::with_interceptor(
acme_authorization::endpoints::access_policy_admin_service::AccessPolicyAdminEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(AccessPolicyHistoryServiceServer::with_interceptor(
acme_authorization::endpoints::access_policy_history_service::AccessPolicyHistoryHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(AccessPolicyNotificationServiceServer::with_interceptor(
acme_authorization::endpoints::access_policy_notification_service::AccessPolicyNotificationController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(AccessPolicyQueryServiceServer::with_interceptor(
acme_authorization::endpoints::access_policy_query_service::AccessPolicyQueryApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(AccessPolicyServiceServer::with_interceptor(
acme_authorization::endpoints::access_policy_service::AccessPolicyEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(AccessPolicyValidationServiceServer::with_interceptor(
acme_authorization::endpoints::access_policy_validation_service::AccessPolicyValidationHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(PermissionCheckAdminServiceServer::with_interceptor(
acme_authorization::endpoints::permission_check_admin_service::PermissionCheckAdminController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(PermissionCheckExportServiceServer::with_interceptor(
acme_authorization::endpoints::permission_check_export_service::PermissionCheckExportApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(PermissionCheckHistoryServiceServer::with_interceptor(
acme_authorization::endpoints::permission_check_history_service::PermissionCheckHistoryEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(PermissionCheckNotificationServiceServer::with_interceptor(
acme_authorization::endpoints::permission_check_notification_service::PermissionCheckNotificationHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(PermissionCheckValidationServiceServer::with_interceptor(
acme_authorization::endpoints::permission_check_validation_service::PermissionCheckValidationController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(RoleBindingAdminServiceServer::with_interceptor(
acme_authorization::endpoints::role_binding_admin_service::RoleBindingAdminApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(RoleBindingExportServiceServer::with_interceptor(
acme_authorization::endpoints::role_binding_export_service::RoleBindingExportEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(RoleBindingHistoryServiceServer::with_interceptor(
acme_authorization::endpoints::role_binding_history_service::RoleBindingHistoryHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(RoleBindingNotificationServiceServer::with_interceptor(
acme_authorization::endpoints::role_binding_notification_service::RoleBindingNotificationController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(RoleBindingQueryServiceServer::with_interceptor(
acme_authorization::endpoints::role_binding_query_service::RoleBindingQueryApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(RoleBindingValidationServiceServer::with_interceptor(
acme_authorization::endpoints::role_binding_validation_service::RoleBindingValidationEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.serve_with_shutdown(config.listen, async {
if let Err(error) = tokio::signal::ctrl_c().await { tracing::error!(%error, "shutdown signal failed"); }
}).await?;
    Ok(())
}
