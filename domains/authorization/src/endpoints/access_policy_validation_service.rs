use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_notification_service_client::AccessPolicyNotificationServiceClient;
use acme_proto::acme::authorization::v1::access_policy_validation_service_server::AccessPolicyValidationService;
use acme_proto::acme::authorization::v1::role_binding_export_service_client::RoleBindingExportServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;

/// Checks access policies against the rules before anything acts on them.
#[derive(Clone)]
pub struct AccessPolicyValidationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl AccessPolicyValidationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ValidateAccessPolicy after validating the tenant and durably recording the command.
    async fn run_validate_access_policy(
        &self,
        req: authorization::ValidateAccessPolicyRequest,
    ) -> Result<authorization::ValidateAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::ValidateAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_policy.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_policy.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run PreviewAccessPolicy after validating the tenant and durably recording the command.
    async fn run_preview_access_policy(
        &self,
        req: authorization::PreviewAccessPolicyRequest,
    ) -> Result<authorization::PreviewAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::PreviewAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_policy.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_policy.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run CheckAccessPolicyEligibility after validating the tenant and durably recording the command.
    async fn run_check_access_policy_eligibility(
        &self,
        req: authorization::CheckAccessPolicyEligibilityRequest,
    ) -> Result<authorization::CheckAccessPolicyEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckAccessPolicyEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckAccessPolicyEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::CheckAccessPolicyEligibilityResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.access_policy.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.access_policy.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeAccessPolicy after validating the tenant and durably recording the command.
    async fn run_normalize_access_policy(
        &self,
        req: authorization::NormalizeAccessPolicyRequest,
    ) -> Result<authorization::NormalizeAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::StreamAccessPolicyNoticesRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .access_policy
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                name: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = AccessPolicyNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::StreamAccessPolicyNoticesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_access_policy_notices(input).await?;
            let mut frames = response.into_inner();
            let mut last = authorization::StreamAccessPolicyNoticesResponse::default();
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
        let mut response = authorization::NormalizeAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: i32 = req.access_policy.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.access_policy.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ExplainAccessPolicyDecision after validating the tenant and durably recording the command.
    async fn run_explain_access_policy_decision(
        &self,
        req: authorization::ExplainAccessPolicyDecisionRequest,
    ) -> Result<authorization::ExplainAccessPolicyDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainAccessPolicyDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainAccessPolicyDecision", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().description.clone(), "recorded audit field");
        let outgoing_0 = authorization::ListRoleBindingExportsRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_role_binding_exports_request::Filter {
                created_after: req
                    .access_policy
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.access_policy.clone().unwrap_or_default().parameters.clone(),
                owner_id: req.access_policy.clone().unwrap_or_default().access_policy_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<authorization::ListRoleBindingExportsRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RoleBindingExportServiceClient::new(channel);
            let input: tonic::Request<authorization::ListRoleBindingExportsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_role_binding_exports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::ExplainAccessPolicyDecisionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: String = req.access_policy.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_policy.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run ScoreAccessPolicy after validating the tenant and durably recording the command.
    async fn run_score_access_policy(
        &self,
        req: authorization::ScoreAccessPolicyRequest,
    ) -> Result<authorization::ScoreAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::ScoreAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: i64 = req.access_policy.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.access_policy.get_or_insert_with(Default::default).version = contribution;
        }
        Ok(response)
    }
    /// Run LintAccessPolicies after validating the tenant and durably recording the command.
    async fn run_lint_access_policies(
        &self,
        req: authorization::LintAccessPoliciesRequest,
    ) -> Result<authorization::LintAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::LintAccessPoliciesResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_policy.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_policy.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run GetAccessPolicyRules after validating the tenant and durably recording the command.
    async fn run_get_access_policy_rules(
        &self,
        req: authorization::GetAccessPolicyRulesRequest,
    ) -> Result<authorization::GetAccessPolicyRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetAccessPolicyRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetAccessPolicyRules", "running workflow");
        let mut response = authorization::GetAccessPolicyRulesResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl AccessPolicyValidationService for AccessPolicyValidationHandler {
    /// Validates an access policy.
    async fn validate_access_policy(
        &self,
        request: tonic::Request<authorization::ValidateAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::ValidateAccessPolicyResponse>, tonic::Status> {
        self.run_validate_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews an access policy.
    async fn preview_access_policy(
        &self,
        request: tonic::Request<authorization::PreviewAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::PreviewAccessPolicyResponse>, tonic::Status> {
        self.run_preview_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks an access policy.
    async fn check_access_policy_eligibility(
        &self,
        request: tonic::Request<authorization::CheckAccessPolicyEligibilityRequest>,
    ) -> Result<tonic::Response<authorization::CheckAccessPolicyEligibilityResponse>, tonic::Status> {
        self.run_check_access_policy_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes an access policy.
    async fn normalize_access_policy(
        &self,
        request: tonic::Request<authorization::NormalizeAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::NormalizeAccessPolicyResponse>, tonic::Status> {
        self.run_normalize_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains an access policy.
    async fn explain_access_policy_decision(
        &self,
        request: tonic::Request<authorization::ExplainAccessPolicyDecisionRequest>,
    ) -> Result<tonic::Response<authorization::ExplainAccessPolicyDecisionResponse>, tonic::Status> {
        self.run_explain_access_policy_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores an access policy.
    async fn score_access_policy(
        &self,
        request: tonic::Request<authorization::ScoreAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::ScoreAccessPolicyResponse>, tonic::Status> {
        self.run_score_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints an access policy.
    async fn lint_access_policies(
        &self,
        request: tonic::Request<authorization::LintAccessPoliciesRequest>,
    ) -> Result<tonic::Response<authorization::LintAccessPoliciesResponse>, tonic::Status> {
        self.run_lint_access_policies(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an access policy.
    async fn get_access_policy_rules(
        &self,
        request: tonic::Request<authorization::GetAccessPolicyRulesRequest>,
    ) -> Result<tonic::Response<authorization::GetAccessPolicyRulesResponse>, tonic::Status> {
        self.run_get_access_policy_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
