use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::assets::v1::fixed_asset_query_service_client::FixedAssetQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_query_service_client::AccessPolicyQueryServiceClient;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::broadcasts::v1::broadcast_query_service_client::BroadcastQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::lending::v1 as lending;
use acme_proto::acme::lending::v1::loan_application_query_service_client::LoanApplicationQueryServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_validation_service_client::NotificationValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_approval_service_server::ThrottleApprovalService;
use acme_proto::acme::ratelimiting::v1::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_version_query_service_client::SecretVersionQueryServiceClient;

/// Routes throttles through approval, with escalation and delegation.
#[derive(Clone)]
pub struct ThrottleApprovalController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    approve_throttle_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::ScheduleThrottleExpiryRequest,
            ratelimiting::ScheduleThrottleExpiryResponse,
        >,
    >,
    delegate_throttle_approval_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::ShareConfigChangeReportRequest,
                configuration::ShareConfigChangeReportResponse,
            >,
    >,
    delegate_throttle_approval_1_port: acme_platform::ports::Callback<
        notifications::CheckNotificationEligibilityRequest,
        notifications::CheckNotificationEligibilityResponse,
    >,
    delegate_throttle_approval_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<lending::BrowseLoanApplicationsRequest, lending::BrowseLoanApplicationsResponse>,
    >,
    recall_throttle_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<ratelimiting::ScoreThrottleRequest, ratelimiting::ScoreThrottleResponse>,
    >,
    recall_throttle_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<broadcasts::ResolveBroadcastsRequest, broadcasts::ResolveBroadcastsResponse>,
    >,
    reject_throttle_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::DeleteThrottleReportRequest,
            ratelimiting::DeleteThrottleReportResponse,
        >,
    >,
    reject_throttle_1_port:
        acme_platform::ports::Callback<ratelimiting::RenewThrottleRequest, ratelimiting::RenewThrottleResponse>,
}
impl ThrottleApprovalController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::ScheduleThrottleExpiryRequest,
            ratelimiting::ScheduleThrottleExpiryResponse,
        > = {
            let client = ThrottleLifecycleServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ScheduleThrottleExpiryRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ScheduleThrottleExpiryResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.schedule_throttle_expiry(request).await?.into_inner())
})
})
        };
        let approve_throttle_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::DeleteThrottleReportRequest,
            ratelimiting::DeleteThrottleReportResponse,
        > = {
            let client = ThrottleReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::DeleteThrottleReportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::DeleteThrottleReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.delete_throttle_report(request).await?.into_inner())
})
})
        };
        let reject_throttle_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::RenewThrottleRequest,
            ratelimiting::RenewThrottleResponse,
        > = {
            let client = ThrottleLifecycleServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::RenewThrottleRequest| -> acme_platform::ports::RpcFuture<ratelimiting::RenewThrottleResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.renew_throttle(request).await?.into_inner())
})
})
        };
        let reject_throttle_1_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::ShareConfigChangeReportRequest,
            configuration::ShareConfigChangeReportResponse,
        > = {
            let client = ConfigChangeReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ShareConfigChangeReportRequest| -> acme_platform::ports::RpcFuture<configuration::ShareConfigChangeReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_config_change_report(request).await?.into_inner())
})
})
        };
        let delegate_throttle_approval_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            notifications::CheckNotificationEligibilityRequest,
            notifications::CheckNotificationEligibilityResponse,
        > = {
            let client = NotificationValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: notifications::CheckNotificationEligibilityRequest| -> acme_platform::ports::RpcFuture<notifications::CheckNotificationEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_notification_eligibility(request).await?.into_inner())
})
})
        };
        let delegate_throttle_approval_1_port = callback;
        let callback: acme_platform::ports::Callback<
            lending::BrowseLoanApplicationsRequest,
            lending::BrowseLoanApplicationsResponse,
        > = {
            let client = LoanApplicationQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: lending::BrowseLoanApplicationsRequest| -> acme_platform::ports::RpcFuture<lending::BrowseLoanApplicationsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_loan_applications(request).await?.into_inner())
})
})
        };
        let delegate_throttle_approval_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::ScoreThrottleRequest,
            ratelimiting::ScoreThrottleResponse,
        > = {
            let client = ThrottleValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ScoreThrottleRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ScoreThrottleResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_throttle(request).await?.into_inner())
})
})
        };
        let recall_throttle_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            broadcasts::ResolveBroadcastsRequest,
            broadcasts::ResolveBroadcastsResponse,
        > = {
            let client = BroadcastQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: broadcasts::ResolveBroadcastsRequest| -> acme_platform::ports::RpcFuture<broadcasts::ResolveBroadcastsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_broadcasts(request).await?.into_inner())
})
})
        };
        let recall_throttle_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            approve_throttle_2_port,
            delegate_throttle_approval_0_port,
            delegate_throttle_approval_1_port,
            delegate_throttle_approval_2_port,
            recall_throttle_0_port,
            recall_throttle_1_port,
            reject_throttle_0_port,
            reject_throttle_1_port,
        }
    }
    /// Run SubmitThrottle after validating the tenant and durably recording the command.
    async fn run_submit_throttle(
        &self,
        req: ratelimiting::SubmitThrottleRequest,
    ) -> Result<ratelimiting::SubmitThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ResolveQuotaBucketConflictRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<ratelimiting::ResolveQuotaBucketConflictRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ResolveQuotaBucketConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_quota_bucket_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::SubmitThrottleResponse::default();
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
        let contribution: String = req.throttle.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ApproveThrottle after validating the tenant and durably recording the command.
    async fn run_approve_throttle(
        &self,
        req: ratelimiting::ApproveThrottleRequest,
    ) -> Result<ratelimiting::ApproveThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SyncEnvironmentsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(configuration::sync_environments_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            environment: Some(configuration::Environment {
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
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::SyncEnvironmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.sync_environments(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ListRateLimitConflictsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_rate_limit_conflicts_request::Filter {
                created_after: reply_0
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                region: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListRateLimitConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_rate_limit_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ScheduleThrottleExpiryRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::schedule_throttle_expiry_request::Details {
                notify: Some(common::ContactInfo {
                    email: reply_0
                        .environment
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
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: req
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
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.approve_throttle_2_port.as_ref(), outgoing_2).await?;
        let mut response = ratelimiting::ApproveThrottleResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
    /// Run RejectThrottle after validating the tenant and durably recording the command.
    async fn run_reject_throttle(
        &self,
        req: ratelimiting::RejectThrottleRequest,
    ) -> Result<ratelimiting::RejectThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().kind.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::DeleteThrottleReportRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            throttle_id: req.throttle_id.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.reject_throttle_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = ratelimiting::RenewThrottleRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::renew_throttle_request::Details {
                new_owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle_id: req.throttle.clone().unwrap_or_default().throttle_id.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.reject_throttle_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = ratelimiting::RejectThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
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
    /// Run EscalateThrottle after validating the tenant and durably recording the command.
    async fn run_escalate_throttle(
        &self,
        req: ratelimiting::EscalateThrottleRequest,
    ) -> Result<ratelimiting::EscalateThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().kind.clone(), "recorded audit field");
        let outgoing_0 = assets::ResolveFixedAssetsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<assets::ResolveFixedAssetsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FixedAssetQueryServiceClient::new(channel);
            let input: tonic::Request<assets::ResolveFixedAssetsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_fixed_assets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_1 = EscalateThrottleDispatch1::from(req.clone());
        let outgoing_1: ratelimiting::GetThrottleLifecycleRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleLifecycleServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::GetThrottleLifecycleRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_throttle_lifecycle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = authorization::BrowseAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::browse_access_policies_request::Filter {
                created_after: reply_1.throttle.clone().unwrap_or_default().occurred_at.clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
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
        let outgoing_2 =
            acme_platform::codec::through_any(&outgoing_2, "acme.authorization.v1.BrowseAccessPoliciesRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AccessPolicyQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::BrowseAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::StreamRateLimitMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .approver
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_3 = async move {
            let mut client = RateLimitReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamRateLimitMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
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
        let mut response = ratelimiting::EscalateThrottleResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: i64 = reply_1.throttle.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.throttle.get_or_insert_with(Default::default).sequence = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
    /// Run ListPendingThrottles after validating the tenant and durably recording the command.
    async fn run_list_pending_throttles(
        &self,
        req: ratelimiting::ListPendingThrottlesRequest,
    ) -> Result<ratelimiting::ListPendingThrottlesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListPendingThrottles", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingThrottles", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = secrets::SuggestSecretVersionsRequest {
            context: req.context.clone(),
            filter: Some(secrets::suggest_secret_versions_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_size: req.page.clone().unwrap_or_default().page_size.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretVersionQueryServiceClient::new(channel);
            let input: tonic::Request<secrets::SuggestSecretVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_secret_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ListPendingThrottlesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            if response.throttles.is_empty() {
                response.throttles.push(Default::default());
            }
            response.throttles[0].correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run DelegateThrottleApproval after validating the tenant and durably recording the command.
    async fn run_delegate_throttle_approval(
        &self,
        req: ratelimiting::DelegateThrottleApprovalRequest,
    ) -> Result<ratelimiting::DelegateThrottleApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateThrottleApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateThrottleApproval", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ShareConfigChangeReportRequest {
            context: req.context.clone(),
            details: Some(configuration::share_config_change_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req.details.clone().unwrap_or_default().approver_email.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.delegate_throttle_approval_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = notifications::CheckNotificationEligibilityRequest {
            context: req.context.clone(),
            details: Some(notifications::check_notification_eligibility_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            notification: Some(notifications::Notification {
                origin: Some(notifications::notification::Origin::Customer(
                    notifications::NotificationTemplateRef {
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
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = (self.delegate_throttle_approval_1_port)(outgoing_1).await?;
        let outgoing_2 = lending::BrowseLoanApplicationsRequest {
            context: req.context.clone(),
            filter: Some(lending::browse_loan_applications_request::Filter {
                created_after: reply_1
                    .notification
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
        };
        let reply_2 =
            self.delegate_throttle_approval_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let outgoing_3 = configuration::StreamConfigChangeMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .approver
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                name: reply_1
                    .notification
                    .clone()
                    .unwrap_or_default()
                    .requester
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                phone: reply_1
                    .notification
                    .clone()
                    .unwrap_or_default()
                    .requester
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: reply_1
                    .notification
                    .clone()
                    .unwrap_or_default()
                    .requester
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_3 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamConfigChangeMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_config_change_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::StreamConfigChangeMetricsResponse::default();
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
        let mut response = ratelimiting::DelegateThrottleApprovalResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
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
    /// Run RecallThrottle after validating the tenant and durably recording the command.
    async fn run_recall_throttle(
        &self,
        req: ratelimiting::RecallThrottleRequest,
    ) -> Result<ratelimiting::RecallThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ScoreThrottleRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
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
        let reply_0 = acme_platform::ports::execute(self.recall_throttle_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = broadcasts::ResolveBroadcastsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.recall_throttle_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = ratelimiting::PushThrottleRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::push_throttle_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                attributes: reply_0.throttle.clone().unwrap_or_default().attributes.clone(),
                kind: req.throttle.clone().unwrap_or_default().kind.clone(),
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: reply_0
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: reply_0
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
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushThrottleRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::RecallThrottleResponse::default();
        let contribution: String = reply_0.throttle.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).source = contribution;
        }
        let contribution: String = reply_0.throttle.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run GetThrottleApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_throttle_approval_status(
        &self,
        req: ratelimiting::GetThrottleApprovalStatusRequest,
    ) -> Result<ratelimiting::GetThrottleApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetThrottleApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetThrottleApprovalStatus", "running workflow");
        let outgoing_0 = ratelimiting::StreamRateLimitMetricsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::stream_rate_limit_metrics_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                owner_id: req.throttle_id.clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
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
        let outgoing_1 = ratelimiting::LintThrottlesRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::lint_throttles_request::Details {
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
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                throttle_id: req.throttle_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::LintThrottlesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_throttles(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::GetThrottleApprovalStatusResponse::default();
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
            response
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by EscalateThrottle for GetThrottleLifecycle.
struct EscalateThrottleDispatch1 {
    context: Option<common::RequestContext>,
    throttle_id: String,
}
impl From<ratelimiting::EscalateThrottleRequest> for EscalateThrottleDispatch1 {
    fn from(req: ratelimiting::EscalateThrottleRequest) -> Self {
        Self {
            context: req.context.clone(),
            throttle_id: req.throttle.clone().unwrap_or_default().throttle_id.clone(),
        }
    }
}
impl From<EscalateThrottleDispatch1> for ratelimiting::GetThrottleLifecycleRequest {
    fn from(value: EscalateThrottleDispatch1) -> Self {
        ratelimiting::GetThrottleLifecycleRequest {
            context: value.context,
            throttle_id: value.throttle_id,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ThrottleApprovalService for ThrottleApprovalController {
    /// Submits a throttle.
    async fn submit_throttle(
        &self,
        request: tonic::Request<ratelimiting::SubmitThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::SubmitThrottleResponse>, tonic::Status> {
        self.run_submit_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a throttle.
    async fn approve_throttle(
        &self,
        request: tonic::Request<ratelimiting::ApproveThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::ApproveThrottleResponse>, tonic::Status> {
        self.run_approve_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a throttle.
    async fn reject_throttle(
        &self,
        request: tonic::Request<ratelimiting::RejectThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::RejectThrottleResponse>, tonic::Status> {
        self.run_reject_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a throttle.
    async fn escalate_throttle(
        &self,
        request: tonic::Request<ratelimiting::EscalateThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::EscalateThrottleResponse>, tonic::Status> {
        self.run_escalate_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists throttles a page at a time.
    async fn list_pending_throttles(
        &self,
        request: tonic::Request<ratelimiting::ListPendingThrottlesRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListPendingThrottlesResponse>, tonic::Status> {
        self.run_list_pending_throttles(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates a throttle.
    async fn delegate_throttle_approval(
        &self,
        request: tonic::Request<ratelimiting::DelegateThrottleApprovalRequest>,
    ) -> Result<tonic::Response<ratelimiting::DelegateThrottleApprovalResponse>, tonic::Status> {
        self.run_delegate_throttle_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls a throttle.
    async fn recall_throttle(
        &self,
        request: tonic::Request<ratelimiting::RecallThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::RecallThrottleResponse>, tonic::Status> {
        self.run_recall_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a throttle.
    async fn get_throttle_approval_status(
        &self,
        request: tonic::Request<ratelimiting::GetThrottleApprovalStatusRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetThrottleApprovalStatusResponse>, tonic::Status> {
        self.run_get_throttle_approval_status(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
