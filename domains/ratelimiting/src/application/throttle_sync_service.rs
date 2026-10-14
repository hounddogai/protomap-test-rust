use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::admissions::v1::admission_calculation_service_client::AdmissionCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_query_service_client::UserQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_client::ThrottleImportServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_server::ThrottleSyncService;
use acme_proto::acme::workplaces::v1 as workplaces;
use acme_proto::acme::workplaces::v1::building_query_service_client::BuildingQueryServiceClient;

/// Keeps throttles in step with the systems of record in other regions.
#[derive(Clone)]
pub struct ThrottleSyncApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    get_throttle_sync_state_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            admissions::RecalculateAdmissionsRequest,
            admissions::RecalculateAdmissionsResponse,
        >,
    >,
    list_throttle_conflicts_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::PullRateLimitsRequest, ratelimiting::PullRateLimitsResponse>,
    >,
    pull_throttles_0_port: acme_platform::ports::Callback<
        ratelimiting::PreviewQuotaBucketRequest,
        ratelimiting::PreviewQuotaBucketResponse,
    >,
    resolve_throttle_conflict_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::CompareFlagEvaluationOptionsRequest,
            featureflags::CompareFlagEvaluationOptionsResponse,
        >,
    >,
    stream_throttle_changes_0_port: acme_platform::ports::Callback<
        ratelimiting::ListRateLimitConflictsRequest,
        ratelimiting::ListRateLimitConflictsResponse,
    >,
    stream_throttle_changes_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::PullQuotaBucketsRequest, ratelimiting::PullQuotaBucketsResponse>,
    >,
    stream_throttle_changes_2_port: acme_platform::ports::Callback<
        ratelimiting::ListRateLimitReportsRequest,
        ratelimiting::ListRateLimitReportsResponse,
    >,
}
impl ThrottleSyncApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
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
        let pull_throttles_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::ListRateLimitConflictsRequest,
            ratelimiting::ListRateLimitConflictsResponse,
        > = {
            let client = RateLimitSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ListRateLimitConflictsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ListRateLimitConflictsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_rate_limit_conflicts(request).await?.into_inner())
})
})
        };
        let stream_throttle_changes_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::PullQuotaBucketsRequest,
            ratelimiting::PullQuotaBucketsResponse,
        > = {
            let client = QuotaBucketSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::PullQuotaBucketsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::PullQuotaBucketsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.pull_quota_buckets(request).await?.into_inner())
})
})
        };
        let stream_throttle_changes_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::ListRateLimitReportsRequest,
            ratelimiting::ListRateLimitReportsResponse,
        > = {
            let client = RateLimitReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ListRateLimitReportsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ListRateLimitReportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_rate_limit_reports(request).await?.into_inner())
})
})
        };
        let stream_throttle_changes_2_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::CompareFlagEvaluationOptionsRequest,
            featureflags::CompareFlagEvaluationOptionsResponse,
        > = {
            let client = FlagEvaluationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::CompareFlagEvaluationOptionsRequest| -> acme_platform::ports::RpcFuture<featureflags::CompareFlagEvaluationOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_flag_evaluation_options(request).await?.into_inner())
})
})
        };
        let resolve_throttle_conflict_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            admissions::RecalculateAdmissionsRequest,
            admissions::RecalculateAdmissionsResponse,
        > = {
            let client = AdmissionCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: admissions::RecalculateAdmissionsRequest| -> acme_platform::ports::RpcFuture<admissions::RecalculateAdmissionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_admissions(request).await?.into_inner())
})
})
        };
        let get_throttle_sync_state_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let list_throttle_conflicts_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            get_throttle_sync_state_0_port,
            list_throttle_conflicts_1_port,
            pull_throttles_0_port,
            resolve_throttle_conflict_0_port,
            stream_throttle_changes_0_port,
            stream_throttle_changes_1_port,
            stream_throttle_changes_2_port,
        }
    }
    /// Run SyncThrottles after validating the tenant and durably recording the command.
    async fn run_sync_throttles(
        &self,
        req: ratelimiting::SyncThrottlesRequest,
    ) -> Result<ratelimiting::SyncThrottlesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SyncThrottles", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SyncThrottles", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::DeleteRateLimitRequest {
            context: req.context.clone(),
            rate_limit_id: req
                .throttle
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .rate_limit_id
                .clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<ratelimiting::DeleteRateLimitRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::DeleteRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.delete_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::SyncThrottlesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.throttle.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.throttle.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run PushThrottle after validating the tenant and durably recording the command.
    async fn run_push_throttle(
        &self,
        req: ratelimiting::PushThrottleRequest,
    ) -> Result<ratelimiting::PushThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PushThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PushThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let command_0 = PushThrottleDispatch0::from(req.clone());
        let outgoing_0: identity::PeekUserRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = UserQueryServiceClient::new(channel);
            let input: tonic::Request<identity::PeekUserRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.peek_user(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::QuoteFlagEvaluationRequest {
            context: req.context.clone(),
            details: Some(featureflags::quote_flag_evaluation_request::Details {
                destination: Some(common::PostalAddress {
                    lines: reply_0
                        .user
                        .clone()
                        .unwrap_or_default()
                        .primary_address
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.featureflags.v1.QuoteFlagEvaluationRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagEvaluationCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::QuoteFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::PushThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: String = req.throttle.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run PullThrottles after validating the tenant and durably recording the command.
    async fn run_pull_throttles(
        &self,
        req: ratelimiting::PullThrottlesRequest,
    ) -> Result<ratelimiting::PullThrottlesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PullThrottles", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PullThrottles", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PreviewQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    rate_limit_id: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .rate_limit_id
                        .clone(),
                    version: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .version
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.pull_throttles_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = ratelimiting::ExplainRateLimitDecisionRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_rate_limit_decision_request::Details {
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
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                version: req
                    .throttle
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .version
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<ratelimiting::ExplainRateLimitDecisionRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainRateLimitDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_rate_limit_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::PullThrottlesResponse::default();
        let contribution: i64 = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run ReconcileThrottles after validating the tenant and durably recording the command.
    async fn run_reconcile_throttles(
        &self,
        req: ratelimiting::ReconcileThrottlesRequest,
    ) -> Result<ratelimiting::ReconcileThrottlesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReconcileThrottles", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReconcileThrottles", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().kind.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::GetThrottleImportStatusRequest {
            context: req.context.clone(),
            throttle_id: req.throttle.clone().unwrap_or_default().throttle_id.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ratelimiting::GetThrottleImportStatusRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ThrottleImportServiceClient::new(channel);
                let input: tonic::Request<ratelimiting::GetThrottleImportStatusRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.get_throttle_import_status(input).await?;
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
        let outgoing_1 = ratelimiting::CloneRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
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
                rate_limit_id: req
                    .throttle
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .rate_limit_id
                    .clone(),
                version: req
                    .throttle
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .version
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = RateLimitServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CloneRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = ratelimiting::SimulateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
                holder: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .rate_limit_id
                        .clone(),
                    version: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .version
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SimulateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::NormalizeRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::normalize_rate_limit_request::Details {
                contact: Some(common::ContactInfo {
                    phone: reply_2
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
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_by: reply_1
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::NormalizeRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ReconcileThrottlesResponse::default();
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
            response
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .rate_limit_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
    /// Run StreamThrottleChanges after validating the tenant and durably recording the command.
    async fn run_stream_throttle_changes(
        &self,
        req: ratelimiting::StreamThrottleChangesRequest,
    ) -> Result<ratelimiting::StreamThrottleChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StreamThrottleChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamThrottleChanges", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(ratelimiting::stream_throttle_changes_request::Frame::Change(value)) => value, _ => Default::default() }).kind.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ListRateLimitConflictsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_rate_limit_conflicts_request::Filter {
                created_after: (match req.frame.clone() {
                    Some(ratelimiting::stream_throttle_changes_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .occurred_at
                .clone(),
                owner_id: (match req.frame.clone() {
                    Some(ratelimiting::stream_throttle_changes_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .throttle_id
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.stream_throttle_changes_0_port)(outgoing_0).await?;
        let outgoing_1 = ratelimiting::PullQuotaBucketsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::pull_quota_buckets_request::Details {
                cursor: req.cursor.clone(),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: (match req.frame.clone() {
                        Some(ratelimiting::stream_throttle_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .display_name
                    .clone(),
                    rate_limit_id: (match req.frame.clone() {
                        Some(ratelimiting::stream_throttle_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .rate_limit_id
                    .clone(),
                    version: (match req.frame.clone() {
                        Some(ratelimiting::stream_throttle_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .version
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            self.stream_throttle_changes_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = ratelimiting::ListRateLimitReportsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_rate_limit_reports_request::Filter {
                labels: reply_1.quota_bucket.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.stream_throttle_changes_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let mut response = ratelimiting::StreamThrottleChangesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = (match req.frame.clone() {
            Some(ratelimiting::stream_throttle_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .throttle_id
        .clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: i64 = (match req.frame.clone() {
            Some(ratelimiting::stream_throttle_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .sequence
        .clone();
        if contribution != <i64>::default() {
            response.sequence = contribution;
        }
        Ok(response)
    }
    /// Run ResolveThrottleConflict after validating the tenant and durably recording the command.
    async fn run_resolve_throttle_conflict(
        &self,
        req: ratelimiting::ResolveThrottleConflictRequest,
    ) -> Result<ratelimiting::ResolveThrottleConflictResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResolveThrottleConflict", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveThrottleConflict", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CompareFlagEvaluationOptionsRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                attributes: req.throttle.clone().unwrap_or_default().attributes.clone(),
                occurred_at: req.throttle.clone().unwrap_or_default().occurred_at.clone(),
                subject: Some(featureflags::FlagRolloutRef {
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
        let reply_0 = acme_platform::ports::execute(self.resolve_throttle_conflict_0_port.as_ref(), outgoing_0).await?;
        let jobs_1 = req
            .throttle
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_0 = reply_0.clone();
                async move {
                    let outgoing_1 = identity::SuggestUsersRequest {
                        context: req.context.clone(),
                        filter: Some(identity::suggest_users_request::Filter {
                            created_after: item.observed_at.clone(),
                            labels: reply_0.flag_evaluation.clone().unwrap_or_default().attributes.clone(),
                            owner_id: req
                                .throttle
                                .clone()
                                .unwrap_or_default()
                                .subject
                                .clone()
                                .unwrap_or_default()
                                .rate_limit_id
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = UserQueryServiceClient::new(channel);
                        let input: tonic::Request<identity::SuggestUsersRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.suggest_users(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = ratelimiting::ResolveThrottleConflictResponse::default();
        let contribution: String = reply_0.flag_evaluation.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).kind = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.flag_evaluation.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
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
    /// Run GetThrottleSyncState after validating the tenant and durably recording the command.
    async fn run_get_throttle_sync_state(
        &self,
        req: ratelimiting::GetThrottleSyncStateRequest,
    ) -> Result<ratelimiting::GetThrottleSyncStateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetThrottleSyncState", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetThrottleSyncState", "running workflow");
        let outgoing_0 = admissions::RecalculateAdmissionsRequest {
            admission: Some(admissions::Admission {
                source: req.throttle_id.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.get_throttle_sync_state_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = workplaces::ExportBuildingsRequest {
            context: req.context.clone(),
            filter: Some(workplaces::export_buildings_request::Filter {
                created_after: reply_0.admission.clone().unwrap_or_default().occurred_at.clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0.admission.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = BuildingQueryServiceClient::new(channel);
            let input: tonic::Request<workplaces::ExportBuildingsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_buildings(input).await?;
            let mut frames = response.into_inner();
            let mut last = workplaces::ExportBuildingsResponse::default();
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
        let mut response = ratelimiting::GetThrottleSyncStateResponse::default();
        let contribution: i64 = reply_0.admission.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.throttle.get_or_insert_with(Default::default).sequence = contribution;
        }
        let contribution: String = reply_0
            .admission
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
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
    /// Run ListThrottleConflicts after validating the tenant and durably recording the command.
    async fn run_list_throttle_conflicts(
        &self,
        req: ratelimiting::ListThrottleConflictsRequest,
    ) -> Result<ratelimiting::ListThrottleConflictsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListThrottleConflicts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListThrottleConflicts", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PushQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::push_quota_bucket_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
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
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::PullRateLimitsRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
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
            }),
            ..Default::default()
        };
        let reply_1 =
            self.list_throttle_conflicts_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = ratelimiting::ListThrottleConflictsResponse::default();
        let contribution: String = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .rate_limit_id
            .clone();
        if contribution != <String>::default() {
            if response.throttles.is_empty() {
                response.throttles.push(Default::default());
            }
            response.throttles[0]
                .subject
                .get_or_insert_with(Default::default)
                .rate_limit_id = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by PushThrottle for PeekUser.
struct PushThrottleDispatch0 {
    context: Option<common::RequestContext>,
    user_id: String,
}
impl From<ratelimiting::PushThrottleRequest> for PushThrottleDispatch0 {
    fn from(req: ratelimiting::PushThrottleRequest) -> Self {
        Self {
            context: req.context.clone(),
            user_id: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
        }
    }
}
impl From<PushThrottleDispatch0> for identity::PeekUserRequest {
    fn from(value: PushThrottleDispatch0) -> Self {
        identity::PeekUserRequest {
            context: value.context,
            user_id: value.user_id,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ThrottleSyncService for ThrottleSyncApi {
    /// Syncs a throttle.
    async fn sync_throttles(
        &self,
        request: tonic::Request<ratelimiting::SyncThrottlesRequest>,
    ) -> Result<tonic::Response<ratelimiting::SyncThrottlesResponse>, tonic::Status> {
        self.run_sync_throttles(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pushs a throttle.
    async fn push_throttle(
        &self,
        request: tonic::Request<ratelimiting::PushThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::PushThrottleResponse>, tonic::Status> {
        self.run_push_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pulls a throttle.
    async fn pull_throttles(
        &self,
        request: tonic::Request<ratelimiting::PullThrottlesRequest>,
    ) -> Result<tonic::Response<ratelimiting::PullThrottlesResponse>, tonic::Status> {
        self.run_pull_throttles(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reconciles a throttle.
    async fn reconcile_throttles(
        &self,
        request: tonic::Request<ratelimiting::ReconcileThrottlesRequest>,
    ) -> Result<tonic::Response<ratelimiting::ReconcileThrottlesResponse>, tonic::Status> {
        self.run_reconcile_throttles(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamThrottleChangesStream =
        tokio_stream::wrappers::ReceiverStream<Result<ratelimiting::StreamThrottleChangesResponse, tonic::Status>>;
    /// Exchanges changes of throttles with a peer.
    async fn stream_throttle_changes(
        &self,
        request: tonic::Request<tonic::Streaming<ratelimiting::StreamThrottleChangesRequest>>,
    ) -> Result<tonic::Response<Self::StreamThrottleChangesStream>, tonic::Status> {
        let handler = self.clone();
        let mut incoming = request.into_inner();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let received = tokio::select! { _ = sender.closed() => break, value = incoming.message() => value };
                let result = match received {
                    Ok(Some(request)) => handler.run_stream_throttle_changes(request).await,
                    Ok(None) => break,
                    Err(error) => Err(error),
                };
                let failed = result.is_err();
                if sender.send(result).await.is_err() || failed {
                    break;
                }
            }
        });
        Ok(tonic::Response::new(tokio_stream::wrappers::ReceiverStream::new(
            receiver,
        )))
    }
    /// Resolves a throttle.
    async fn resolve_throttle_conflict(
        &self,
        request: tonic::Request<ratelimiting::ResolveThrottleConflictRequest>,
    ) -> Result<tonic::Response<ratelimiting::ResolveThrottleConflictResponse>, tonic::Status> {
        self.run_resolve_throttle_conflict(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a throttle.
    async fn get_throttle_sync_state(
        &self,
        request: tonic::Request<ratelimiting::GetThrottleSyncStateRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetThrottleSyncStateResponse>, tonic::Status> {
        self.run_get_throttle_sync_state(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists throttles a page at a time.
    async fn list_throttle_conflicts(
        &self,
        request: tonic::Request<ratelimiting::ListThrottleConflictsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListThrottleConflictsResponse>, tonic::Status> {
        self.run_list_throttle_conflicts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
