use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_server::FlagEvaluationValidationService;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Checks flag evaluations against the rules before anything acts on them.
#[derive(Clone)]
pub struct FlagEvaluationValidationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagEvaluationValidationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ValidateFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_validate_flag_evaluation(
        &self,
        req: featureflags::ValidateFlagEvaluationRequest,
    ) -> Result<featureflags::ValidateFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::ValidateFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.flag_evaluation.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PreviewFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_preview_flag_evaluation(
        &self,
        req: featureflags::PreviewFlagEvaluationRequest,
    ) -> Result<featureflags::PreviewFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::PreviewFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.flag_evaluation.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run CheckFlagEvaluationEligibility after validating the tenant and durably recording the command.
    async fn run_check_flag_evaluation_eligibility(
        &self,
        req: featureflags::CheckFlagEvaluationEligibilityRequest,
    ) -> Result<featureflags::CheckFlagEvaluationEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckFlagEvaluationEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckFlagEvaluationEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::CheckFlagEvaluationEligibilityResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_normalize_flag_evaluation(
        &self,
        req: featureflags::NormalizeFlagEvaluationRequest,
    ) -> Result<featureflags::NormalizeFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NormalizeFlagEvaluation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let jobs_0 = req
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = venues::CloneVenueRequest {
                        context: req.context.clone(),
                        venue: Some(venues::Venue {
                            dimensions: Some(venues::VenueDimensions {
                                unit: item.unit.clone(),
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
                    async move {
                        let mut client = VenueServiceClient::new(channel);
                        let input: tonic::Request<venues::CloneVenueRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.clone_venue(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = featureflags::NormalizeFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.flag_evaluation.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run ExplainFlagEvaluationDecision after validating the tenant and durably recording the command.
    async fn run_explain_flag_evaluation_decision(
        &self,
        req: featureflags::ExplainFlagEvaluationDecisionRequest,
    ) -> Result<featureflags::ExplainFlagEvaluationDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainFlagEvaluationDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainFlagEvaluationDecision", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::ExplainFlagEvaluationDecisionResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ScoreFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_score_flag_evaluation(
        &self,
        req: featureflags::ScoreFlagEvaluationRequest,
    ) -> Result<featureflags::ScoreFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::ScoreFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<featureflags::FlagRolloutRef> =
            req.flag_evaluation.clone().unwrap_or_default().subject.clone();
        if contribution != <Option<featureflags::FlagRolloutRef>>::default() {
            response.flag_evaluation.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run LintFlagEvaluations after validating the tenant and durably recording the command.
    async fn run_lint_flag_evaluations(
        &self,
        req: featureflags::LintFlagEvaluationsRequest,
    ) -> Result<featureflags::LintFlagEvaluationsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintFlagEvaluations", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintFlagEvaluations", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::LintFlagEvaluationsResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run GetFlagEvaluationRules after validating the tenant and durably recording the command.
    async fn run_get_flag_evaluation_rules(
        &self,
        req: featureflags::GetFlagEvaluationRulesRequest,
    ) -> Result<featureflags::GetFlagEvaluationRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFlagEvaluationRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagEvaluationRules", "running workflow");
        let mut response = featureflags::GetFlagEvaluationRulesResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FlagEvaluationValidationService for FlagEvaluationValidationEndpoint {
    /// Validates a flag evaluation.
    async fn validate_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::ValidateFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::ValidateFlagEvaluationResponse>, tonic::Status> {
        self.run_validate_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a flag evaluation.
    async fn preview_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::PreviewFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::PreviewFlagEvaluationResponse>, tonic::Status> {
        self.run_preview_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a flag evaluation.
    async fn check_flag_evaluation_eligibility(
        &self,
        request: tonic::Request<featureflags::CheckFlagEvaluationEligibilityRequest>,
    ) -> Result<tonic::Response<featureflags::CheckFlagEvaluationEligibilityResponse>, tonic::Status> {
        self.run_check_flag_evaluation_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a flag evaluation.
    async fn normalize_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::NormalizeFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::NormalizeFlagEvaluationResponse>, tonic::Status> {
        self.run_normalize_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a flag evaluation.
    async fn explain_flag_evaluation_decision(
        &self,
        request: tonic::Request<featureflags::ExplainFlagEvaluationDecisionRequest>,
    ) -> Result<tonic::Response<featureflags::ExplainFlagEvaluationDecisionResponse>, tonic::Status> {
        self.run_explain_flag_evaluation_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a flag evaluation.
    async fn score_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::ScoreFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::ScoreFlagEvaluationResponse>, tonic::Status> {
        self.run_score_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a flag evaluation.
    async fn lint_flag_evaluations(
        &self,
        request: tonic::Request<featureflags::LintFlagEvaluationsRequest>,
    ) -> Result<tonic::Response<featureflags::LintFlagEvaluationsResponse>, tonic::Status> {
        self.run_lint_flag_evaluations(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag evaluation.
    async fn get_flag_evaluation_rules(
        &self,
        request: tonic::Request<featureflags::GetFlagEvaluationRulesRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagEvaluationRulesResponse>, tonic::Status> {
        self.run_get_flag_evaluation_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
