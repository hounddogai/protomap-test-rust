use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::configuration::v1::config_set_approval_service_client::ConfigSetApprovalServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::insurance::v1 as insurance;
use acme_proto::acme::insurance::v1::policy_service_client::PolicyServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_lifecycle_service_client::QuotaBucketLifecycleServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_approval_service_client::RateLimitApprovalServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_workflow_service_server::RateLimitWorkflowService;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_validation_service_client::SearchIndexValidationServiceClient;
use acme_proto::acme::yards::v1 as yards;
use acme_proto::acme::yards::v1::yard_calculation_service_client::YardCalculationServiceClient;

/// Runs the long-lived workflow that takes a rate limit from start to finish.
#[derive(Clone)]
pub struct RateLimitWorkflowHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    retry_rate_limit_workflow_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::CommentConfigSetApprovalRequest,
            configuration::CommentConfigSetApprovalResponse,
        >,
    >,
    retry_rate_limit_workflow_3_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::SubmitQuotaBucketRequest,
                ratelimiting::SubmitQuotaBucketResponse,
            >,
    >,
    signal_rate_limit_workflow_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::QuoteQuotaBucketRequest, ratelimiting::QuoteQuotaBucketResponse>,
    >,
    signal_rate_limit_workflow_2_port:
        acme_platform::ports::Callback<yards::EstimateYardRequest, yards::EstimateYardResponse>,
}
impl RateLimitWorkflowHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::CommentConfigSetApprovalRequest,
            configuration::CommentConfigSetApprovalResponse,
        > = {
            let client = ConfigSetApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CommentConfigSetApprovalRequest| -> acme_platform::ports::RpcFuture<configuration::CommentConfigSetApprovalResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.comment_config_set_approval(request).await?.into_inner())
})
})
        };
        let retry_rate_limit_workflow_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::SubmitQuotaBucketRequest,
            ratelimiting::SubmitQuotaBucketResponse,
        > = {
            let client = QuotaBucketApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::SubmitQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::SubmitQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.submit_quota_bucket(request).await?.into_inner())
})
})
        };
        let retry_rate_limit_workflow_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::QuoteQuotaBucketRequest,
            ratelimiting::QuoteQuotaBucketResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::QuoteQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::QuoteQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_quota_bucket(request).await?.into_inner())
})
})
        };
        let signal_rate_limit_workflow_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<yards::EstimateYardRequest, yards::EstimateYardResponse> = {
            let client = YardCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: yards::EstimateYardRequest| -> acme_platform::ports::RpcFuture<yards::EstimateYardResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_yard(request).await?.into_inner())
})
})
        };
        let signal_rate_limit_workflow_2_port = callback;
        Self {
            channel,
            repository,
            retry_rate_limit_workflow_2_port,
            retry_rate_limit_workflow_3_port,
            signal_rate_limit_workflow_1_port,
            signal_rate_limit_workflow_2_port,
        }
    }
    /// Run StartRateLimitWorkflow after validating the tenant and durably recording the command.
    async fn run_start_rate_limit_workflow(
        &self,
        req: ratelimiting::StartRateLimitWorkflowRequest,
    ) -> Result<ratelimiting::StartRateLimitWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StartRateLimitWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartRateLimitWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let command_0 = StartRateLimitWorkflowDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::ScheduleQuotaBucketExpiryRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketLifecycleServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ScheduleQuotaBucketExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_quota_bucket_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::CalculateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                holder: Some(ratelimiting::RateLimitRef {
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.ratelimiting.v1.CalculateQuotaBucketRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CalculateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = insurance::SearchPoliciesRequest {
            context: req.context.clone(),
            filter: Some(insurance::search_policies_request::Filter {
                created_after: reply_0
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: reply_0.quota_bucket.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<insurance::SearchPoliciesRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = PolicyServiceClient::new(channel);
                let input: tonic::Request<insurance::SearchPoliciesRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.search_policies(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_2)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_2 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let outgoing_3 = ratelimiting::ScheduleThrottleReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(ratelimiting::schedule_throttle_report_request::Details {
                filters: reply_2.result.clone().unwrap_or_default().details.clone(),
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
        let reply_3 = acme_platform::transport::deadline(async move {
            let mut client = ThrottleReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ScheduleThrottleReportRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_throttle_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = ratelimiting::StartRateLimitWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: i32 = req.rate_limit.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceRateLimitWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_rate_limit_workflow(
        &self,
        req: ratelimiting::AdvanceRateLimitWorkflowRequest,
    ) -> Result<ratelimiting::AdvanceRateLimitWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceRateLimitWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceRateLimitWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PullRateLimitsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::pull_rate_limits_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_by: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                action: Some(ratelimiting::rate_limit::Action::Discount(
                    ratelimiting::RateLimitDiscountAction {
                        percent: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                            Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .percent
                        .clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PullRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pull_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::RecallRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::recall_rate_limit_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    user_id: req
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
            rate_limit: Some(ratelimiting::RateLimit {
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
                action: Some(ratelimiting::rate_limit::Action::Route(
                    ratelimiting::RateLimitRouteAction {
                        target: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
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
        let outgoing_1 = acme_platform::ports::RequestBuilder::<ratelimiting::RecallRateLimitRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitApprovalServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RecallRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recall_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::AdvanceRateLimitWorkflowResponse::default();
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
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run CancelRateLimitWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_rate_limit_workflow(
        &self,
        req: ratelimiting::CancelRateLimitWorkflowRequest,
    ) -> Result<ratelimiting::CancelRateLimitWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelRateLimitWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelRateLimitWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PullRateLimitsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::pull_rate_limits_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PullRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pull_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CancelRateLimitWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: i64 = req.rate_limit.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        Ok(response)
    }
    /// Run RetryRateLimitWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_rate_limit_workflow(
        &self,
        req: ratelimiting::RetryRateLimitWorkflowRequest,
    ) -> Result<ratelimiting::RetryRateLimitWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryRateLimitWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryRateLimitWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = yards::EstimateYardRequest {
            context: req.context.clone(),
            yard: Some(yards::Yard {
                audit: Some(common::AuditInfo {
                    etag: req
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = featureflags::NotifyFlagEvaluationWatchersRequest {
            context: req.context.clone(),
            details: Some(featureflags::notify_flag_evaluation_watchers_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = YardCalculationServiceClient::new(channel_0);
                let input: tonic::Request<yards::EstimateYardRequest> = tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.estimate_yard(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = FlagEvaluationNotificationServiceClient::new(channel_1);
                let input: tonic::Request<featureflags::NotifyFlagEvaluationWatchersRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.notify_flag_evaluation_watchers(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = configuration::CommentConfigSetApprovalRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            country_code: (match reply_0.yard.clone().unwrap_or_default().locator.clone() {
                                Some(yards::yard::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .country_code
                            .clone(),
                            lines: (match reply_0.yard.clone().unwrap_or_default().locator.clone() {
                                Some(yards::yard::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .lines
                            .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.retry_rate_limit_workflow_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = ratelimiting::SubmitQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::submit_quota_bucket_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_0
                        .yard
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
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .yard
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_at: reply_2
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(ratelimiting::RateLimitRef {
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
            ..Default::default()
        };
        let reply_3 = acme_platform::ports::execute(self.retry_rate_limit_workflow_3_port.as_ref(), outgoing_3).await?;
        let mut response = ratelimiting::RetryRateLimitWorkflowResponse::default();
        let contribution: String = reply_0
            .yard
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .config_set
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
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
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
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.rate_limit.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run PauseRateLimitWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_rate_limit_workflow(
        &self,
        req: ratelimiting::PauseRateLimitWorkflowRequest,
    ) -> Result<ratelimiting::PauseRateLimitWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PauseRateLimitWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseRateLimitWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamRateLimitMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                    Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                    _ => Default::default(),
                })
                .notify_email
                .clone(),
                name: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                phone: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
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
        let _reply_0 = async move {
            let mut client = RateLimitReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamRateLimitMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_rate_limit_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamRateLimitMetricsResponse::default();
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
        let mut response = ratelimiting::PauseRateLimitWorkflowResponse::default();
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.rate_limit.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run InspectRateLimitWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_rate_limit_workflow(
        &self,
        req: ratelimiting::InspectRateLimitWorkflowRequest,
    ) -> Result<ratelimiting::InspectRateLimitWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectRateLimitWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectRateLimitWorkflow", "running workflow");
        let outgoing_0 = yards::RecalculateYardsRequest {
            context: req.context.clone(),
            yard: Some(yards::Yard {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.yards.v1.RecalculateYardsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = YardCalculationServiceClient::new(channel);
            let input: tonic::Request<yards::RecalculateYardsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_yards(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::InspectRateLimitWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .yard
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
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
    /// Run ListRateLimitWorkflows after validating the tenant and durably recording the command.
    async fn run_list_rate_limit_workflows(
        &self,
        req: ratelimiting::ListRateLimitWorkflowsRequest,
    ) -> Result<ratelimiting::ListRateLimitWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListRateLimitWorkflows", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRateLimitWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = search::ScoreSearchIndexRequest {
            context: req.context.clone(),
            search_index: Some(search::SearchIndex {
                locator: Some(search::search_index::Locator::Address(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                })),
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<search::ScoreSearchIndexRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = SearchIndexValidationServiceClient::new(channel);
                let input: tonic::Request<search::ScoreSearchIndexRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.score_search_index(input).await?;
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
        let mut response = ratelimiting::ListRateLimitWorkflowsResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .search_index
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.rate_limits.is_empty() {
                response.rate_limits.push(Default::default());
            }
            response.rate_limits[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_0
            .search_index
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            if response.rate_limits.is_empty() {
                response.rate_limits.push(Default::default());
            }
            response.rate_limits[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        Ok(response)
    }
    /// Run SignalRateLimitWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_rate_limit_workflow(
        &self,
        req: ratelimiting::SignalRateLimitWorkflowRequest,
    ) -> Result<ratelimiting::SignalRateLimitWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalRateLimitWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalRateLimitWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamThrottleMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                phone: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = ThrottleReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamThrottleMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
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
        let outgoing_1 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: req
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
                holder: Some(ratelimiting::RateLimitRef {
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
            ..Default::default()
        };
        let reply_1 =
            self.signal_rate_limit_workflow_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = yards::EstimateYardRequest {
            context: req.context.clone(),
            yard: Some(yards::Yard {
                audit: Some(common::AuditInfo {
                    etag: req
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
                owner: Some(identity::UserRef {
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
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.signal_rate_limit_workflow_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let outgoing_3 = configuration::CancelConfigChangeWorkflowRequest {
            context: req.context.clone(),
            details: Some(configuration::cancel_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    phone: reply_1
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
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
        let outgoing_3 =
            acme_platform::ports::RequestBuilder::<configuration::CancelConfigChangeWorkflowRequest>::default()
                .map(|draft| *draft = outgoing_3)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::CancelConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.cancel_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::SignalRateLimitWorkflowResponse::default();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: i32 = req.rate_limit.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by StartRateLimitWorkflow for ScheduleQuotaBucketExpiry.
struct StartRateLimitWorkflowDispatch0 {
    context: Option<common::RequestContext>,
    quota_bucket_audit_updated_at: Option<prost_types::Timestamp>,
}
impl From<ratelimiting::StartRateLimitWorkflowRequest> for StartRateLimitWorkflowDispatch0 {
    fn from(req: ratelimiting::StartRateLimitWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            quota_bucket_audit_updated_at: req
                .rate_limit
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
        }
    }
}
impl From<StartRateLimitWorkflowDispatch0> for ratelimiting::ScheduleQuotaBucketExpiryRequest {
    fn from(value: StartRateLimitWorkflowDispatch0) -> Self {
        ratelimiting::ScheduleQuotaBucketExpiryRequest {
            context: value.context,
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    updated_at: value.quota_bucket_audit_updated_at,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl RateLimitWorkflowService for RateLimitWorkflowHandler {
    /// Starts a rate limit.
    async fn start_rate_limit_workflow(
        &self,
        request: tonic::Request<ratelimiting::StartRateLimitWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::StartRateLimitWorkflowResponse>, tonic::Status> {
        self.run_start_rate_limit_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances a rate limit.
    async fn advance_rate_limit_workflow(
        &self,
        request: tonic::Request<ratelimiting::AdvanceRateLimitWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::AdvanceRateLimitWorkflowResponse>, tonic::Status> {
        self.run_advance_rate_limit_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a rate limit.
    async fn cancel_rate_limit_workflow(
        &self,
        request: tonic::Request<ratelimiting::CancelRateLimitWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::CancelRateLimitWorkflowResponse>, tonic::Status> {
        self.run_cancel_rate_limit_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a rate limit.
    async fn retry_rate_limit_workflow(
        &self,
        request: tonic::Request<ratelimiting::RetryRateLimitWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::RetryRateLimitWorkflowResponse>, tonic::Status> {
        self.run_retry_rate_limit_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses a rate limit.
    async fn pause_rate_limit_workflow(
        &self,
        request: tonic::Request<ratelimiting::PauseRateLimitWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::PauseRateLimitWorkflowResponse>, tonic::Status> {
        self.run_pause_rate_limit_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a rate limit.
    async fn inspect_rate_limit_workflow(
        &self,
        request: tonic::Request<ratelimiting::InspectRateLimitWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::InspectRateLimitWorkflowResponse>, tonic::Status> {
        self.run_inspect_rate_limit_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists rate limits a page at a time.
    async fn list_rate_limit_workflows(
        &self,
        request: tonic::Request<ratelimiting::ListRateLimitWorkflowsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListRateLimitWorkflowsResponse>, tonic::Status> {
        self.run_list_rate_limit_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals a rate limit.
    async fn signal_rate_limit_workflow(
        &self,
        request: tonic::Request<ratelimiting::SignalRateLimitWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::SignalRateLimitWorkflowResponse>, tonic::Status> {
        self.run_signal_rate_limit_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
