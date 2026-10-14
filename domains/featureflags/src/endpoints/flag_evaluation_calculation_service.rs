use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_server::FlagEvaluationCalculationService;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;

/// Computes the figures of a flag evaluation, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct FlagEvaluationCalculationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    explain_flag_evaluation_calculation_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::RecalculateFlagRolloutsRequest,
            featureflags::RecalculateFlagRolloutsResponse,
        >,
    >,
}
impl FlagEvaluationCalculationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::RecalculateFlagRolloutsRequest,
            featureflags::RecalculateFlagRolloutsResponse,
        > = {
            let client = FlagRolloutCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::RecalculateFlagRolloutsRequest| -> acme_platform::ports::RpcFuture<featureflags::RecalculateFlagRolloutsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_flag_rollouts(request).await?.into_inner())
})
})
        };
        let explain_flag_evaluation_calculation_0_port =
            std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            explain_flag_evaluation_calculation_0_port,
        }
    }
    /// Run CalculateFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_calculate_flag_evaluation(
        &self,
        req: featureflags::CalculateFlagEvaluationRequest,
    ) -> Result<featureflags::CalculateFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CalculateFlagEvaluation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::CalculateFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.flag_evaluation.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run EstimateFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_estimate_flag_evaluation(
        &self,
        req: featureflags::EstimateFlagEvaluationRequest,
    ) -> Result<featureflags::EstimateFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::EstimateFlagEvaluationResponse::default();
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
    /// Run QuoteFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_quote_flag_evaluation(
        &self,
        req: featureflags::QuoteFlagEvaluationRequest,
    ) -> Result<featureflags::QuoteFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::QuoteFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run CompareFlagEvaluationOptions after validating the tenant and durably recording the command.
    async fn run_compare_flag_evaluation_options(
        &self,
        req: featureflags::CompareFlagEvaluationOptionsRequest,
    ) -> Result<featureflags::CompareFlagEvaluationOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareFlagEvaluationOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareFlagEvaluationOptions", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::CompareFlagEvaluationOptionsResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateFlagEvaluations after validating the tenant and durably recording the command.
    async fn run_recalculate_flag_evaluations(
        &self,
        req: featureflags::RecalculateFlagEvaluationsRequest,
    ) -> Result<featureflags::RecalculateFlagEvaluationsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecalculateFlagEvaluations", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateFlagEvaluations", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::RecalculateFlagEvaluationsResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run SimulateFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_simulate_flag_evaluation(
        &self,
        req: featureflags::SimulateFlagEvaluationRequest,
    ) -> Result<featureflags::SimulateFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::SimulateFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: i64 = req.flag_evaluation.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.flag_evaluation.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run ExplainFlagEvaluationCalculation after validating the tenant and durably recording the command.
    async fn run_explain_flag_evaluation_calculation(
        &self,
        req: featureflags::ExplainFlagEvaluationCalculationRequest,
    ) -> Result<featureflags::ExplainFlagEvaluationCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainFlagEvaluationCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainFlagEvaluationCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::RecalculateFlagRolloutsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        preferred_channel: req
                            .details
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.explain_flag_evaluation_calculation_0_port.as_ref(), outgoing_0).await?;
        let mut response = featureflags::ExplainFlagEvaluationCalculationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
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
    /// Run GetFlagEvaluationRates after validating the tenant and durably recording the command.
    async fn run_get_flag_evaluation_rates(
        &self,
        req: featureflags::GetFlagEvaluationRatesRequest,
    ) -> Result<featureflags::GetFlagEvaluationRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFlagEvaluationRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagEvaluationRates", "running workflow");
        let mut response = featureflags::GetFlagEvaluationRatesResponse::default();
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
impl FlagEvaluationCalculationService for FlagEvaluationCalculationApi {
    /// Calculates a flag evaluation.
    async fn calculate_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::CalculateFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::CalculateFlagEvaluationResponse>, tonic::Status> {
        self.run_calculate_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a flag evaluation.
    async fn estimate_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::EstimateFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::EstimateFlagEvaluationResponse>, tonic::Status> {
        self.run_estimate_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a flag evaluation.
    async fn quote_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::QuoteFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::QuoteFlagEvaluationResponse>, tonic::Status> {
        self.run_quote_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a flag evaluation.
    async fn compare_flag_evaluation_options(
        &self,
        request: tonic::Request<featureflags::CompareFlagEvaluationOptionsRequest>,
    ) -> Result<tonic::Response<featureflags::CompareFlagEvaluationOptionsResponse>, tonic::Status> {
        self.run_compare_flag_evaluation_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a flag evaluation.
    async fn recalculate_flag_evaluations(
        &self,
        request: tonic::Request<featureflags::RecalculateFlagEvaluationsRequest>,
    ) -> Result<tonic::Response<featureflags::RecalculateFlagEvaluationsResponse>, tonic::Status> {
        self.run_recalculate_flag_evaluations(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a flag evaluation.
    async fn simulate_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::SimulateFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::SimulateFlagEvaluationResponse>, tonic::Status> {
        self.run_simulate_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a flag evaluation.
    async fn explain_flag_evaluation_calculation(
        &self,
        request: tonic::Request<featureflags::ExplainFlagEvaluationCalculationRequest>,
    ) -> Result<tonic::Response<featureflags::ExplainFlagEvaluationCalculationResponse>, tonic::Status> {
        self.run_explain_flag_evaluation_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag evaluation.
    async fn get_flag_evaluation_rates(
        &self,
        request: tonic::Request<featureflags::GetFlagEvaluationRatesRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagEvaluationRatesResponse>, tonic::Status> {
        self.run_get_flag_evaluation_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
