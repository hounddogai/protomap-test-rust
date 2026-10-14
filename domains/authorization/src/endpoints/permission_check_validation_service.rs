use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_validation_service_server::PermissionCheckValidationService;
use acme_proto::acme::authorization::v1::role_binding_history_service_client::RoleBindingHistoryServiceClient;

/// Checks permission checks against the rules before anything acts on them.
#[derive(Clone)]
pub struct PermissionCheckValidationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl PermissionCheckValidationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ValidatePermissionCheck after validating the tenant and durably recording the command.
    async fn run_validate_permission_check(
        &self,
        req: authorization::ValidatePermissionCheckRequest,
    ) -> Result<authorization::ValidatePermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ValidatePermissionCheck", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidatePermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::ValidatePermissionCheckResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: String = req.permission_check.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run PreviewPermissionCheck after validating the tenant and durably recording the command.
    async fn run_preview_permission_check(
        &self,
        req: authorization::PreviewPermissionCheckRequest,
    ) -> Result<authorization::PreviewPermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewPermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewPermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::PreviewPermissionCheckResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: String = req.permission_check.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run CheckPermissionCheckEligibility after validating the tenant and durably recording the command.
    async fn run_check_permission_check_eligibility(
        &self,
        req: authorization::CheckPermissionCheckEligibilityRequest,
    ) -> Result<authorization::CheckPermissionCheckEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckPermissionCheckEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckPermissionCheckEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::CheckPermissionCheckEligibilityResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: String = req.permission_check.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run NormalizePermissionCheck after validating the tenant and durably recording the command.
    async fn run_normalize_permission_check(
        &self,
        req: authorization::NormalizePermissionCheckRequest,
    ) -> Result<authorization::NormalizePermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NormalizePermissionCheck", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizePermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::NormalizePermissionCheckResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: String = req.permission_check.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.permission_check.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run ExplainPermissionCheckDecision after validating the tenant and durably recording the command.
    async fn run_explain_permission_check_decision(
        &self,
        req: authorization::ExplainPermissionCheckDecisionRequest,
    ) -> Result<authorization::ExplainPermissionCheckDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainPermissionCheckDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainPermissionCheckDecision", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::ExplainPermissionCheckDecisionResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: String = req.permission_check.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ScorePermissionCheck after validating the tenant and durably recording the command.
    async fn run_score_permission_check(
        &self,
        req: authorization::ScorePermissionCheckRequest,
    ) -> Result<authorization::ScorePermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScorePermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScorePermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::ScorePermissionCheckResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: String = req.permission_check.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.permission_check.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run GetPermissionCheckRules after validating the tenant and durably recording the command.
    async fn run_get_permission_check_rules(
        &self,
        req: authorization::GetPermissionCheckRulesRequest,
    ) -> Result<authorization::GetPermissionCheckRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetPermissionCheckRules", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetPermissionCheckRules", "running workflow");
        let outgoing_0 = authorization::CompareRoleBindingVersionsRequest {
            context: req.context.clone(),
            details: Some(authorization::compare_role_binding_versions_request::Details {
                author_email: req
                    .context
                    .clone()
                    .unwrap_or_default()
                    .actor
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RoleBindingHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::CompareRoleBindingVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_role_binding_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::GetPermissionCheckRulesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0
            .role_binding
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .access_policy_id
            .clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl PermissionCheckValidationService for PermissionCheckValidationController {
    /// Validates a permission check.
    async fn validate_permission_check(
        &self,
        request: tonic::Request<authorization::ValidatePermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::ValidatePermissionCheckResponse>, tonic::Status> {
        self.run_validate_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a permission check.
    async fn preview_permission_check(
        &self,
        request: tonic::Request<authorization::PreviewPermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::PreviewPermissionCheckResponse>, tonic::Status> {
        self.run_preview_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a permission check.
    async fn check_permission_check_eligibility(
        &self,
        request: tonic::Request<authorization::CheckPermissionCheckEligibilityRequest>,
    ) -> Result<tonic::Response<authorization::CheckPermissionCheckEligibilityResponse>, tonic::Status> {
        self.run_check_permission_check_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a permission check.
    async fn normalize_permission_check(
        &self,
        request: tonic::Request<authorization::NormalizePermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::NormalizePermissionCheckResponse>, tonic::Status> {
        self.run_normalize_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a permission check.
    async fn explain_permission_check_decision(
        &self,
        request: tonic::Request<authorization::ExplainPermissionCheckDecisionRequest>,
    ) -> Result<tonic::Response<authorization::ExplainPermissionCheckDecisionResponse>, tonic::Status> {
        self.run_explain_permission_check_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a permission check.
    async fn score_permission_check(
        &self,
        request: tonic::Request<authorization::ScorePermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::ScorePermissionCheckResponse>, tonic::Status> {
        self.run_score_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a permission check.
    async fn get_permission_check_rules(
        &self,
        request: tonic::Request<authorization::GetPermissionCheckRulesRequest>,
    ) -> Result<tonic::Response<authorization::GetPermissionCheckRulesResponse>, tonic::Status> {
        self.run_get_permission_check_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
