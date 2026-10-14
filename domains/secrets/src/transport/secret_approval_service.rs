use acme_proto::acme::analytics::v1 as analytics;
use acme_proto::acme::analytics::v1::analytics_event_service_client::AnalyticsEventServiceClient;
use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::careplans::v1::care_plan_calculation_service_client::CarePlanCalculationServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_validation_service_client::SearchIndexValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::secret_approval_service_server::SecretApprovalService;
use acme_proto::acme::secrets::v1::secret_service_client::SecretServiceClient;
use acme_proto::acme::secrets::v1::secret_version_validation_service_client::SecretVersionValidationServiceClient;

/// Routes secrets through approval, with escalation and delegation.
#[derive(Clone)]
pub struct SecretApprovalController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
}
impl SecretApprovalController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SubmitSecret after validating the tenant and durably recording the command.
    async fn run_submit_secret(
        &self,
        req: secrets::SubmitSecretRequest,
    ) -> Result<secrets::SubmitSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitSecret", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::RecallCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            created_at: req
                                .secret
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
                    manager: Some(identity::UserRef {
                        email: req
                            .secret
                            .clone()
                            .unwrap_or_default()
                            .holder
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::AdvanceQuotaBucketWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::advance_quota_bucket_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .contact
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CertificateAuthorityApprovalServiceClient::new(channel_0);
                let input: tonic::Request<certificates::RecallCertificateAuthorityRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.recall_certificate_authority(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = QuotaBucketWorkflowServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::AdvanceQuotaBucketWorkflowRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.advance_quota_bucket_workflow(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = secrets::SubmitSecretResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.secret.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ApproveSecret after validating the tenant and durably recording the command.
    async fn run_approve_secret(
        &self,
        req: secrets::ApproveSecretRequest,
    ) -> Result<secrets::ApproveSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveSecret", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::EstimateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
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
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::EstimateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ApproveSecretResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.secret.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.secret.get_or_insert_with(Default::default).contact = contribution;
        }
        Ok(response)
    }
    /// Run RejectSecret after validating the tenant and durably recording the command.
    async fn run_reject_secret(
        &self,
        req: secrets::RejectSecretRequest,
    ) -> Result<secrets::RejectSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectSecret", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ResumeQuotaBucketWorkflowRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    etag: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    preferred_channel: req
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ResumeQuotaBucketWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_quota_bucket_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ResumeConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: reply_0
                                .quota_bucket
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
                            created_by: reply_0
                                .quota_bucket
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
                                .secret
                                .clone()
                                .unwrap_or_default()
                                .holder
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: req
                                .secret
                                .clone()
                                .unwrap_or_default()
                                .holder
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::ResumeConfigSetWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_config_set_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = analytics::WatchAnalyticsEventsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: reply_1
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                phone: req
                    .secret
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: req
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
        let _reply_2 = async move {
            let mut client = AnalyticsEventServiceClient::new(channel);
            let input: tonic::Request<analytics::WatchAnalyticsEventsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_analytics_events(input).await?;
            let mut frames = response.into_inner();
            let mut last = analytics::WatchAnalyticsEventsResponse::default();
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
        let mut response = secrets::RejectSecretResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.secret.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.secret.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
    /// Run EscalateSecret after validating the tenant and durably recording the command.
    async fn run_escalate_secret(
        &self,
        req: secrets::EscalateSecretRequest,
    ) -> Result<secrets::EscalateSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateSecret", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::ExplainAccessGrantCalculationRequest {
            context: req.context.clone(),
            details: Some(secrets::explain_access_grant_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
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
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::ExplainAccessGrantCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_access_grant_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::CheckSecretVersionEligibilityRequest {
            context: req.context.clone(),
            details: Some(secrets::check_secret_version_eligibility_request::Details {
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
                    phone: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret_version: Some(secrets::SecretVersion {
                subject: Some(secrets::AccessGrantRef {
                    access_grant_id: reply_0.access_grant.clone().unwrap_or_default().access_grant_id.clone(),
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
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
            let mut client = SecretVersionValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::CheckSecretVersionEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_secret_version_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::EscalateSecretResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: String = req.secret.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingSecrets after validating the tenant and durably recording the command.
    async fn run_list_pending_secrets(
        &self,
        req: secrets::ListPendingSecretsRequest,
    ) -> Result<secrets::ListPendingSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListPendingSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = secrets::ExplainAccessGrantCalculationRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
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
            context: req.context.clone(),
            details: Some(secrets::explain_access_grant_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::ExplainAccessGrantCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_access_grant_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ListPendingSecretsResponse::default();
        let contribution: String = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response.secrets.is_empty() {
                response.secrets.push(Default::default());
            }
            response.secrets[0].contact.get_or_insert_with(Default::default).email = contribution;
        }
        let contribution: Option<common::Money> =
            (match reply_0.access_grant.clone().unwrap_or_default().action.clone() {
                Some(secrets::access_grant::Action::Discount(value)) => value,
                _ => Default::default(),
            })
            .amount
            .clone();
        if contribution != <Option<common::Money>>::default() {
            if response.secrets.is_empty() {
                response.secrets.push(Default::default());
            }
            if response.secrets[0].limits.is_empty() {
                response.secrets[0].limits.push(Default::default());
            }
            response.secrets[0].limits[0].amount = contribution;
        }
        Ok(response)
    }
    /// Run DelegateSecretApproval after validating the tenant and durably recording the command.
    async fn run_delegate_secret_approval(
        &self,
        req: secrets::DelegateSecretApprovalRequest,
    ) -> Result<secrets::DelegateSecretApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DelegateSecretApproval", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateSecretApproval", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::UpdateSecretRequest {
            context: req.context.clone(),
            secret: Some(secrets::Secret {
                funding: Some(secrets::secret::Funding::Card(secrets::SecretCardFunding {
                    network: (match req.secret.clone().unwrap_or_default().funding.clone() {
                        Some(secrets::secret::Funding::Card(value)) => value,
                        _ => Default::default(),
                    })
                    .network
                    .clone(),
                    ..Default::default()
                })),
                contact: Some(common::ContactInfo {
                    email: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .contact
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
            let mut client = SecretServiceClient::new(channel);
            let input: tonic::Request<secrets::UpdateSecretRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_secret(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::LintFlagRolloutsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                organizer: Some(identity::UserRef {
                    email: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: reply_0
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
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
            let mut client = FlagRolloutValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::LintFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_flag_rollouts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = search::ScoreSearchIndexRequest {
            context: req.context.clone(),
            search_index: Some(search::SearchIndex {
                locator: Some(search::search_index::Locator::Address(common::PostalAddress {
                    region: reply_1
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .region
                        .clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    created_at: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = SearchIndexValidationServiceClient::new(channel);
            let input: tonic::Request<search::ScoreSearchIndexRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_search_index(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::GenerateConfigSetReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(configuration::generate_config_set_report_request::Details {
                filters: reply_0.secret.clone().unwrap_or_default().labels.clone(),
                period: reply_1.flag_rollout.clone().unwrap_or_default().window.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigSetReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::GenerateConfigSetReportRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_config_set_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::DelegateSecretApprovalResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0.secret.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.secret.get_or_insert_with(Default::default).opened_at = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_2
            .search_index
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RecallSecret after validating the tenant and durably recording the command.
    async fn run_recall_secret(
        &self,
        req: secrets::RecallSecretRequest,
    ) -> Result<secrets::RecallSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallSecret", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let jobs_0 = req
            .secret
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = careplans::CompareCarePlanOptionsRequest {
                        care_plan: Some(careplans::CarePlan {
                            audit: Some(common::AuditInfo {
                                updated_by: req
                                    .secret
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .updated_by
                                    .clone(),
                                ..Default::default()
                            }),
                            counterparty: Some(appointments::AppointmentRef {
                                display_name: req
                                    .details
                                    .clone()
                                    .unwrap_or_default()
                                    .approver
                                    .clone()
                                    .unwrap_or_default()
                                    .display_name
                                    .clone(),
                                ..Default::default()
                            }),
                            lines: vec![careplans::CarePlanLine {
                                amount: item.amount.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CarePlanCalculationServiceClient::new(channel);
                        let input: tonic::Request<careplans::CompareCarePlanOptionsRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.compare_care_plan_options(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = secrets::EstimateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_at: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(secrets::estimate_access_grant_request::Details {
                destination: Some(common::PostalAddress {
                    lines: reply_0
                        .care_plan
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::EstimateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::RecallSecretResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.secret.clone().unwrap_or_default().holder.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.secret.get_or_insert_with(Default::default).holder = contribution;
        }
        Ok(response)
    }
    /// Run GetSecretApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_secret_approval_status(
        &self,
        req: secrets::GetSecretApprovalStatusRequest,
    ) -> Result<secrets::GetSecretApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetSecretApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetSecretApprovalStatus", "running workflow");
        let outgoing_0 = configuration::SignalConfigChangeWorkflowRequest {
            context: req.context.clone(),
            details: Some(configuration::signal_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
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
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::SignalConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::PreviewSecretVersionRequest {
            context: req.context.clone(),
            secret_version: Some(secrets::SecretVersion {
                subject: Some(secrets::AccessGrantRef {
                    display_name: reply_0
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
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
            let mut client = SecretVersionValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::PreviewSecretVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_secret_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::GetSecretApprovalStatusResponse::default();
        let contribution: String = reply_0
            .config_change
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { SecretApprovalService, SecretApprovalController;
unary {
submit_secret (secrets::SubmitSecretRequest) -> secrets::SubmitSecretResponse => run_submit_secret;
approve_secret (secrets::ApproveSecretRequest) -> secrets::ApproveSecretResponse => run_approve_secret;
reject_secret (secrets::RejectSecretRequest) -> secrets::RejectSecretResponse => run_reject_secret;
escalate_secret (secrets::EscalateSecretRequest) -> secrets::EscalateSecretResponse => run_escalate_secret;
list_pending_secrets (secrets::ListPendingSecretsRequest) -> secrets::ListPendingSecretsResponse => run_list_pending_secrets;
delegate_secret_approval (secrets::DelegateSecretApprovalRequest) -> secrets::DelegateSecretApprovalResponse => run_delegate_secret_approval;
recall_secret (secrets::RecallSecretRequest) -> secrets::RecallSecretResponse => run_recall_secret;
get_secret_approval_status (secrets::GetSecretApprovalStatusRequest) -> secrets::GetSecretApprovalStatusResponse => run_get_secret_approval_status;
}
server {
}
client {
}
bidi {
}
}
