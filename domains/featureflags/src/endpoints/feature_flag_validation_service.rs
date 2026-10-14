use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_server::FeatureFlagValidationService;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Checks feature flags against the rules before anything acts on them.
#[derive(Clone)]
pub struct FeatureFlagValidationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FeatureFlagValidationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ValidateFeatureFlag after validating the tenant and durably recording the command.
    async fn run_validate_feature_flag(
        &self,
        req: featureflags::ValidateFeatureFlagRequest,
    ) -> Result<featureflags::ValidateFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::ValidateFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.feature_flag.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PreviewFeatureFlag after validating the tenant and durably recording the command.
    async fn run_preview_feature_flag(
        &self,
        req: featureflags::PreviewFeatureFlagRequest,
    ) -> Result<featureflags::PreviewFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::PreviewFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.feature_flag.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CheckFeatureFlagEligibility after validating the tenant and durably recording the command.
    async fn run_check_feature_flag_eligibility(
        &self,
        req: featureflags::CheckFeatureFlagEligibilityRequest,
    ) -> Result<featureflags::CheckFeatureFlagEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckFeatureFlagEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckFeatureFlagEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let outgoing_0 = venues::ExplainVenueDecisionRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                audit: Some(common::AuditInfo {
                    etag: req
                        .feature_flag
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_by: req
                        .feature_flag
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::ExplainVenueDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_venue_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::CheckFeatureFlagEligibilityResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req.feature_flag.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeFeatureFlag after validating the tenant and durably recording the command.
    async fn run_normalize_feature_flag(
        &self,
        req: featureflags::NormalizeFeatureFlagRequest,
    ) -> Result<featureflags::NormalizeFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::NormalizeFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req.feature_flag.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run ExplainFeatureFlagDecision after validating the tenant and durably recording the command.
    async fn run_explain_feature_flag_decision(
        &self,
        req: featureflags::ExplainFeatureFlagDecisionRequest,
    ) -> Result<featureflags::ExplainFeatureFlagDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainFeatureFlagDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainFeatureFlagDecision", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::ExplainFeatureFlagDecisionResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: i32 = req.feature_flag.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.feature_flag.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ScoreFeatureFlag after validating the tenant and durably recording the command.
    async fn run_score_feature_flag(
        &self,
        req: featureflags::ScoreFeatureFlagRequest,
    ) -> Result<featureflags::ScoreFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::ScoreFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run LintFeatureFlags after validating the tenant and durably recording the command.
    async fn run_lint_feature_flags(
        &self,
        req: featureflags::LintFeatureFlagsRequest,
    ) -> Result<featureflags::LintFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::LintFeatureFlagsResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req.feature_flag.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run GetFeatureFlagRules after validating the tenant and durably recording the command.
    async fn run_get_feature_flag_rules(
        &self,
        req: featureflags::GetFeatureFlagRulesRequest,
    ) -> Result<featureflags::GetFeatureFlagRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFeatureFlagRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFeatureFlagRules", "running workflow");
        let mut response = featureflags::GetFeatureFlagRulesResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { FeatureFlagValidationService, FeatureFlagValidationHandler;
unary {
validate_feature_flag (featureflags::ValidateFeatureFlagRequest) -> featureflags::ValidateFeatureFlagResponse => run_validate_feature_flag;
preview_feature_flag (featureflags::PreviewFeatureFlagRequest) -> featureflags::PreviewFeatureFlagResponse => run_preview_feature_flag;
check_feature_flag_eligibility (featureflags::CheckFeatureFlagEligibilityRequest) -> featureflags::CheckFeatureFlagEligibilityResponse => run_check_feature_flag_eligibility;
normalize_feature_flag (featureflags::NormalizeFeatureFlagRequest) -> featureflags::NormalizeFeatureFlagResponse => run_normalize_feature_flag;
explain_feature_flag_decision (featureflags::ExplainFeatureFlagDecisionRequest) -> featureflags::ExplainFeatureFlagDecisionResponse => run_explain_feature_flag_decision;
score_feature_flag (featureflags::ScoreFeatureFlagRequest) -> featureflags::ScoreFeatureFlagResponse => run_score_feature_flag;
lint_feature_flags (featureflags::LintFeatureFlagsRequest) -> featureflags::LintFeatureFlagsResponse => run_lint_feature_flags;
get_feature_flag_rules (featureflags::GetFeatureFlagRulesRequest) -> featureflags::GetFeatureFlagRulesResponse => run_get_feature_flag_rules;
}
server {
}
client {
}
bidi {
}
}
