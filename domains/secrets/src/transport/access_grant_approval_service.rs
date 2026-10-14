use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::customers::v1 as customers;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_client::FeatureFlagHistoryServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::forms::v1 as forms;
use acme_proto::acme::forms::v1::form_calculation_service_client::FormCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::invoicing::v1 as invoicing;
use acme_proto::acme::invoicing::v1::invoice_batch_service_client::InvoiceBatchServiceClient;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_validation_service_client::LedgerAccountValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_approval_service_server::AccessGrantApprovalService;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::secret_approval_service_client::SecretApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_query_service_client::SecretQueryServiceClient;
use acme_proto::acme::secrets::v1::secret_version_approval_service_client::SecretVersionApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_version_validation_service_client::SecretVersionValidationServiceClient;

/// Routes access grants through approval, with escalation and delegation.
#[derive(Clone)]
pub struct AccessGrantApprovalController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    comment_access_grant_approval_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<ledger::PreviewLedgerAccountRequest, ledger::PreviewLedgerAccountResponse>,
    >,
    comment_access_grant_approval_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                secrets::ExplainSecretVersionDecisionRequest,
                secrets::ExplainSecretVersionDecisionResponse,
            >,
    >,
    delegate_access_grant_approval_0_port: acme_platform::ports::Callback<
        certificates::CancelCertificateAuthorityWorkflowRequest,
        certificates::CancelCertificateAuthorityWorkflowResponse,
    >,
    delegate_access_grant_approval_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::CompareFeatureFlagVersionsRequest,
            featureflags::CompareFeatureFlagVersionsResponse,
        >,
    >,
    get_access_grant_approval_status_0_port:
        acme_platform::ports::Callback<invoicing::ListInvoiceBatchesRequest, invoicing::ListInvoiceBatchesResponse>,
}
impl AccessGrantApprovalController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            certificates::CancelCertificateAuthorityWorkflowRequest,
            certificates::CancelCertificateAuthorityWorkflowResponse,
        > = {
            let client = CertificateAuthorityWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CancelCertificateAuthorityWorkflowRequest| -> acme_platform::ports::RpcFuture<certificates::CancelCertificateAuthorityWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.cancel_certificate_authority_workflow(request).await?.into_inner())
})
})
        };
        let delegate_access_grant_approval_0_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::CompareFeatureFlagVersionsRequest,
            featureflags::CompareFeatureFlagVersionsResponse,
        > = {
            let client = FeatureFlagHistoryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::CompareFeatureFlagVersionsRequest| -> acme_platform::ports::RpcFuture<featureflags::CompareFeatureFlagVersionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_feature_flag_versions(request).await?.into_inner())
})
})
        };
        let delegate_access_grant_approval_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            invoicing::ListInvoiceBatchesRequest,
            invoicing::ListInvoiceBatchesResponse,
        > = {
            let client = InvoiceBatchServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: invoicing::ListInvoiceBatchesRequest| -> acme_platform::ports::RpcFuture<invoicing::ListInvoiceBatchesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_invoice_batches(request).await?.into_inner())
})
})
        };
        let get_access_grant_approval_status_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ledger::PreviewLedgerAccountRequest,
            ledger::PreviewLedgerAccountResponse,
        > = {
            let client = LedgerAccountValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ledger::PreviewLedgerAccountRequest| -> acme_platform::ports::RpcFuture<ledger::PreviewLedgerAccountResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_ledger_account(request).await?.into_inner())
})
})
        };
        let comment_access_grant_approval_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            secrets::ExplainSecretVersionDecisionRequest,
            secrets::ExplainSecretVersionDecisionResponse,
        > = {
            let client = SecretVersionValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::ExplainSecretVersionDecisionRequest| -> acme_platform::ports::RpcFuture<secrets::ExplainSecretVersionDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_secret_version_decision(request).await?.into_inner())
})
})
        };
        let comment_access_grant_approval_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            comment_access_grant_approval_0_port,
            comment_access_grant_approval_1_port,
            delegate_access_grant_approval_0_port,
            delegate_access_grant_approval_1_port,
            get_access_grant_approval_status_0_port,
        }
    }
    /// Run SubmitAccessGrant after validating the tenant and durably recording the command.
    async fn run_submit_access_grant(
        &self,
        req: secrets::SubmitAccessGrantRequest,
    ) -> Result<secrets::SubmitAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ScheduleThrottleReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                Some(secrets::access_grant::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ScheduleThrottleReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_throttle_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::SubmitAccessGrantResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_grant.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_grant.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run ApproveAccessGrant after validating the tenant and durably recording the command.
    async fn run_approve_access_grant(
        &self,
        req: secrets::ApproveAccessGrantRequest,
    ) -> Result<secrets::ApproveAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = forms::CalculateFormRequest {
            context: req.context.clone(),
            form: Some(forms::Form {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                counterparty: Some(customers::CustomerRef {
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
                lines: vec![forms::FormLine {
                    tax: Some(forms::FormLineTax {
                        amount: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                            Some(secrets::access_grant::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .amount
                        .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = FormCalculationServiceClient::new(channel);
            let input: tonic::Request<forms::CalculateFormRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_form(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = featureflags::RecordFlagEvaluationVersionRequest {
            context: req.context.clone(),
            details: Some(featureflags::record_flag_evaluation_version_request::Details {
                author_email: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .approver
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: reply_0
                        .form
                        .clone()
                        .unwrap_or_default()
                        .counterparty
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
            let mut client = FlagEvaluationHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::RecordFlagEvaluationVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.record_flag_evaluation_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = secrets::QuoteAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    updated_at: reply_0
                        .form
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::QuoteAccessGrantRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ApproveAccessGrantResponse::default();
        let contribution: String = reply_0
            .form
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .access_grant
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.access_grant.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_grant.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run RejectAccessGrant after validating the tenant and durably recording the command.
    async fn run_reject_access_grant(
        &self,
        req: secrets::RejectAccessGrantRequest,
    ) -> Result<secrets::RejectAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().description.clone(), "recorded audit field");
        let outgoing_0 = secrets::SubmitSecretVersionRequest {
            context: req.context.clone(),
            details: Some(secrets::submit_secret_version_request::Details {
                approver: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
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
            }),
            secret_version: Some(secrets::SecretVersion {
                subject: Some(secrets::AccessGrantRef {
                    access_grant_id: req.access_grant.clone().unwrap_or_default().access_grant_id.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = SecretVersionApprovalServiceClient::new(channel);
            let input: tonic::Request<secrets::SubmitSecretVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.submit_secret_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = featureflags::RecalculateFlagRolloutsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                organizer: Some(identity::UserRef {
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
                participants: vec![featureflags::FlagRolloutParticipant {
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagRolloutCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::RecalculateFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_flag_rollouts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::RejectAccessGrantResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.access_grant.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_grant.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingAccessGrants after validating the tenant and durably recording the command.
    async fn run_list_pending_access_grants(
        &self,
        req: secrets::ListPendingAccessGrantsRequest,
    ) -> Result<secrets::ListPendingAccessGrantsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPendingAccessGrants", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingAccessGrants", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = secrets::RejectSecretRequest {
            context: req.context.clone(),
            details: Some(secrets::reject_secret_request::Details {
                approver: Some(identity::UserRef {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret: Some(secrets::Secret {
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretApprovalServiceClient::new(channel);
            let input: tonic::Request<secrets::RejectSecretRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reject_secret(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::StreamThrottleMetricsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::stream_throttle_metrics_request::Filter {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
                email: reply_0
                    .secret
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                preferred_channel: reply_0
                    .secret
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = ThrottleReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamThrottleMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
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
        let mut response = secrets::ListPendingAccessGrantsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0
            .secret
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response.access_grants.is_empty() {
                response.access_grants.push(Default::default());
            }
            response.access_grants[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        Ok(response)
    }
    /// Run DelegateAccessGrantApproval after validating the tenant and durably recording the command.
    async fn run_delegate_access_grant_approval(
        &self,
        req: secrets::DelegateAccessGrantApprovalRequest,
    ) -> Result<secrets::DelegateAccessGrantApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateAccessGrantApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateAccessGrantApproval", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CancelCertificateAuthorityWorkflowRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    audit: Some(common::AuditInfo {
                        updated_at: req
                            .access_grant
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
                        audit: Some(common::AuditInfo {
                            created_at: req
                                .access_grant
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
                            display_name: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .approver
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            user_id: req
                                .access_grant
                                .clone()
                                .unwrap_or_default()
                                .owner
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
        let reply_0 = (self.delegate_access_grant_approval_0_port)(outgoing_0).await?;
        let outgoing_1 = featureflags::CompareFeatureFlagVersionsRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                owner: Some(identity::UserRef {
                    user_id: reply_0
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            self.delegate_access_grant_approval_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = secrets::DelegateAccessGrantApprovalResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.access_grant.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_grant.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run RecallAccessGrant after validating the tenant and durably recording the command.
    async fn run_recall_access_grant(
        &self,
        req: secrets::RecallAccessGrantRequest,
    ) -> Result<secrets::RecallAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ExplainFlagRolloutCalculationRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                organizer: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
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
                participants: vec![featureflags::FlagRolloutParticipant {
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(
            &outgoing_0,
            "acme.featureflags.v1.ExplainFlagRolloutCalculationRequest",
        )?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExplainFlagRolloutCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_flag_rollout_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::ExportSecretsRequest {
            context: req.context.clone(),
            filter: Some(secrets::export_secrets_request::Filter {
                region: reply_0
                    .flag_rollout
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
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = SecretQueryServiceClient::new(channel);
            let input: tonic::Request<secrets::ExportSecretsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_secrets(input).await?;
            let mut frames = response.into_inner();
            let mut last = secrets::ExportSecretsResponse::default();
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
        let mut response = secrets::RecallAccessGrantResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.access_grant.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.access_grant.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run GetAccessGrantApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_access_grant_approval_status(
        &self,
        req: secrets::GetAccessGrantApprovalStatusRequest,
    ) -> Result<secrets::GetAccessGrantApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetAccessGrantApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetAccessGrantApprovalStatus", "running workflow");
        let outgoing_0 = invoicing::ListInvoiceBatchesRequest {
            context: req.context.clone(),
            filter: Some(invoicing::list_invoice_batches_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.access_grant_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.get_access_grant_approval_status_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = secrets::GetAccessGrantApprovalStatusResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        Ok(response)
    }
    /// Run CommentAccessGrantApproval after validating the tenant and durably recording the command.
    async fn run_comment_access_grant_approval(
        &self,
        req: secrets::CommentAccessGrantApprovalRequest,
    ) -> Result<secrets::CommentAccessGrantApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CommentAccessGrantApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CommentAccessGrantApproval", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = ledger::PreviewLedgerAccountRequest {
            context: req.context.clone(),
            ledger_account: Some(ledger::LedgerAccount {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.comment_access_grant_approval_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = secrets::ExplainSecretVersionDecisionRequest {
            context: req.context.clone(),
            details: Some(secrets::explain_secret_version_decision_request::Details {
                contact: Some(common::ContactInfo {
                    name: reply_0
                        .ledger_account
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    preferred_channel: reply_0
                        .ledger_account
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret_version: Some(secrets::SecretVersion {
                subject: Some(secrets::AccessGrantRef {
                    access_grant_id: req.access_grant.clone().unwrap_or_default().access_grant_id.clone(),
                    display_name: reply_0
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
        let reply_1 =
            acme_platform::ports::execute(self.comment_access_grant_approval_1_port.as_ref(), outgoing_1).await?;
        let mut response = secrets::CommentAccessGrantApprovalResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_grant.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_grant.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl AccessGrantApprovalService for AccessGrantApprovalController {
    /// Submits an access grant.
    async fn submit_access_grant(
        &self,
        request: tonic::Request<secrets::SubmitAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::SubmitAccessGrantResponse>, tonic::Status> {
        self.run_submit_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves an access grant.
    async fn approve_access_grant(
        &self,
        request: tonic::Request<secrets::ApproveAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::ApproveAccessGrantResponse>, tonic::Status> {
        self.run_approve_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects an access grant.
    async fn reject_access_grant(
        &self,
        request: tonic::Request<secrets::RejectAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::RejectAccessGrantResponse>, tonic::Status> {
        self.run_reject_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists access grants a page at a time.
    async fn list_pending_access_grants(
        &self,
        request: tonic::Request<secrets::ListPendingAccessGrantsRequest>,
    ) -> Result<tonic::Response<secrets::ListPendingAccessGrantsResponse>, tonic::Status> {
        self.run_list_pending_access_grants(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates an access grant.
    async fn delegate_access_grant_approval(
        &self,
        request: tonic::Request<secrets::DelegateAccessGrantApprovalRequest>,
    ) -> Result<tonic::Response<secrets::DelegateAccessGrantApprovalResponse>, tonic::Status> {
        self.run_delegate_access_grant_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls an access grant.
    async fn recall_access_grant(
        &self,
        request: tonic::Request<secrets::RecallAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::RecallAccessGrantResponse>, tonic::Status> {
        self.run_recall_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an access grant.
    async fn get_access_grant_approval_status(
        &self,
        request: tonic::Request<secrets::GetAccessGrantApprovalStatusRequest>,
    ) -> Result<tonic::Response<secrets::GetAccessGrantApprovalStatusResponse>, tonic::Status> {
        self.run_get_access_grant_approval_status(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Comments an access grant.
    async fn comment_access_grant_approval(
        &self,
        request: tonic::Request<secrets::CommentAccessGrantApprovalRequest>,
    ) -> Result<tonic::Response<secrets::CommentAccessGrantApprovalResponse>, tonic::Status> {
        self.run_comment_access_grant_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
