use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::certificates::v1::csr_lifecycle_service_server::CsrLifecycleService;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::chat::v1::chat_room_service_client::ChatRoomServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_workflow_service_client::EnvironmentWorkflowServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_service_client::LedgerAccountServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_query_service_client::NotificationQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;

/// Moves a csr between the states of its lifecycle.
#[derive(Clone)]
pub struct CsrLifecycleController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    expire_csr_0_port: acme_platform::ports::Callback<
        ratelimiting::ShareRateLimitReportRequest,
        ratelimiting::ShareRateLimitReportResponse,
    >,
    expire_csr_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            certificates::ValidateCertificateAuthorityRequest,
            certificates::ValidateCertificateAuthorityResponse,
        >,
    >,
    get_csr_lifecycle_0_port: acme_platform::ports::Callback<
        notifications::FindNotificationsRequest,
        notifications::FindNotificationsResponse,
    >,
    terminate_csr_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<certificates::EstimateCsrRequest, certificates::EstimateCsrResponse>,
    >,
}
impl CsrLifecycleController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            certificates::EstimateCsrRequest,
            certificates::EstimateCsrResponse,
        > = {
            let client = CsrCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::EstimateCsrRequest| -> acme_platform::ports::RpcFuture<certificates::EstimateCsrResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_csr(request).await?.into_inner())
})
})
        };
        let terminate_csr_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::ShareRateLimitReportRequest,
            ratelimiting::ShareRateLimitReportResponse,
        > = {
            let client = RateLimitReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ShareRateLimitReportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ShareRateLimitReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_rate_limit_report(request).await?.into_inner())
})
})
        };
        let expire_csr_0_port = callback;
        let callback: acme_platform::ports::Callback<
            certificates::ValidateCertificateAuthorityRequest,
            certificates::ValidateCertificateAuthorityResponse,
        > = {
            let client = CertificateAuthorityValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::ValidateCertificateAuthorityRequest| -> acme_platform::ports::RpcFuture<certificates::ValidateCertificateAuthorityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.validate_certificate_authority(request).await?.into_inner())
})
})
        };
        let expire_csr_1_port = std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            notifications::FindNotificationsRequest,
            notifications::FindNotificationsResponse,
        > = {
            let client = NotificationQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: notifications::FindNotificationsRequest| -> acme_platform::ports::RpcFuture<notifications::FindNotificationsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.find_notifications(request).await?.into_inner())
})
})
        };
        let get_csr_lifecycle_0_port = callback;
        Self {
            channel,
            repository,
            expire_csr_0_port,
            expire_csr_1_port,
            get_csr_lifecycle_0_port,
            terminate_csr_0_port,
        }
    }
    /// Run ActivateCsr after validating the tenant and durably recording the command.
    async fn run_activate_csr(
        &self,
        req: certificates::ActivateCsrRequest,
    ) -> Result<certificates::ActivateCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ActivateCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ActivateCsr", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = certificates::ScoreCertificateRequest {
            certificate: Some(certificates::Certificate {
                counterparty: Some(certificates::CsrRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ScoreCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ListRateLimitReportsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_rate_limit_reports_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                owner_id: reply_0.certificate.clone().unwrap_or_default().certificate_id.clone(),
                region: reply_0
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .billing_address
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListRateLimitReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_rate_limit_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = chat::BatchCreateChatRoomsRequest {
            context: req.context.clone(),
            chunk: Some(chat::batch_create_chat_rooms_request::Chunk::Header(
                chat::batch_create_chat_rooms_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .new_owner
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        phone: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify_contact
                            .clone()
                            .unwrap_or_default()
                            .phone
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ChatRoomServiceClient::new(channel);
            let input: tonic::Request<chat::BatchCreateChatRoomsRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_chat_rooms(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::StreamThrottleChangesRequest {
            frame: Some(ratelimiting::stream_throttle_changes_request::Frame::Change(
                ratelimiting::Throttle {
                    correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                    occurred_at: reply_0.certificate.clone().unwrap_or_default().issued_at.clone(),
                    subject: Some(ratelimiting::RateLimitRef {
                        display_name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .new_owner
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamThrottleChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_throttle_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamThrottleChangesResponse::default();
            // The caller asks for a bounded snapshot, then drops its subscription.
            for _ in 0..8 {
                match frames.message().await? {
                    Some(frame) => last = frame,
                    None => break,
                }
            }
            Ok::<_, tonic::Status>(last)
        }
        .await?;
        let mut response = certificates::ActivateCsrResponse::default();
        let contribution: String = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .assignee
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run SuspendCsr after validating the tenant and durably recording the command.
    async fn run_suspend_csr(
        &self,
        req: certificates::SuspendCsrRequest,
    ) -> Result<certificates::SuspendCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuspendCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuspendCsr", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = certificates::ExportCertificatesRequest {
            context: req.context.clone(),
            filter: Some(certificates::export_certificates_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req.csr_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = CertificateQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::ExportCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_certificates(input).await?;
            let mut frames = response.into_inner();
            let mut last = certificates::ExportCertificatesResponse::default();
            // The caller asks for a bounded snapshot, then drops its subscription.
            for _ in 0..8 {
                match frames.message().await? {
                    Some(frame) => last = frame,
                    None => break,
                }
            }
            Ok::<_, tonic::Status>(last)
        }
        .await?;
        let outgoing_1 = ledger::CreateLedgerAccountRequest {
            context: req.context.clone(),
            ledger_account: Some(ledger::LedgerAccount {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(ledger::PostingBatchRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = LedgerAccountServiceClient::new(channel);
            let input: tonic::Request<ledger::CreateLedgerAccountRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_ledger_account(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::AdvanceEnvironmentWorkflowRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    display_name: reply_1
                        .ledger_account
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = EnvironmentWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::AdvanceEnvironmentWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_environment_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::SuspendCsrResponse::default();
        let contribution: String = reply_1
            .ledger_account
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .phone
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .requester
                .get_or_insert_with(Default::default)
                .phone = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ReinstateCsr after validating the tenant and durably recording the command.
    async fn run_reinstate_csr(
        &self,
        req: certificates::ReinstateCsrRequest,
    ) -> Result<certificates::ReinstateCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReinstateCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReinstateCsr", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamThrottleMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify_contact
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                name: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify_contact
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                phone: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify_contact
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify_contact
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = ThrottleReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamThrottleMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_throttle_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamThrottleMetricsResponse::default();
            // The caller asks for a bounded snapshot, then drops its subscription.
            for _ in 0..8 {
                match frames.message().await? {
                    Some(frame) => last = frame,
                    None => break,
                }
            }
            Ok::<_, tonic::Status>(last)
        }
        .await?;
        let command_1 = ReinstateCsrDispatch1::from(req.clone());
        let outgoing_1: certificates::CountCertificateAuthoritiesRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateAuthorityQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::CountCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_certificate_authorities(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ReinstateCsrResponse::default();
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TerminateCsr after validating the tenant and durably recording the command.
    async fn run_terminate_csr(
        &self,
        req: certificates::TerminateCsrRequest,
    ) -> Result<certificates::TerminateCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TerminateCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TerminateCsr", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                })),
                requester: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.terminate_csr_0_port.as_ref(), outgoing_0).await?;
        let mut response = certificates::TerminateCsrResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RenewCsr after validating the tenant and durably recording the command.
    async fn run_renew_csr(
        &self,
        req: certificates::RenewCsrRequest,
    ) -> Result<certificates::RenewCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RenewCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RenewCsr", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = certificates::QuoteCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                requester: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.certificates.v1.QuoteCsrRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CsrCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::QuoteCsrRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::EstimateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    audit: Some(common::AuditInfo {
                        updated_at: reply_0
                            .csr
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        ..Default::default()
                    }),
                    children: vec![certificates::CertificateAuthority {
                        manager: Some(identity::UserRef {
                            email: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                                Some(certificates::csr::Origin::Customer(value)) => value,
                                _ => Default::default(),
                            })
                            .email
                            .clone(),
                            user_id: reply_0
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .assignee
                                .clone()
                                .unwrap_or_default()
                                .user_id
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify_contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<certificates::EstimateCertificateAuthorityRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
                let input: tonic::Request<certificates::EstimateCertificateAuthorityRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.estimate_certificate_authority(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_1)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_1 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = certificates::RenewCsrResponse::default();
        let contribution: Option<common::PersonName> = reply_0
            .csr
            .clone()
            .unwrap_or_default()
            .requester
            .clone()
            .unwrap_or_default()
            .name
            .clone();
        if contribution != <Option<common::PersonName>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .requester
                .get_or_insert_with(Default::default)
                .name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ExpireCsr after validating the tenant and durably recording the command.
    async fn run_expire_csr(
        &self,
        req: certificates::ExpireCsrRequest,
    ) -> Result<certificates::ExpireCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExpireCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExpireCsr", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ShareRateLimitReportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::share_rate_limit_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.expire_csr_0_port)(outgoing_0).await?;
        let outgoing_1 = certificates::ValidateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            created_at: reply_0
                                .rate_limit
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            user_id: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .new_owner
                                .clone()
                                .unwrap_or_default()
                                .user_id
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        email: reply_0
                            .rate_limit
                            .clone()
                            .unwrap_or_default()
                            .owner
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.expire_csr_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = certificates::ExpireCsrResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run GetCsrLifecycle after validating the tenant and durably recording the command.
    async fn run_get_csr_lifecycle(
        &self,
        req: certificates::GetCsrLifecycleRequest,
    ) -> Result<certificates::GetCsrLifecycleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCsrLifecycle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCsrLifecycle", "running workflow");
        let outgoing_0 = notifications::FindNotificationsRequest {
            context: req.context.clone(),
            filter: Some(notifications::find_notifications_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.csr_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.get_csr_lifecycle_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = certificates::GetCsrLifecycleResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ReinstateCsr for CountCertificateAuthorities.
struct ReinstateCsrDispatch1 {
    context: Option<common::RequestContext>,
    filter_owner_id: String,
    filter_created_after: Option<prost_types::Timestamp>,
    filter_labels: std::collections::BTreeMap<String, String>,
}
impl From<certificates::ReinstateCsrRequest> for ReinstateCsrDispatch1 {
    fn from(req: certificates::ReinstateCsrRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_owner_id: req
                .details
                .clone()
                .unwrap_or_default()
                .new_owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            filter_created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
            filter_labels: req.annotations.clone(),
        }
    }
}
impl From<ReinstateCsrDispatch1> for certificates::CountCertificateAuthoritiesRequest {
    fn from(value: ReinstateCsrDispatch1) -> Self {
        certificates::CountCertificateAuthoritiesRequest {
            context: value.context,
            filter: Some(certificates::count_certificate_authorities_request::Filter {
                created_after: value.filter_created_after,
                labels: value.filter_labels,
                owner_id: value.filter_owner_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CsrLifecycleService for CsrLifecycleController {
    /// Activates a csr.
    async fn activate_csr(
        &self,
        request: tonic::Request<certificates::ActivateCsrRequest>,
    ) -> Result<tonic::Response<certificates::ActivateCsrResponse>, tonic::Status> {
        self.run_activate_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Suspends a csr.
    async fn suspend_csr(
        &self,
        request: tonic::Request<certificates::SuspendCsrRequest>,
    ) -> Result<tonic::Response<certificates::SuspendCsrResponse>, tonic::Status> {
        self.run_suspend_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reinstates a csr.
    async fn reinstate_csr(
        &self,
        request: tonic::Request<certificates::ReinstateCsrRequest>,
    ) -> Result<tonic::Response<certificates::ReinstateCsrResponse>, tonic::Status> {
        self.run_reinstate_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Terminates a csr.
    async fn terminate_csr(
        &self,
        request: tonic::Request<certificates::TerminateCsrRequest>,
    ) -> Result<tonic::Response<certificates::TerminateCsrResponse>, tonic::Status> {
        self.run_terminate_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Renews a csr.
    async fn renew_csr(
        &self,
        request: tonic::Request<certificates::RenewCsrRequest>,
    ) -> Result<tonic::Response<certificates::RenewCsrResponse>, tonic::Status> {
        self.run_renew_csr(request.into_inner()).await.map(tonic::Response::new)
    }
    /// Expires a csr.
    async fn expire_csr(
        &self,
        request: tonic::Request<certificates::ExpireCsrRequest>,
    ) -> Result<tonic::Response<certificates::ExpireCsrResponse>, tonic::Status> {
        self.run_expire_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a csr.
    async fn get_csr_lifecycle(
        &self,
        request: tonic::Request<certificates::GetCsrLifecycleRequest>,
    ) -> Result<tonic::Response<certificates::GetCsrLifecycleResponse>, tonic::Status> {
        self.run_get_csr_lifecycle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
