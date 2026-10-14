use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::authorization::v1::role_binding_validation_service_client::RoleBindingValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_export_service_client::FeatureFlagExportServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_server::ThrottleValidationService;

/// Checks throttles against the rules before anything acts on them.
#[derive(Clone)]
pub struct ThrottleValidationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    lint_throttles_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::CheckEnvironmentEligibilityRequest,
            configuration::CheckEnvironmentEligibilityResponse,
        >,
    >,
    preview_throttle_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<configuration::CreateConfigSetRequest, configuration::CreateConfigSetResponse>,
    >,
    score_throttle_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::SimulateConfigChangeRequest,
            configuration::SimulateConfigChangeResponse,
        >,
    >,
    score_throttle_1_port: acme_platform::ports::Callback<
        featureflags::ListFeatureFlagExportsRequest,
        featureflags::ListFeatureFlagExportsResponse,
    >,
}
impl ThrottleValidationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::CreateConfigSetRequest,
            configuration::CreateConfigSetResponse,
        > = {
            let client = ConfigSetServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CreateConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::CreateConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.create_config_set(request).await?.into_inner())
})
})
        };
        let preview_throttle_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::SimulateConfigChangeRequest,
            configuration::SimulateConfigChangeResponse,
        > = {
            let client = ConfigChangeCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SimulateConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::SimulateConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_config_change(request).await?.into_inner())
})
})
        };
        let score_throttle_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            featureflags::ListFeatureFlagExportsRequest,
            featureflags::ListFeatureFlagExportsResponse,
        > = {
            let client = FeatureFlagExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ListFeatureFlagExportsRequest| -> acme_platform::ports::RpcFuture<featureflags::ListFeatureFlagExportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_feature_flag_exports(request).await?.into_inner())
})
})
        };
        let score_throttle_1_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::CheckEnvironmentEligibilityRequest,
            configuration::CheckEnvironmentEligibilityResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CheckEnvironmentEligibilityRequest| -> acme_platform::ports::RpcFuture<configuration::CheckEnvironmentEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_environment_eligibility(request).await?.into_inner())
})
})
        };
        let lint_throttles_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            lint_throttles_2_port,
            preview_throttle_0_port,
            score_throttle_0_port,
            score_throttle_1_port,
        }
    }
    /// Run ValidateThrottle after validating the tenant and durably recording the command.
    async fn run_validate_throttle(
        &self,
        req: ratelimiting::ValidateThrottleRequest,
    ) -> Result<ratelimiting::ValidateThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::EstimateConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                source: req.throttle.clone().unwrap_or_default().source.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<configuration::EstimateConfigChangeRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::EstimateConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ValidateThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: i64 = reply_0.config_change.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.throttle.get_or_insert_with(Default::default).sequence = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.throttle.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PreviewThrottle after validating the tenant and durably recording the command.
    async fn run_preview_throttle(
        &self,
        req: ratelimiting::PreviewThrottleRequest,
    ) -> Result<ratelimiting::PreviewThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CreateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        children: vec![configuration::ConfigSet {
                            attributes: req.throttle.clone().unwrap_or_default().attributes.clone(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        display_name: req
                            .throttle
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
                }],
                manager: Some(identity::UserRef {
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
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.preview_throttle_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = authorization::CompareAccessPolicyVersionsRequest {
            access_policy: Some(authorization::AccessPolicy {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .config_set
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
            context: req.context.clone(),
            details: Some(authorization::compare_access_policy_versions_request::Details {
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessPolicyHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::CompareAccessPolicyVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_access_policy_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::PreviewThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: String = req.throttle.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run CheckThrottleEligibility after validating the tenant and durably recording the command.
    async fn run_check_throttle_eligibility(
        &self,
        req: ratelimiting::CheckThrottleEligibilityRequest,
    ) -> Result<ratelimiting::CheckThrottleEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckThrottleEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckThrottleEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let command_0 = CheckThrottleEligibilityDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::QuoteRateLimitRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::QuoteRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ScoreEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    display_name: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: reply_0
                        .rate_limit
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
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.configuration.v1.ScoreEnvironmentRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ScoreEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CheckThrottleEligibilityResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.rate_limit.clone().unwrap_or_default().rate_limit_id.clone();
        if contribution != <String>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .rate_limit_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: String = req.throttle.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run ExplainThrottleDecision after validating the tenant and durably recording the command.
    async fn run_explain_throttle_decision(
        &self,
        req: ratelimiting::ExplainThrottleDecisionRequest,
    ) -> Result<ratelimiting::ExplainThrottleDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainThrottleDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainThrottleDecision", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::PreviewConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                sequence: req.throttle.clone().unwrap_or_default().sequence.clone(),
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
                        .throttle
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
            context: req.context.clone(),
            details: Some(configuration::preview_config_change_request::Details {
                contact: Some(common::ContactInfo {
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::PreviewConfigChangeRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigChangeValidationServiceClient::new(channel);
                let input: tonic::Request<configuration::PreviewConfigChangeRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.preview_config_change(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_0)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_0 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = ratelimiting::ExplainThrottleDecisionResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.throttle.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ScoreThrottle after validating the tenant and durably recording the command.
    async fn run_score_throttle(
        &self,
        req: ratelimiting::ScoreThrottleRequest,
    ) -> Result<ratelimiting::ScoreThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SimulateConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::simulate_config_change_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    locality: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    region: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .region
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.score_throttle_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = featureflags::ListFeatureFlagExportsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_exports_request::Filter {
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.score_throttle_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = ratelimiting::ScoreThrottleResponse::default();
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
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: i64 = req.throttle.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.throttle.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run LintThrottles after validating the tenant and durably recording the command.
    async fn run_lint_throttles(
        &self,
        req: ratelimiting::LintThrottlesRequest,
    ) -> Result<ratelimiting::LintThrottlesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintThrottles", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintThrottles", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::CheckRoleBindingEligibilityRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
                counterparty: Some(authorization::AccessPolicyRef {
                    display_name: req
                        .throttle
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
        let reply_0 = async move {
            let mut client = RoleBindingValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::CheckRoleBindingEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_role_binding_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::ListAccessPolicyVersionsRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_access_policy_versions_request::Filter {
                created_after: reply_0
                    .role_binding
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: (reply_0.role_binding.clone().unwrap_or_default().metadata.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                region: reply_0
                    .role_binding
                    .clone()
                    .unwrap_or_default()
                    .billing_address
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessPolicyHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::ListAccessPolicyVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_access_policy_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::CheckEnvironmentEligibilityRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .role_binding
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
        let reply_2 = acme_platform::ports::execute(self.lint_throttles_2_port.as_ref(), outgoing_2).await?;
        let mut response = ratelimiting::LintThrottlesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_2.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: i64 = req.throttle.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.throttle.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run GetThrottleRules after validating the tenant and durably recording the command.
    async fn run_get_throttle_rules(
        &self,
        req: ratelimiting::GetThrottleRulesRequest,
    ) -> Result<ratelimiting::GetThrottleRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetThrottleRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetThrottleRules", "running workflow");
        let outgoing_0 = featureflags::ScheduleFlagEvaluationExportRequest {
            context: req.context.clone(),
            details: Some(featureflags::schedule_flag_evaluation_export_request::Details {
                filters: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = FlagEvaluationExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::ScheduleFlagEvaluationExportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_flag_evaluation_export(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = ratelimiting::GetThrottleRulesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CheckThrottleEligibility for QuoteRateLimit.
struct CheckThrottleEligibilityDispatch0 {
    context: Option<common::RequestContext>,
    rate_limit_owner_display_name: String,
    rate_limit_block_notify_email: String,
    rate_limit_block_reason: String,
    rate_limit_version: i64,
}
impl From<ratelimiting::CheckThrottleEligibilityRequest> for CheckThrottleEligibilityDispatch0 {
    fn from(req: ratelimiting::CheckThrottleEligibilityRequest) -> Self {
        Self {
            context: req.context.clone(),
            rate_limit_owner_display_name: req
                .throttle
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .display_name
                .clone(),
            rate_limit_block_notify_email: req
                .details
                .clone()
                .unwrap_or_default()
                .contact
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            rate_limit_block_reason: req.reason.clone(),
            rate_limit_version: req
                .throttle
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .version
                .clone(),
        }
    }
}
impl From<CheckThrottleEligibilityDispatch0> for ratelimiting::QuoteRateLimitRequest {
    fn from(value: CheckThrottleEligibilityDispatch0) -> Self {
        ratelimiting::QuoteRateLimitRequest {
            context: value.context,
            rate_limit: Some(ratelimiting::RateLimit {
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        notify_email: value.rate_limit_block_notify_email,
                        reason: value.rate_limit_block_reason,
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
                    display_name: value.rate_limit_owner_display_name,
                    ..Default::default()
                }),
                version: value.rate_limit_version,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ThrottleValidationService for ThrottleValidationEndpoint {
    /// Validates a throttle.
    async fn validate_throttle(
        &self,
        request: tonic::Request<ratelimiting::ValidateThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::ValidateThrottleResponse>, tonic::Status> {
        self.run_validate_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a throttle.
    async fn preview_throttle(
        &self,
        request: tonic::Request<ratelimiting::PreviewThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::PreviewThrottleResponse>, tonic::Status> {
        self.run_preview_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a throttle.
    async fn check_throttle_eligibility(
        &self,
        request: tonic::Request<ratelimiting::CheckThrottleEligibilityRequest>,
    ) -> Result<tonic::Response<ratelimiting::CheckThrottleEligibilityResponse>, tonic::Status> {
        self.run_check_throttle_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a throttle.
    async fn explain_throttle_decision(
        &self,
        request: tonic::Request<ratelimiting::ExplainThrottleDecisionRequest>,
    ) -> Result<tonic::Response<ratelimiting::ExplainThrottleDecisionResponse>, tonic::Status> {
        self.run_explain_throttle_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a throttle.
    async fn score_throttle(
        &self,
        request: tonic::Request<ratelimiting::ScoreThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::ScoreThrottleResponse>, tonic::Status> {
        self.run_score_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a throttle.
    async fn lint_throttles(
        &self,
        request: tonic::Request<ratelimiting::LintThrottlesRequest>,
    ) -> Result<tonic::Response<ratelimiting::LintThrottlesResponse>, tonic::Status> {
        self.run_lint_throttles(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a throttle.
    async fn get_throttle_rules(
        &self,
        request: tonic::Request<ratelimiting::GetThrottleRulesRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetThrottleRulesResponse>, tonic::Status> {
        self.run_get_throttle_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
