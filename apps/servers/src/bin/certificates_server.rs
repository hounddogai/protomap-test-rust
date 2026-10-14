//! Composition root of the certificates domain.

use acme_proto::acme::certificates::v1::certificate_approval_service_server::CertificateApprovalServiceServer;
use acme_proto::acme::certificates::v1::certificate_authority_approval_service_server::CertificateAuthorityApprovalServiceServer;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_server::CertificateAuthorityCalculationServiceServer;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_server::CertificateAuthorityQueryServiceServer;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_server::CertificateAuthorityValidationServiceServer;
use acme_proto::acme::certificates::v1::certificate_authority_workflow_service_server::CertificateAuthorityWorkflowServiceServer;
use acme_proto::acme::certificates::v1::certificate_calculation_service_server::CertificateCalculationServiceServer;
use acme_proto::acme::certificates::v1::certificate_query_service_server::CertificateQueryServiceServer;
use acme_proto::acme::certificates::v1::certificate_service_server::CertificateServiceServer;
use acme_proto::acme::certificates::v1::certificate_validation_service_server::CertificateValidationServiceServer;
use acme_proto::acme::certificates::v1::certificate_workflow_service_server::CertificateWorkflowServiceServer;
use acme_proto::acme::certificates::v1::csr_approval_service_server::CsrApprovalServiceServer;
use acme_proto::acme::certificates::v1::csr_calculation_service_server::CsrCalculationServiceServer;
use acme_proto::acme::certificates::v1::csr_lifecycle_service_server::CsrLifecycleServiceServer;
use acme_proto::acme::certificates::v1::csr_validation_service_server::CsrValidationServiceServer;
use acme_proto::acme::certificates::v1::csr_workflow_service_server::CsrWorkflowServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let config = acme_platform::config::Config::load(5101)?;
    let channel = config.channel()?;
    let journal = acme_platform::journal::Journal::connect(&config.database_url).await?;
    journal.migrate().await?;
    let repository = std::sync::Arc::new(acme_certificates::repository::CertificatesRepository::new(journal));
    let auth = acme_platform::auth::IdentityLayer::new(channel.clone());
    tonic::transport::Server::builder().layer(auth)
.add_service(CertificateApprovalServiceServer::with_interceptor(
acme_certificates::services::certificate_approval_service::CertificateApprovalHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateAuthorityApprovalServiceServer::with_interceptor(
acme_certificates::services::certificate_authority_approval_service::CertificateAuthorityApprovalController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateAuthorityCalculationServiceServer::with_interceptor(
acme_certificates::services::certificate_authority_calculation_service::CertificateAuthorityCalculationApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateAuthorityQueryServiceServer::with_interceptor(
acme_certificates::services::certificate_authority_query_service::CertificateAuthorityQueryEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateAuthorityValidationServiceServer::with_interceptor(
acme_certificates::services::certificate_authority_validation_service::CertificateAuthorityValidationHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateAuthorityWorkflowServiceServer::with_interceptor(
acme_certificates::services::certificate_authority_workflow_service::CertificateAuthorityWorkflowController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateCalculationServiceServer::with_interceptor(
acme_certificates::services::certificate_calculation_service::CertificateCalculationApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateQueryServiceServer::with_interceptor(
acme_certificates::services::certificate_query_service::CertificateQueryEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateServiceServer::with_interceptor(
acme_certificates::services::certificate_service::CertificateHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateValidationServiceServer::with_interceptor(
acme_certificates::services::certificate_validation_service::CertificateValidationController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CertificateWorkflowServiceServer::with_interceptor(
acme_certificates::services::certificate_workflow_service::CertificateWorkflowApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CsrApprovalServiceServer::with_interceptor(
acme_certificates::services::csr_approval_service::CsrApprovalEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CsrCalculationServiceServer::with_interceptor(
acme_certificates::services::csr_calculation_service::CsrCalculationHandler::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CsrLifecycleServiceServer::with_interceptor(
acme_certificates::services::csr_lifecycle_service::CsrLifecycleController::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CsrValidationServiceServer::with_interceptor(
acme_certificates::services::csr_validation_service::CsrValidationApi::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.add_service(CsrWorkflowServiceServer::with_interceptor(
acme_certificates::services::csr_workflow_service::CsrWorkflowEndpoint::new(channel.clone(), repository.clone()), acme_platform::auth::require_identity))
.serve_with_shutdown(config.listen, async {
if let Err(error) = tokio::signal::ctrl_c().await { tracing::error!(%error, "shutdown signal failed"); }
}).await?;
    Ok(())
}
