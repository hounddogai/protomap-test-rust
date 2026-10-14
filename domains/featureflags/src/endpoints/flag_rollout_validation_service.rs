use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_server::FlagRolloutValidationService;
use acme_proto::acme::identity::v1 as identity;

/// Checks flag rollouts against the rules before anything acts on them.
#[derive(Clone)]
pub struct FlagRolloutValidationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagRolloutValidationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ValidateFlagRollout after validating the tenant and durably recording the command.
    async fn run_validate_flag_rollout(
        &self,
        req: featureflags::ValidateFlagRolloutRequest,
    ) -> Result<featureflags::ValidateFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::ValidateFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.flag_rollout.clone().unwrap_or_default().window.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).window = contribution;
        }
        Ok(response)
    }
    /// Run PreviewFlagRollout after validating the tenant and durably recording the command.
    async fn run_preview_flag_rollout(
        &self,
        req: featureflags::PreviewFlagRolloutRequest,
    ) -> Result<featureflags::PreviewFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::PushFeatureFlagRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    updated_by: req
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: req
                        .details
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
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::PushFeatureFlagRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_feature_flag(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::PreviewFlagRolloutResponse::default();
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.flag_rollout.clone().unwrap_or_default().window.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).window = contribution;
        }
        Ok(response)
    }
    /// Run CheckFlagRolloutEligibility after validating the tenant and durably recording the command.
    async fn run_check_flag_rollout_eligibility(
        &self,
        req: featureflags::CheckFlagRolloutEligibilityRequest,
    ) -> Result<featureflags::CheckFlagRolloutEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckFlagRolloutEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckFlagRolloutEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::CheckFlagRolloutEligibilityResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: String = req.flag_rollout.clone().unwrap_or_default().title.clone();
        if contribution != <String>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).title = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeFlagRollout after validating the tenant and durably recording the command.
    async fn run_normalize_flag_rollout(
        &self,
        req: featureflags::NormalizeFlagRolloutRequest,
    ) -> Result<featureflags::NormalizeFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::NormalizeFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.flag_rollout.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ExplainFlagRolloutDecision after validating the tenant and durably recording the command.
    async fn run_explain_flag_rollout_decision(
        &self,
        req: featureflags::ExplainFlagRolloutDecisionRequest,
    ) -> Result<featureflags::ExplainFlagRolloutDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainFlagRolloutDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainFlagRolloutDecision", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::ExplainFlagRolloutDecisionResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: String = req.flag_rollout.clone().unwrap_or_default().title.clone();
        if contribution != <String>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).title = contribution;
        }
        Ok(response)
    }
    /// Run ScoreFlagRollout after validating the tenant and durably recording the command.
    async fn run_score_flag_rollout(
        &self,
        req: featureflags::ScoreFlagRolloutRequest,
    ) -> Result<featureflags::ScoreFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::ScoreFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: i32 = req.flag_rollout.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run LintFlagRollouts after validating the tenant and durably recording the command.
    async fn run_lint_flag_rollouts(
        &self,
        req: featureflags::LintFlagRolloutsRequest,
    ) -> Result<featureflags::LintFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::LintFlagRolloutsResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.flag_rollout.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run GetFlagRolloutRules after validating the tenant and durably recording the command.
    async fn run_get_flag_rollout_rules(
        &self,
        req: featureflags::GetFlagRolloutRulesRequest,
    ) -> Result<featureflags::GetFlagRolloutRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFlagRolloutRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagRolloutRules", "running workflow");
        let mut response = featureflags::GetFlagRolloutRulesResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { FlagRolloutValidationService, FlagRolloutValidationEndpoint;
unary {
validate_flag_rollout (featureflags::ValidateFlagRolloutRequest) -> featureflags::ValidateFlagRolloutResponse => run_validate_flag_rollout;
preview_flag_rollout (featureflags::PreviewFlagRolloutRequest) -> featureflags::PreviewFlagRolloutResponse => run_preview_flag_rollout;
check_flag_rollout_eligibility (featureflags::CheckFlagRolloutEligibilityRequest) -> featureflags::CheckFlagRolloutEligibilityResponse => run_check_flag_rollout_eligibility;
normalize_flag_rollout (featureflags::NormalizeFlagRolloutRequest) -> featureflags::NormalizeFlagRolloutResponse => run_normalize_flag_rollout;
explain_flag_rollout_decision (featureflags::ExplainFlagRolloutDecisionRequest) -> featureflags::ExplainFlagRolloutDecisionResponse => run_explain_flag_rollout_decision;
score_flag_rollout (featureflags::ScoreFlagRolloutRequest) -> featureflags::ScoreFlagRolloutResponse => run_score_flag_rollout;
lint_flag_rollouts (featureflags::LintFlagRolloutsRequest) -> featureflags::LintFlagRolloutsResponse => run_lint_flag_rollouts;
get_flag_rollout_rules (featureflags::GetFlagRolloutRulesRequest) -> featureflags::GetFlagRolloutRulesResponse => run_get_flag_rollout_rules;
}
server {
}
client {
}
bidi {
}
}
