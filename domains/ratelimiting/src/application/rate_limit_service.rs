use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::broadcasts::v1::broadcast_query_service_client::BroadcastQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_client::QuotaBucketQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_client::RateLimitQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_server::RateLimitService;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_client::ThrottleCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Manages rate limits: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct RateLimitController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    batch_get_rate_limits_0_port:
        acme_platform::ports::Callback<configuration::LintEnvironmentsRequest, configuration::LintEnvironmentsResponse>,
    update_rate_limit_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::SimulateConfigChangeRequest,
            configuration::SimulateConfigChangeResponse,
        >,
    >,
    watch_rate_limits_0_port: acme_platform::ports::Callback<
        ratelimiting::PreviewQuotaBucketRequest,
        ratelimiting::PreviewQuotaBucketResponse,
    >,
    watch_rate_limits_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::ExplainThrottleDecisionRequest,
            ratelimiting::ExplainThrottleDecisionResponse,
        >,
    >,
    watch_rate_limits_2_port: acme_platform::ports::Callback<
        ratelimiting::EstimateQuotaBucketRequest,
        ratelimiting::EstimateQuotaBucketResponse,
    >,
}
impl RateLimitController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
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
        let update_rate_limit_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::LintEnvironmentsRequest,
            configuration::LintEnvironmentsResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::LintEnvironmentsRequest| -> acme_platform::ports::RpcFuture<configuration::LintEnvironmentsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_environments(request).await?.into_inner())
})
})
        };
        let batch_get_rate_limits_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::PreviewQuotaBucketRequest,
            ratelimiting::PreviewQuotaBucketResponse,
        > = {
            let client = QuotaBucketValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::PreviewQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::PreviewQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_quota_bucket(request).await?.into_inner())
})
})
        };
        let watch_rate_limits_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::ExplainThrottleDecisionRequest,
            ratelimiting::ExplainThrottleDecisionResponse,
        > = {
            let client = ThrottleValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ExplainThrottleDecisionRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ExplainThrottleDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_throttle_decision(request).await?.into_inner())
})
})
        };
        let watch_rate_limits_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::EstimateQuotaBucketRequest,
            ratelimiting::EstimateQuotaBucketResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::EstimateQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::EstimateQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_quota_bucket(request).await?.into_inner())
})
})
        };
        let watch_rate_limits_2_port = callback;
        Self {
            channel,
            repository,
            batch_get_rate_limits_0_port,
            update_rate_limit_0_port,
            watch_rate_limits_0_port,
            watch_rate_limits_1_port,
            watch_rate_limits_2_port,
        }
    }
    /// Run CreateRateLimit after validating the tenant and durably recording the command.
    async fn run_create_rate_limit(
        &self,
        req: ratelimiting::CreateRateLimitRequest,
    ) -> Result<ratelimiting::CreateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().description.clone(), "recorded audit field");
        let command_0 = CreateRateLimitDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::SimulateThrottleRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SimulateThrottleRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CreateRateLimitResponse::default();
        let contribution: String = reply_0
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .rate_limit_id
            .clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: Option<identity::UserRef> = req.rate_limit.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run GetRateLimit after validating the tenant and durably recording the command.
    async fn run_get_rate_limit(
        &self,
        req: ratelimiting::GetRateLimitRequest,
    ) -> Result<ratelimiting::GetRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRateLimit", "running workflow");
        let outgoing_0 = configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                audit: Some(common::AuditInfo {
                    updated_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
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
                        parent_id: req.rate_limit_id.clone(),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::SimulateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = ratelimiting::GetRateLimitResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
    /// Run UpdateRateLimit after validating the tenant and durably recording the command.
    async fn run_update_rate_limit(
        &self,
        req: ratelimiting::UpdateRateLimitRequest,
    ) -> Result<ratelimiting::UpdateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UpdateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UpdateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::SimulateConfigChangeRequest {
            context: req.context.clone(),
            reason: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.update_rate_limit_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = broadcasts::SummarizeBroadcastsRequest {
            annotations: reply_0.result.clone().unwrap_or_default().details.clone(),
            context: req.context.clone(),
            details: Some(broadcasts::summarize_broadcasts_request::Details {
                filters: reply_0.config_change.clone().unwrap_or_default().attributes.clone(),
                period: req.rate_limit.clone().unwrap_or_default().effective.clone(),
                ..Default::default()
            }),
            reason: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = BroadcastQueryServiceClient::new(channel);
            let input: tonic::Request<broadcasts::SummarizeBroadcastsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_broadcasts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::UpdateRateLimitResponse::default();
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
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i64 = req.rate_limit.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
    /// Run DeleteRateLimit after validating the tenant and durably recording the command.
    async fn run_delete_rate_limit(
        &self,
        req: ratelimiting::DeleteRateLimitRequest,
    ) -> Result<ratelimiting::DeleteRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteRateLimit", "running workflow");
        let outgoing_0 = ratelimiting::CompareRateLimitOptionsRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    etag: req.etag.clone(),
                    ..Default::default()
                }),
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
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
                rate_limit_id: req.rate_limit_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.ratelimiting.v1.CompareRateLimitOptionsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CompareRateLimitOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_rate_limit_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::CloneAccessPolicyRequest {
            access_policy: Some(authorization::AccessPolicy {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    etag: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                action: Some(authorization::access_policy::Action::Discount(
                    authorization::AccessPolicyDiscountAction {
                        amount: (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                            Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .amount
                        .clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
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
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<authorization::CloneAccessPolicyRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = AccessPolicyServiceClient::new(channel);
                let input: tonic::Request<authorization::CloneAccessPolicyRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.clone_access_policy(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_1)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_1 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = ratelimiting::DeleteRateLimitResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ListRateLimits after validating the tenant and durably recording the command.
    async fn run_list_rate_limits(
        &self,
        req: ratelimiting::ListRateLimitsRequest,
    ) -> Result<ratelimiting::ListRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = ratelimiting::ListRateLimitsResponse::default();
        Ok(response)
    }
    /// Run BatchGetRateLimits after validating the tenant and durably recording the command.
    async fn run_batch_get_rate_limits(
        &self,
        req: ratelimiting::BatchGetRateLimitsRequest,
    ) -> Result<ratelimiting::BatchGetRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchGetRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchGetRateLimits", "running workflow");
        let outgoing_0 = configuration::LintEnvironmentsRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
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
        let reply_0 = (self.batch_get_rate_limits_0_port)(outgoing_0).await?;
        let mut response = ratelimiting::BatchGetRateLimitsResponse::default();
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .components
            .clone()
            .get("primary")
            .cloned()
            .unwrap_or_default()
            .description
            .clone();
        if contribution != <String>::default() {
            if response.rate_limits.is_empty() {
                response.rate_limits.push(Default::default());
            }
            response.rate_limits[0].description = contribution;
        }
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response.rate_limits.is_empty() {
                response.rate_limits.push(Default::default());
            }
            response.rate_limits[0].audit.get_or_insert_with(Default::default).etag = contribution;
        }
        Ok(response)
    }
    /// Run SearchRateLimits after validating the tenant and durably recording the command.
    async fn run_search_rate_limits(
        &self,
        req: ratelimiting::SearchRateLimitsRequest,
    ) -> Result<ratelimiting::SearchRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SearchRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SearchRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::SimulateThrottleRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::simulate_throttle_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                measurements: vec![ratelimiting::ThrottleMeasurement {
                    observed_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::CountQuotaBucketsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::count_quota_buckets_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ThrottleCalculationServiceClient::new(channel_0);
                let input: tonic::Request<ratelimiting::SimulateThrottleRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.simulate_throttle(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = QuotaBucketQueryServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::CountQuotaBucketsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_quota_buckets(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::SearchRateLimitsResponse::default();
        let contribution: i64 = reply_0
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            if response.rate_limits.is_empty() {
                response.rate_limits.push(Default::default());
            }
            response.rate_limits[0].version = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run WatchRateLimits after validating the tenant and durably recording the command.
    async fn run_watch_rate_limits(
        &self,
        req: ratelimiting::WatchRateLimitsRequest,
    ) -> Result<ratelimiting::WatchRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("WatchRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "WatchRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PreviewQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::preview_quota_bucket_request::Details {
                contact: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.watch_rate_limits_0_port)(outgoing_0).await?;
        let outgoing_1 = ratelimiting::ExplainThrottleDecisionRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_throttle_decision_request::Details {
                contact: Some(common::ContactInfo {
                    preferred_channel: req.subscriber.clone().unwrap_or_default().preferred_channel.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
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
        let reply_1 =
            self.watch_rate_limits_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = ratelimiting::EstimateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    preferred_channel: reply_0
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
                holder: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .rate_limit_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.watch_rate_limits_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let outgoing_3 = authorization::BatchGetAccessPoliciesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_3 =
            acme_platform::ports::RequestBuilder::<authorization::BatchGetAccessPoliciesRequest>::default()
                .map(|draft| *draft = outgoing_3)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = AccessPolicyServiceClient::new(channel);
            let input: tonic::Request<authorization::BatchGetAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::WatchRateLimitsResponse::default();
        let contribution: String = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: i64 = reply_1
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_3.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run BatchCreateRateLimits after validating the tenant and durably recording the command.
    async fn run_batch_create_rate_limits(
        &self,
        req: ratelimiting::BatchCreateRateLimitsRequest,
    ) -> Result<ratelimiting::BatchCreateRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchCreateRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateRateLimits", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(ratelimiting::batch_create_rate_limits_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::EstimateRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
                    email: (match req.chunk.clone() {
                        Some(ratelimiting::batch_create_rate_limits_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                    ..Default::default()
                }),
                action: Some(ratelimiting::rate_limit::Action::Route(
                    ratelimiting::RateLimitRouteAction {
                        target: (match (match req.chunk.clone() {
                            Some(ratelimiting::batch_create_rate_limits_request::Chunk::Record(value)) => value,
                            _ => Default::default(),
                        })
                        .action
                        .clone()
                        {
                            Some(ratelimiting::rate_limit::Action::Route(value)) => value,
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
        let reply_0 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::PushFlagEvaluationRequest {
            context: req.context.clone(),
            details: Some(featureflags::push_flag_evaluation_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: (match req.chunk.clone() {
                        Some(ratelimiting::batch_create_rate_limits_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .display_name
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::PushFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::RecalculateEnvironmentsRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    email: (match req.chunk.clone() {
                        Some(ratelimiting::batch_create_rate_limits_request::Chunk::Header(value)) => value,
                        _ => Default::default(),
                    })
                    .submitted_by
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = EnvironmentCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateEnvironmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_environments(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::BatchCreateRateLimitsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CloneRateLimit after validating the tenant and durably recording the command.
    async fn run_clone_rate_limit(
        &self,
        req: ratelimiting::CloneRateLimitRequest,
    ) -> Result<ratelimiting::CloneRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CloneRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CloneRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::CheckConfigChangeEligibilityRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    observed_at: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }],
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::check_config_change_eligibility_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::PeekRateLimitRequest {
            context: req.context.clone(),
            rate_limit_id: req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeValidationServiceClient::new(channel_0);
                let input: tonic::Request<configuration::CheckConfigChangeEligibilityRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.check_config_change_eligibility(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = RateLimitQueryServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::PeekRateLimitRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.peek_rate_limit(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = venues::ExplainVenueDecisionRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                audit: Some(common::AuditInfo {
                    updated_at: reply_1
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<venues::ExplainVenueDecisionRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::ExplainVenueDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_venue_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CloneRateLimitResponse::default();
        let contribution: String = reply_0.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<identity::UserRef> = reply_1.rate_limit.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).owner = contribution;
        }
        let contribution: String = reply_2
            .venue
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_2.venue.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CreateRateLimit for SimulateThrottle.
struct CreateRateLimitDispatch0 {
    context: Option<common::RequestContext>,
    throttle_subject_version: i64,
    throttle_subject_rate_limit_id: String,
    throttle_subject_display_name: String,
}
impl From<ratelimiting::CreateRateLimitRequest> for CreateRateLimitDispatch0 {
    fn from(req: ratelimiting::CreateRateLimitRequest) -> Self {
        Self {
            context: req.context.clone(),
            throttle_subject_version: req.rate_limit.clone().unwrap_or_default().version.clone(),
            throttle_subject_rate_limit_id: req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(),
            throttle_subject_display_name: req
                .rate_limit
                .clone()
                .unwrap_or_default()
                .owner
                .clone()
                .unwrap_or_default()
                .display_name
                .clone(),
        }
    }
}
impl From<CreateRateLimitDispatch0> for ratelimiting::SimulateThrottleRequest {
    fn from(value: CreateRateLimitDispatch0) -> Self {
        ratelimiting::SimulateThrottleRequest {
            context: value.context,
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: value.throttle_subject_display_name,
                    rate_limit_id: value.throttle_subject_rate_limit_id,
                    version: value.throttle_subject_version,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl RateLimitService for RateLimitController {
    /// Creates a rate limit.
    async fn create_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::CreateRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::CreateRateLimitResponse>, tonic::Status> {
        self.run_create_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a rate limit.
    async fn get_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::GetRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetRateLimitResponse>, tonic::Status> {
        self.run_get_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Changes a rate limit.
    async fn update_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::UpdateRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::UpdateRateLimitResponse>, tonic::Status> {
        self.run_update_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a rate limit.
    async fn delete_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::DeleteRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::DeleteRateLimitResponse>, tonic::Status> {
        self.run_delete_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists rate limits a page at a time.
    async fn list_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::ListRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListRateLimitsResponse>, tonic::Status> {
        self.run_list_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several rate limits at once.
    async fn batch_get_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::BatchGetRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::BatchGetRateLimitsResponse>, tonic::Status> {
        self.run_batch_get_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists rate limits a page at a time.
    async fn search_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::SearchRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::SearchRateLimitsResponse>, tonic::Status> {
        self.run_search_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type WatchRateLimitsStream =
        tokio_stream::wrappers::ReceiverStream<Result<ratelimiting::WatchRateLimitsResponse, tonic::Status>>;
    /// Streams the changes of rate limits as they happen.
    async fn watch_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::WatchRateLimitsRequest>,
    ) -> Result<tonic::Response<Self::WatchRateLimitsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_watch_rate_limits(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Takes a stream of rate limits and answers once at the end.
    async fn batch_create_rate_limits(
        &self,
        request: tonic::Request<tonic::Streaming<ratelimiting::BatchCreateRateLimitsRequest>>,
    ) -> Result<tonic::Response<ratelimiting::BatchCreateRateLimitsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = ratelimiting::BatchCreateRateLimitsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_batch_create_rate_limits(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Creates a rate limit.
    async fn clone_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::CloneRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::CloneRateLimitResponse>, tonic::Status> {
        self.run_clone_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
