use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::assets::v1::fixed_asset_calculation_service_client::FixedAssetCalculationServiceClient;
use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_query_service_client::RoleBindingQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::lineage::v1 as lineage;
use acme_proto::acme::lineage::v1::lineage_node_query_service_client::LineageNodeQueryServiceClient;
use acme_proto::acme::promotions::v1 as promotions;
use acme_proto::acme::promotions::v1::promotion_service_client::PromotionServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_lifecycle_service_server::QuotaBucketLifecycleService;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;

/// Moves a quota bucket between the states of its lifecycle.
#[derive(Clone)]
pub struct QuotaBucketLifecycleApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    expire_quota_bucket_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<lineage::SummarizeLineageNodesRequest, lineage::SummarizeLineageNodesResponse>,
    >,
    reinstate_quota_bucket_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::ScheduleThrottleReportRequest,
                ratelimiting::ScheduleThrottleReportResponse,
            >,
    >,
    reinstate_quota_bucket_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::PullRateLimitsRequest, ratelimiting::PullRateLimitsResponse>,
    >,
    renew_quota_bucket_0_port:
        acme_platform::ports::Callback<promotions::ClonePromotionRequest, promotions::ClonePromotionResponse>,
    suspend_quota_bucket_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::ExplainFeatureFlagDecisionRequest,
            featureflags::ExplainFeatureFlagDecisionResponse,
        >,
    >,
    terminate_quota_bucket_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<assets::QuoteFixedAssetRequest, assets::QuoteFixedAssetResponse>,
    >,
    terminate_quota_bucket_1_port: acme_platform::ports::Callback<
        configuration::SubmitConfigChangeRequest,
        configuration::SubmitConfigChangeResponse,
    >,
}
impl QuotaBucketLifecycleApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::ExplainFeatureFlagDecisionRequest,
            featureflags::ExplainFeatureFlagDecisionResponse,
        > = {
            let client = FeatureFlagValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ExplainFeatureFlagDecisionRequest| -> acme_platform::ports::RpcFuture<featureflags::ExplainFeatureFlagDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_feature_flag_decision(request).await?.into_inner())
})
})
        };
        let suspend_quota_bucket_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::ScheduleThrottleReportRequest,
            ratelimiting::ScheduleThrottleReportResponse,
        > = {
            let client = ThrottleReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ScheduleThrottleReportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ScheduleThrottleReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.schedule_throttle_report(request).await?.into_inner())
})
})
        };
        let reinstate_quota_bucket_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::PullRateLimitsRequest,
            ratelimiting::PullRateLimitsResponse,
        > = {
            let client = RateLimitSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::PullRateLimitsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::PullRateLimitsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.pull_rate_limits(request).await?.into_inner())
})
})
        };
        let reinstate_quota_bucket_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<assets::QuoteFixedAssetRequest, assets::QuoteFixedAssetResponse> = {
            let client = FixedAssetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: assets::QuoteFixedAssetRequest| -> acme_platform::ports::RpcFuture<assets::QuoteFixedAssetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_fixed_asset(request).await?.into_inner())
})
})
        };
        let terminate_quota_bucket_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::SubmitConfigChangeRequest,
            configuration::SubmitConfigChangeResponse,
        > = {
            let client = ConfigChangeApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SubmitConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::SubmitConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.submit_config_change(request).await?.into_inner())
})
})
        };
        let terminate_quota_bucket_1_port = callback;
        let callback: acme_platform::ports::Callback<
            promotions::ClonePromotionRequest,
            promotions::ClonePromotionResponse,
        > = {
            let client = PromotionServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: promotions::ClonePromotionRequest| -> acme_platform::ports::RpcFuture<promotions::ClonePromotionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_promotion(request).await?.into_inner())
})
})
        };
        let renew_quota_bucket_0_port = callback;
        let callback: acme_platform::ports::Callback<
            lineage::SummarizeLineageNodesRequest,
            lineage::SummarizeLineageNodesResponse,
        > = {
            let client = LineageNodeQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: lineage::SummarizeLineageNodesRequest| -> acme_platform::ports::RpcFuture<lineage::SummarizeLineageNodesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.summarize_lineage_nodes(request).await?.into_inner())
})
})
        };
        let expire_quota_bucket_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            expire_quota_bucket_0_port,
            reinstate_quota_bucket_0_port,
            reinstate_quota_bucket_2_port,
            renew_quota_bucket_0_port,
            suspend_quota_bucket_0_port,
            terminate_quota_bucket_0_port,
            terminate_quota_bucket_1_port,
        }
    }
    /// Run ActivateQuotaBucket after validating the tenant and durably recording the command.
    async fn run_activate_quota_bucket(
        &self,
        req: ratelimiting::ActivateQuotaBucketRequest,
    ) -> Result<ratelimiting::ActivateQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ActivateQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ActivateQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::RecalculateRateLimitsRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RecalculateRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::QuoteQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ActivateQuotaBucketResponse::default();
        let contribution: String = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run SuspendQuotaBucket after validating the tenant and durably recording the command.
    async fn run_suspend_quota_bucket(
        &self,
        req: ratelimiting::SuspendQuotaBucketRequest,
    ) -> Result<ratelimiting::SuspendQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuspendQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuspendQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ExplainFeatureFlagDecisionRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_feature_flag_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.suspend_quota_bucket_0_port.as_ref(), outgoing_0).await?;
        let mut response = ratelimiting::SuspendQuotaBucketResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ReinstateQuotaBucket after validating the tenant and durably recording the command.
    async fn run_reinstate_quota_bucket(
        &self,
        req: ratelimiting::ReinstateQuotaBucketRequest,
    ) -> Result<ratelimiting::ReinstateQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReinstateQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReinstateQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ScheduleThrottleReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(ratelimiting::schedule_throttle_report_request::Details {
                filters: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.reinstate_quota_bucket_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = authorization::SuggestRoleBindingsRequest {
            context: req.context.clone(),
            filter: Some(authorization::suggest_role_bindings_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: (reply_0.result.clone().unwrap_or_default().details.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .new_owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RoleBindingQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::SuggestRoleBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_role_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::PullRateLimitsRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let reply_2 =
            self.reinstate_quota_bucket_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let mut response = ratelimiting::ReinstateQuotaBucketResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
            .rate_limit
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TerminateQuotaBucket after validating the tenant and durably recording the command.
    async fn run_terminate_quota_bucket(
        &self,
        req: ratelimiting::TerminateQuotaBucketRequest,
    ) -> Result<ratelimiting::TerminateQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TerminateQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TerminateQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = assets::QuoteFixedAssetRequest {
            context: req.context.clone(),
            fixed_asset: Some(assets::FixedAsset {
                owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let reply_0 = acme_platform::ports::execute(self.terminate_quota_bucket_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::SubmitConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    unit: reply_0
                        .fixed_asset
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .unit
                        .clone(),
                    ..Default::default()
                }],
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::submit_config_change_request::Details {
                approver: Some(identity::UserRef {
                    email: reply_0
                        .fixed_asset
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let reply_1 = (self.terminate_quota_bucket_1_port)(outgoing_1).await?;
        let mut response = ratelimiting::TerminateQuotaBucketResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RenewQuotaBucket after validating the tenant and durably recording the command.
    async fn run_renew_quota_bucket(
        &self,
        req: ratelimiting::RenewQuotaBucketRequest,
    ) -> Result<ratelimiting::RenewQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RenewQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RenewQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = promotions::ClonePromotionRequest {
            context: req.context.clone(),
            promotion: Some(promotions::Promotion {
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let port = acme_platform::ports::opaque(self.renew_quota_bucket_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = authorization::ResolveRoleBindingsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<authorization::ResolveRoleBindingsRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RoleBindingQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::ResolveRoleBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_role_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = RenewQuotaBucketDispatch2::from(req.clone());
        command_2.filter_created_after = reply_0
            .promotion
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        command_2.filter_labels = reply_0.promotion.clone().unwrap_or_default().parameters.clone();
        command_2.filter_owner_id = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        let outgoing_2: configuration::ListEnvironmentReportsRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = EnvironmentReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ListEnvironmentReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_environment_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::PushRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
                        .promotion
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
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                action: Some(ratelimiting::rate_limit::Action::Route(
                    ratelimiting::RateLimitRouteAction {
                        target: (match reply_0.promotion.clone().unwrap_or_default().action.clone() {
                            Some(promotions::promotion::Action::Route(value)) => value,
                            _ => Default::default(),
                        })
                        .target
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::RenewQuotaBucketResponse::default();
        let contribution: String = reply_0
            .promotion
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_3.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ExpireQuotaBucket after validating the tenant and durably recording the command.
    async fn run_expire_quota_bucket(
        &self,
        req: ratelimiting::ExpireQuotaBucketRequest,
    ) -> Result<ratelimiting::ExpireQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExpireQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExpireQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = lineage::SummarizeLineageNodesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(lineage::summarize_lineage_nodes_request::Details {
                filters: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.expire_quota_bucket_0_port.as_ref(), outgoing_0).await?;
        let mut response = ratelimiting::ExpireQuotaBucketResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run GetQuotaBucketLifecycle after validating the tenant and durably recording the command.
    async fn run_get_quota_bucket_lifecycle(
        &self,
        req: ratelimiting::GetQuotaBucketLifecycleRequest,
    ) -> Result<ratelimiting::GetQuotaBucketLifecycleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetQuotaBucketLifecycle", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetQuotaBucketLifecycle", "running workflow");
        let mut response = ratelimiting::GetQuotaBucketLifecycleResponse::default();
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleQuotaBucketExpiry after validating the tenant and durably recording the command.
    async fn run_schedule_quota_bucket_expiry(
        &self,
        req: ratelimiting::ScheduleQuotaBucketExpiryRequest,
    ) -> Result<ratelimiting::ScheduleQuotaBucketExpiryResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleQuotaBucketExpiry", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleQuotaBucketExpiry", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::AdvanceConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        updated_by: req
                            .quota_bucket
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_by
                            .clone(),
                        ..Default::default()
                    }),
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: req
                                .quota_bucket
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
                    }],
                    manager: Some(identity::UserRef {
                        email: req
                            .quota_bucket
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::AdvanceConfigSetWorkflowRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigSetWorkflowServiceClient::new(channel);
                let input: tonic::Request<configuration::AdvanceConfigSetWorkflowRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.advance_config_set_workflow(input).await?;
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
        let outgoing_1 = ratelimiting::StreamThrottleMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                preferred_channel: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
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
        let outgoing_2 = audit::ResolveAuditEventsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AuditEventQueryServiceClient::new(channel);
            let input: tonic::Request<audit::ResolveAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::ExplainFlagEvaluationCalculationRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_flag_evaluation_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    phone: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .quota_bucket
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
        let reply_3 = async move {
            let mut client = FlagEvaluationCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExplainFlagEvaluationCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_flag_evaluation_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ScheduleQuotaBucketExpiryResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_3.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: String = req.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by RenewQuotaBucket for ListEnvironmentReports.
struct RenewQuotaBucketDispatch2 {
    context: Option<common::RequestContext>,
    filter_created_after: Option<prost_types::Timestamp>,
    filter_labels: std::collections::BTreeMap<String, String>,
    filter_owner_id: String,
}
impl From<ratelimiting::RenewQuotaBucketRequest> for RenewQuotaBucketDispatch2 {
    fn from(req: ratelimiting::RenewQuotaBucketRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_created_after: Default::default(),
            filter_labels: Default::default(),
            filter_owner_id: Default::default(),
        }
    }
}
impl From<RenewQuotaBucketDispatch2> for configuration::ListEnvironmentReportsRequest {
    fn from(value: RenewQuotaBucketDispatch2) -> Self {
        configuration::ListEnvironmentReportsRequest {
            context: value.context,
            filter: Some(configuration::list_environment_reports_request::Filter {
                created_after: value.filter_created_after,
                labels: value.filter_labels,
                owner_id: value.filter_owner_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl QuotaBucketLifecycleService for QuotaBucketLifecycleApi {
    /// Activates a quota bucket.
    async fn activate_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::ActivateQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::ActivateQuotaBucketResponse>, tonic::Status> {
        self.run_activate_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Suspends a quota bucket.
    async fn suspend_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::SuspendQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::SuspendQuotaBucketResponse>, tonic::Status> {
        self.run_suspend_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reinstates a quota bucket.
    async fn reinstate_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::ReinstateQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::ReinstateQuotaBucketResponse>, tonic::Status> {
        self.run_reinstate_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Terminates a quota bucket.
    async fn terminate_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::TerminateQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::TerminateQuotaBucketResponse>, tonic::Status> {
        self.run_terminate_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Renews a quota bucket.
    async fn renew_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::RenewQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::RenewQuotaBucketResponse>, tonic::Status> {
        self.run_renew_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Expires a quota bucket.
    async fn expire_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::ExpireQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::ExpireQuotaBucketResponse>, tonic::Status> {
        self.run_expire_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a quota bucket.
    async fn get_quota_bucket_lifecycle(
        &self,
        request: tonic::Request<ratelimiting::GetQuotaBucketLifecycleRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetQuotaBucketLifecycleResponse>, tonic::Status> {
        self.run_get_quota_bucket_lifecycle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a quota bucket.
    async fn schedule_quota_bucket_expiry(
        &self,
        request: tonic::Request<ratelimiting::ScheduleQuotaBucketExpiryRequest>,
    ) -> Result<tonic::Response<ratelimiting::ScheduleQuotaBucketExpiryResponse>, tonic::Status> {
        self.run_schedule_quota_bucket_expiry(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
