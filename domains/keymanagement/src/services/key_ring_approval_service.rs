use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_approval_service_server::KeyRingApprovalService;
use acme_proto::acme::keymanagement::v1::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_client::KeyRingReportingServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_reporting_service_client::KeyRotationReportingServiceClient;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_validation_service_client::LedgerAccountValidationServiceClient;

/// Routes key rings through approval, with escalation and delegation.
#[derive(Clone)]
pub struct KeyRingApprovalEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    escalate_key_ring_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::StartConfigSetWorkflowRequest,
            configuration::StartConfigSetWorkflowResponse,
        >,
    >,
    escalate_key_ring_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                keymanagement::ListPendingCryptoKeysRequest,
                keymanagement::ListPendingCryptoKeysResponse,
            >,
    >,
    recall_key_ring_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            keymanagement::ShareKeyRingReportRequest,
            keymanagement::ShareKeyRingReportResponse,
        >,
    >,
    recall_key_ring_2_port: acme_platform::ports::Callback<
        keymanagement::ExplainKeyRingDecisionRequest,
        keymanagement::ExplainKeyRingDecisionResponse,
    >,
}
impl KeyRingApprovalEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::StartConfigSetWorkflowRequest,
            configuration::StartConfigSetWorkflowResponse,
        > = {
            let client = ConfigSetWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::StartConfigSetWorkflowRequest| -> acme_platform::ports::RpcFuture<configuration::StartConfigSetWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.start_config_set_workflow(request).await?.into_inner())
})
})
        };
        let escalate_key_ring_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::ListPendingCryptoKeysRequest,
            keymanagement::ListPendingCryptoKeysResponse,
        > = {
            let client = CryptoKeyApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ListPendingCryptoKeysRequest| -> acme_platform::ports::RpcFuture<keymanagement::ListPendingCryptoKeysResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_pending_crypto_keys(request).await?.into_inner())
})
})
        };
        let escalate_key_ring_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::ShareKeyRingReportRequest,
            keymanagement::ShareKeyRingReportResponse,
        > = {
            let client = KeyRingReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ShareKeyRingReportRequest| -> acme_platform::ports::RpcFuture<keymanagement::ShareKeyRingReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_key_ring_report(request).await?.into_inner())
})
})
        };
        let recall_key_ring_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            keymanagement::ExplainKeyRingDecisionRequest,
            keymanagement::ExplainKeyRingDecisionResponse,
        > = {
            let client = KeyRingValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ExplainKeyRingDecisionRequest| -> acme_platform::ports::RpcFuture<keymanagement::ExplainKeyRingDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_key_ring_decision(request).await?.into_inner())
})
})
        };
        let recall_key_ring_2_port = callback;
        Self {
            channel,
            repository,
            escalate_key_ring_0_port,
            escalate_key_ring_1_port,
            recall_key_ring_1_port,
            recall_key_ring_2_port,
        }
    }
    /// Run SubmitKeyRing after validating the tenant and durably recording the command.
    async fn run_submit_key_ring(
        &self,
        req: keymanagement::SubmitKeyRingRequest,
    ) -> Result<keymanagement::SubmitKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().parent_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::SuspendKeyRingRequest {
            context: req.context.clone(),
            details: Some(keymanagement::suspend_key_ring_request::Details {
                new_owner: Some(identity::UserRef {
                    display_name: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    user_id: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                notify_contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_ring_id: req.key_ring.clone().unwrap_or_default().key_ring_id.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.keymanagement.v1.SuspendKeyRingRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingLifecycleServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SuspendKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suspend_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::SubmitCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            created_by: req
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            etag: req
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            postal_code: req
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .postal_code
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            user_id: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .approver
                                .clone()
                                .unwrap_or_default()
                                .user_id
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<certificates::SubmitCertificateAuthorityRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CertificateAuthorityApprovalServiceClient::new(channel);
                let input: tonic::Request<certificates::SubmitCertificateAuthorityRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.submit_certificate_authority(input).await?;
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
        let outgoing_2 = keymanagement::ShareKeyRotationReportRequest {
            context: req.context.clone(),
            details: Some(keymanagement::share_key_rotation_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = KeyRotationReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ShareKeyRotationReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_key_rotation_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = keymanagement::SubmitKeyRingResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run ApproveKeyRing after validating the tenant and durably recording the command.
    async fn run_approve_key_ring(
        &self,
        req: keymanagement::ApproveKeyRingRequest,
    ) -> Result<keymanagement::ApproveKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = ledger::ValidateLedgerAccountRequest {
            context: req.context.clone(),
            ledger_account: Some(ledger::LedgerAccount {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = LedgerAccountValidationServiceClient::new(channel);
            let input: tonic::Request<ledger::ValidateLedgerAccountRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_ledger_account(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::GenerateKeyRotationReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(keymanagement::generate_key_rotation_report_request::Details {
                filters: reply_0.ledger_account.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = KeyRotationReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::GenerateKeyRotationReportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_key_rotation_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ApproveKeyRingResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0
            .ledger_account
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).parent_id = contribution;
        }
        Ok(response)
    }
    /// Run RejectKeyRing after validating the tenant and durably recording the command.
    async fn run_reject_key_ring(
        &self,
        req: keymanagement::RejectKeyRingRequest,
    ) -> Result<keymanagement::RejectKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::SuspendKeyRingRequest {
            context: req.context.clone(),
            details: Some(keymanagement::suspend_key_ring_request::Details {
                effective_at: req.details.clone().unwrap_or_default().effective_at.clone(),
                new_owner: Some(identity::UserRef {
                    display_name: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                notify_contact: Some(common::ContactInfo {
                    email: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_ring_id: req.key_ring.clone().unwrap_or_default().key_ring_id.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<keymanagement::SuspendKeyRingRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingLifecycleServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SuspendKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suspend_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = RejectKeyRingDispatch1::from(req.clone());
        command_1.crypto_key_holder_display_name = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        let outgoing_1: keymanagement::ExplainCryptoKeyCalculationRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ExplainCryptoKeyCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_crypto_key_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::RejectKeyRingResponse::default();
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = reply_0.key_ring.clone().unwrap_or_default().key_ring_id.clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].key_ring_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: i32 = req.key_ring.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.key_ring.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run EscalateKeyRing after validating the tenant and durably recording the command.
    async fn run_escalate_key_ring(
        &self,
        req: keymanagement::EscalateKeyRingRequest,
    ) -> Result<keymanagement::EscalateKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::StartConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_by: req
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_by
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            lines: req
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .approver
                                .clone()
                                .unwrap_or_default()
                                .email
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.escalate_key_ring_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = keymanagement::ListPendingCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::list_pending_crypto_keys_request::Filter {
                created_after: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: reply_0.outcome.clone().unwrap_or_default().reference_id.clone(),
                region: reply_0
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .location
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.escalate_key_ring_1_port.as_ref(), outgoing_1).await?;
        let mut response = keymanagement::EscalateKeyRingResponse::default();
        let contribution: String = reply_0.config_set.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0].children[0].name =
                contribution;
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.key_ring.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.key_ring.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingKeyRings after validating the tenant and durably recording the command.
    async fn run_list_pending_key_rings(
        &self,
        req: keymanagement::ListPendingKeyRingsRequest,
    ) -> Result<keymanagement::ListPendingKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListPendingKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = keymanagement::ListPendingKeyRingsResponse::default();
        Ok(response)
    }
    /// Run RecallKeyRing after validating the tenant and durably recording the command.
    async fn run_recall_key_ring(
        &self,
        req: keymanagement::RecallKeyRingRequest,
    ) -> Result<keymanagement::RecallKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ScheduleKeyRotationReportRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(keymanagement::schedule_key_rotation_report_request::Details {
                filters: (req.key_ring.clone().unwrap_or_default().attributes.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ScheduleKeyRotationReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_key_rotation_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::ShareKeyRingReportRequest {
            context: req.context.clone(),
            details: Some(keymanagement::share_key_ring_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_ring_id: req.key_ring_id.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.recall_key_ring_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = keymanagement::ExplainKeyRingDecisionRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            updated_at: reply_1
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            updated_by: reply_1
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_by
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            postal_code: reply_1
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .postal_code
                                .clone(),
                            region: reply_1
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .region
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.recall_key_ring_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let mut response = keymanagement::RecallKeyRingResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .key_ring
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: String = reply_1
            .key_ring
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .recipient
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .location
                .get_or_insert_with(Default::default)
                .recipient = contribution;
        }
        let contribution: i32 = reply_2.key_ring.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].status = contribution;
        }
        let contribution: Vec<String> = reply_2.key_ring.clone().unwrap_or_default().path.clone();
        if contribution != <Vec<String>>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .path
                .extend(contribution);
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run GetKeyRingApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_key_ring_approval_status(
        &self,
        req: keymanagement::GetKeyRingApprovalStatusRequest,
    ) -> Result<keymanagement::GetKeyRingApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetKeyRingApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetKeyRingApprovalStatus", "running workflow");
        let mut response = keymanagement::GetKeyRingApprovalStatusResponse::default();
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by RejectKeyRing for ExplainCryptoKeyCalculation.
struct RejectKeyRingDispatch1 {
    context: Option<common::RequestContext>,
    crypto_key_holder_display_name: String,
    details_contact_email: String,
}
impl From<keymanagement::RejectKeyRingRequest> for RejectKeyRingDispatch1 {
    fn from(req: keymanagement::RejectKeyRingRequest) -> Self {
        Self {
            context: req.context.clone(),
            crypto_key_holder_display_name: Default::default(),
            details_contact_email: req
                .details
                .clone()
                .unwrap_or_default()
                .approver
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
        }
    }
}
impl From<RejectKeyRingDispatch1> for keymanagement::ExplainCryptoKeyCalculationRequest {
    fn from(value: RejectKeyRingDispatch1) -> Self {
        keymanagement::ExplainCryptoKeyCalculationRequest {
            context: value.context,
            crypto_key: Some(keymanagement::CryptoKey {
                holder: Some(keymanagement::KeyRingRef {
                    display_name: value.crypto_key_holder_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            details: Some(keymanagement::explain_crypto_key_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: value.details_contact_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRingApprovalService for KeyRingApprovalEndpoint {
    /// Submits a key ring.
    async fn submit_key_ring(
        &self,
        request: tonic::Request<keymanagement::SubmitKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::SubmitKeyRingResponse>, tonic::Status> {
        self.run_submit_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a key ring.
    async fn approve_key_ring(
        &self,
        request: tonic::Request<keymanagement::ApproveKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::ApproveKeyRingResponse>, tonic::Status> {
        self.run_approve_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a key ring.
    async fn reject_key_ring(
        &self,
        request: tonic::Request<keymanagement::RejectKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::RejectKeyRingResponse>, tonic::Status> {
        self.run_reject_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a key ring.
    async fn escalate_key_ring(
        &self,
        request: tonic::Request<keymanagement::EscalateKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::EscalateKeyRingResponse>, tonic::Status> {
        self.run_escalate_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists key rings a page at a time.
    async fn list_pending_key_rings(
        &self,
        request: tonic::Request<keymanagement::ListPendingKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::ListPendingKeyRingsResponse>, tonic::Status> {
        self.run_list_pending_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls a key ring.
    async fn recall_key_ring(
        &self,
        request: tonic::Request<keymanagement::RecallKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::RecallKeyRingResponse>, tonic::Status> {
        self.run_recall_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key ring.
    async fn get_key_ring_approval_status(
        &self,
        request: tonic::Request<keymanagement::GetKeyRingApprovalStatusRequest>,
    ) -> Result<tonic::Response<keymanagement::GetKeyRingApprovalStatusResponse>, tonic::Status> {
        self.run_get_key_ring_approval_status(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
