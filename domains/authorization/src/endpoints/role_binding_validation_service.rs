use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_admin_service_client::AccessPolicyAdminServiceClient;
use acme_proto::acme::authorization::v1::role_binding_validation_service_server::RoleBindingValidationService;
use acme_proto::acme::common::v1 as common;

/// Checks role bindings against the rules before anything acts on them.
#[derive(Clone)]
pub struct RoleBindingValidationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    normalize_role_binding_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                authorization::RepairAccessPolicyRequest,
                authorization::RepairAccessPolicyResponse,
            >,
    >,
}
impl RoleBindingValidationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            authorization::RepairAccessPolicyRequest,
            authorization::RepairAccessPolicyResponse,
        > = {
            let client = AccessPolicyAdminServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::RepairAccessPolicyRequest| -> acme_platform::ports::RpcFuture<authorization::RepairAccessPolicyResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.repair_access_policy(request).await?.into_inner())
})
})
        };
        let normalize_role_binding_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            normalize_role_binding_0_port,
        }
    }
    /// Run ValidateRoleBinding after validating the tenant and durably recording the command.
    async fn run_validate_role_binding(
        &self,
        req: authorization::ValidateRoleBindingRequest,
    ) -> Result<authorization::ValidateRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::ValidateRoleBindingResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: i32 = req.role_binding.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.role_binding.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run PreviewRoleBinding after validating the tenant and durably recording the command.
    async fn run_preview_role_binding(
        &self,
        req: authorization::PreviewRoleBindingRequest,
    ) -> Result<authorization::PreviewRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::PreviewRoleBindingResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.role_binding.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.role_binding.get_or_insert_with(Default::default).due_at = contribution;
        }
        Ok(response)
    }
    /// Run CheckRoleBindingEligibility after validating the tenant and durably recording the command.
    async fn run_check_role_binding_eligibility(
        &self,
        req: authorization::CheckRoleBindingEligibilityRequest,
    ) -> Result<authorization::CheckRoleBindingEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckRoleBindingEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckRoleBindingEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::CheckRoleBindingEligibilityResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: String = req.role_binding.clone().unwrap_or_default().notes.clone();
        if contribution != <String>::default() {
            response.role_binding.get_or_insert_with(Default::default).notes = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeRoleBinding after validating the tenant and durably recording the command.
    async fn run_normalize_role_binding(
        &self,
        req: authorization::NormalizeRoleBindingRequest,
    ) -> Result<authorization::NormalizeRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::RepairAccessPolicyRequest {
            access_policy: Some(authorization::AccessPolicy {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .role_binding
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: req
                        .role_binding
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.normalize_role_binding_0_port.as_ref(), outgoing_0).await?;
        let mut response = authorization::NormalizeRoleBindingResponse::default();
        let contribution: String = reply_0
            .access_policy
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.role_binding.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.role_binding.get_or_insert_with(Default::default).due_at = contribution;
        }
        Ok(response)
    }
    /// Run ExplainRoleBindingDecision after validating the tenant and durably recording the command.
    async fn run_explain_role_binding_decision(
        &self,
        req: authorization::ExplainRoleBindingDecisionRequest,
    ) -> Result<authorization::ExplainRoleBindingDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainRoleBindingDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainRoleBindingDecision", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::ExplainRoleBindingDecisionResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.role_binding.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ScoreRoleBinding after validating the tenant and durably recording the command.
    async fn run_score_role_binding(
        &self,
        req: authorization::ScoreRoleBindingRequest,
    ) -> Result<authorization::ScoreRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::ScoreRoleBindingResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.role_binding.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.role_binding.get_or_insert_with(Default::default).due_at = contribution;
        }
        Ok(response)
    }
    /// Run LintRoleBindings after validating the tenant and durably recording the command.
    async fn run_lint_role_bindings(
        &self,
        req: authorization::LintRoleBindingsRequest,
    ) -> Result<authorization::LintRoleBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintRoleBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintRoleBindings", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::LintRoleBindingsResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: String = req.role_binding.clone().unwrap_or_default().notes.clone();
        if contribution != <String>::default() {
            response.role_binding.get_or_insert_with(Default::default).notes = contribution;
        }
        Ok(response)
    }
    /// Run GetRoleBindingRules after validating the tenant and durably recording the command.
    async fn run_get_role_binding_rules(
        &self,
        req: authorization::GetRoleBindingRulesRequest,
    ) -> Result<authorization::GetRoleBindingRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRoleBindingRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRoleBindingRules", "running workflow");
        let mut response = authorization::GetRoleBindingRulesResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl RoleBindingValidationService for RoleBindingValidationEndpoint {
    /// Validates a role binding.
    async fn validate_role_binding(
        &self,
        request: tonic::Request<authorization::ValidateRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::ValidateRoleBindingResponse>, tonic::Status> {
        self.run_validate_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a role binding.
    async fn preview_role_binding(
        &self,
        request: tonic::Request<authorization::PreviewRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::PreviewRoleBindingResponse>, tonic::Status> {
        self.run_preview_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a role binding.
    async fn check_role_binding_eligibility(
        &self,
        request: tonic::Request<authorization::CheckRoleBindingEligibilityRequest>,
    ) -> Result<tonic::Response<authorization::CheckRoleBindingEligibilityResponse>, tonic::Status> {
        self.run_check_role_binding_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a role binding.
    async fn normalize_role_binding(
        &self,
        request: tonic::Request<authorization::NormalizeRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::NormalizeRoleBindingResponse>, tonic::Status> {
        self.run_normalize_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a role binding.
    async fn explain_role_binding_decision(
        &self,
        request: tonic::Request<authorization::ExplainRoleBindingDecisionRequest>,
    ) -> Result<tonic::Response<authorization::ExplainRoleBindingDecisionResponse>, tonic::Status> {
        self.run_explain_role_binding_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a role binding.
    async fn score_role_binding(
        &self,
        request: tonic::Request<authorization::ScoreRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::ScoreRoleBindingResponse>, tonic::Status> {
        self.run_score_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a role binding.
    async fn lint_role_bindings(
        &self,
        request: tonic::Request<authorization::LintRoleBindingsRequest>,
    ) -> Result<tonic::Response<authorization::LintRoleBindingsResponse>, tonic::Status> {
        self.run_lint_role_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a role binding.
    async fn get_role_binding_rules(
        &self,
        request: tonic::Request<authorization::GetRoleBindingRulesRequest>,
    ) -> Result<tonic::Response<authorization::GetRoleBindingRulesResponse>, tonic::Status> {
        self.run_get_role_binding_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
