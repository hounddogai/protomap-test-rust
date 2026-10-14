use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::configuration::v1::config_set_approval_service_client::ConfigSetApprovalServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_service_client::UserServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;
use acme_proto::acme::notifications::v1::notification_validation_service_client::NotificationValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_approval_service_server::RateLimitApprovalService;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_client::RateLimitQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_client::ThrottleImportServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Routes rate limits through approval, with escalation and delegation.
#[derive(Clone)]
pub struct RateLimitApprovalHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    approve_rate_limit_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<identity::ListUsersRequest, identity::ListUsersResponse>,
    >,
    get_rate_limit_approval_status_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::ReconcileThrottlesRequest,
                ratelimiting::ReconcileThrottlesResponse,
            >,
    >,
    list_pending_rate_limits_0_port: acme_platform::ports::Callback<
        notifications::LintNotificationsRequest,
        notifications::LintNotificationsResponse,
    >,
    list_pending_rate_limits_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::SignalConfigChangeWorkflowRequest,
            configuration::SignalConfigChangeWorkflowResponse,
        >,
    >,
    recall_rate_limit_0_port: acme_platform::ports::Callback<venues::CloneVenueRequest, venues::CloneVenueResponse>,
    submit_rate_limit_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ResumeConfigChangeWorkflowRequest,
            configuration::ResumeConfigChangeWorkflowResponse,
        >,
    >,
    submit_rate_limit_3_port:
        acme_platform::ports::Callback<ratelimiting::PeekRateLimitRequest, ratelimiting::PeekRateLimitResponse>,
}
impl RateLimitApprovalHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::ResumeConfigChangeWorkflowRequest,
            configuration::ResumeConfigChangeWorkflowResponse,
        > = {
            let client = ConfigChangeWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ResumeConfigChangeWorkflowRequest| -> acme_platform::ports::RpcFuture<configuration::ResumeConfigChangeWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resume_config_change_workflow(request).await?.into_inner())
})
})
        };
        let submit_rate_limit_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::PeekRateLimitRequest,
            ratelimiting::PeekRateLimitResponse,
        > = {
            let client = RateLimitQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::PeekRateLimitRequest| -> acme_platform::ports::RpcFuture<ratelimiting::PeekRateLimitResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.peek_rate_limit(request).await?.into_inner())
})
})
        };
        let submit_rate_limit_3_port = callback;
        let callback: acme_platform::ports::Callback<identity::ListUsersRequest, identity::ListUsersResponse> = {
            let client = UserServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: identity::ListUsersRequest| -> acme_platform::ports::RpcFuture<identity::ListUsersResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_users(request).await?.into_inner())
})
})
        };
        let approve_rate_limit_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            notifications::LintNotificationsRequest,
            notifications::LintNotificationsResponse,
        > = {
            let client = NotificationValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: notifications::LintNotificationsRequest| -> acme_platform::ports::RpcFuture<notifications::LintNotificationsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_notifications(request).await?.into_inner())
})
})
        };
        let list_pending_rate_limits_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::SignalConfigChangeWorkflowRequest,
            configuration::SignalConfigChangeWorkflowResponse,
        > = {
            let client = ConfigChangeWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SignalConfigChangeWorkflowRequest| -> acme_platform::ports::RpcFuture<configuration::SignalConfigChangeWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.signal_config_change_workflow(request).await?.into_inner())
})
})
        };
        let list_pending_rate_limits_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<venues::CloneVenueRequest, venues::CloneVenueResponse> = {
            let client = VenueServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: venues::CloneVenueRequest| -> acme_platform::ports::RpcFuture<venues::CloneVenueResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_venue(request).await?.into_inner())
})
})
        };
        let recall_rate_limit_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::ReconcileThrottlesRequest,
            ratelimiting::ReconcileThrottlesResponse,
        > = {
            let client = ThrottleSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ReconcileThrottlesRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ReconcileThrottlesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.reconcile_throttles(request).await?.into_inner())
})
})
        };
        let get_rate_limit_approval_status_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            approve_rate_limit_1_port,
            get_rate_limit_approval_status_0_port,
            list_pending_rate_limits_0_port,
            list_pending_rate_limits_1_port,
            recall_rate_limit_0_port,
            submit_rate_limit_1_port,
            submit_rate_limit_3_port,
        }
    }
    /// Run SubmitRateLimit after validating the tenant and durably recording the command.
    async fn run_submit_rate_limit(
        &self,
        req: ratelimiting::SubmitRateLimitRequest,
    ) -> Result<ratelimiting::SubmitRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = notifications::CloneNotificationRequest {
            context: req.context.clone(),
            notification: Some(notifications::Notification {
                assignee: Some(identity::UserRef {
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = NotificationServiceClient::new(channel);
            let input: tonic::Request<notifications::CloneNotificationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_notification(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ResumeConfigChangeWorkflowRequest {
            context: req.context.clone(),
            details: Some(configuration::resume_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    preferred_channel: reply_0
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
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.submit_rate_limit_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::ScheduleConfigChangeExpiryRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    config_set_id: reply_1
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .config_set_id
                        .clone(),
                    display_name: (match reply_0.notification.clone().unwrap_or_default().origin.clone() {
                        Some(notifications::notification::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .display_name
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::schedule_config_change_expiry_request::Details {
                notify: Some(common::ContactInfo {
                    name: reply_0
                        .notification
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: reply_0
                        .notification
                        .clone()
                        .unwrap_or_default()
                        .requester
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigChangeLifecycleServiceClient::new(channel);
            let input: tonic::Request<configuration::ScheduleConfigChangeExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_config_change_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::PeekRateLimitRequest {
            context: req.context.clone(),
            rate_limit_id: req.rate_limit_id.clone(),
            ..Default::default()
        };
        let reply_3 = (self.submit_rate_limit_3_port)(outgoing_3).await?;
        let mut response = ratelimiting::SubmitRateLimitResponse::default();
        let contribution: String = reply_0
            .notification
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i32 = reply_3.rate_limit.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).priority = contribution;
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
    /// Run ApproveRateLimit after validating the tenant and durably recording the command.
    async fn run_approve_rate_limit(
        &self,
        req: ratelimiting::ApproveRateLimitRequest,
    ) -> Result<ratelimiting::ApproveRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveRateLimit", "running workflow");
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
        let outgoing_1 = identity::ListUsersRequest {
            context: req.context.clone(),
            filter: Some(identity::list_users_request::Filter {
                created_after: reply_0
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.rate_limit.clone().unwrap_or_default().parameters.clone(),
                owner_id: req
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
        };
        let reply_1 =
            self.approve_rate_limit_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = ratelimiting::GenerateQuotaBucketReportRequest {
            context: req.context.clone(),
            reason: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::GenerateQuotaBucketReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_quota_bucket_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = identity::SearchUsersRequest {
            context: req.context.clone(),
            filter: Some(identity::search_users_request::Filter {
                created_after: reply_0
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_2.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0.outcome.clone().unwrap_or_default().reference_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_3 = acme_platform::ports::RequestBuilder::<identity::SearchUsersRequest>::default()
            .map(|draft| *draft = outgoing_3)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::SearchUsersRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_users(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ApproveRateLimitResponse::default();
        let contribution: i32 = reply_0.rate_limit.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
    /// Run RejectRateLimit after validating the tenant and durably recording the command.
    async fn run_reject_rate_limit(
        &self,
        req: ratelimiting::RejectRateLimitRequest,
    ) -> Result<ratelimiting::RejectRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = notifications::CloneNotificationRequest {
            context: req.context.clone(),
            notification: Some(notifications::Notification {
                assignee: Some(identity::UserRef {
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
                origin: Some(notifications::notification::Origin::Customer(
                    notifications::NotificationTemplateRef {
                        display_name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .approver
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::GenerateThrottleReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(ratelimiting::generate_throttle_report_request::Details {
                filters: req.rate_limit.clone().unwrap_or_default().parameters.clone(),
                period: req.rate_limit.clone().unwrap_or_default().effective.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = NotificationServiceClient::new(channel_0);
                let input: tonic::Request<notifications::CloneNotificationRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.clone_notification(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ThrottleReportingServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::GenerateThrottleReportRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.generate_throttle_report(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = identity::CloneUserRequest {
            context: req.context.clone(),
            user: Some(identity::User {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
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
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .notification
                        .clone()
                        .unwrap_or_default()
                        .assignee
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                relationship_manager: Some(identity::UserRef {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::CloneUserRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_user(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_3 = identity::BatchGetUsersRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::BatchGetUsersRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_users(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::RejectRateLimitResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2
            .user
            .clone()
            .unwrap_or_default()
            .relationship_manager
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run EscalateRateLimit after validating the tenant and durably recording the command.
    async fn run_escalate_rate_limit(
        &self,
        req: ratelimiting::EscalateRateLimitRequest,
    ) -> Result<ratelimiting::EscalateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ValidateFlagRolloutRequest {
            context: req.context.clone(),
            details: Some(featureflags::validate_flag_rollout_request::Details {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = FlagRolloutValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ValidateFlagRolloutRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_flag_rollout(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = ratelimiting::ResolveQuotaBucketConflictRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::resolve_quota_bucket_conflict_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
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
                    display_name: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ResolveQuotaBucketConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_quota_bucket_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = identity::CloneUserRequest {
            context: req.context.clone(),
            user: Some(identity::User {
                contact: Some(common::ContactInfo {
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
                primary_address: Some(common::PostalAddress {
                    lines: (reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone())
                    .into_iter()
                    .filter_map(|value| {
                        let value = value.trim();
                        if value.is_empty() { None } else { Some(value.to_owned()) }
                    })
                    .collect(),
                    ..Default::default()
                }),
                relationship_manager: Some(identity::UserRef {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::CloneUserRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_user(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::ResolveFeatureFlagConflictRequest {
            context: req.context.clone(),
            details: Some(featureflags::resolve_feature_flag_conflict_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    etag: reply_2
                        .user
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
                    email: reply_1
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ResolveFeatureFlagConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_feature_flag_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::EscalateRateLimitResponse::default();
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_2
            .user
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
        let contribution: String = reply_2
            .user
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_3.feature_flag.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.rate_limit.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingRateLimits after validating the tenant and durably recording the command.
    async fn run_list_pending_rate_limits(
        &self,
        req: ratelimiting::ListPendingRateLimitsRequest,
    ) -> Result<ratelimiting::ListPendingRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListPendingRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = notifications::LintNotificationsRequest {
            context: req.context.clone(),
            notification: Some(notifications::Notification {
                assignee: Some(identity::UserRef {
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
                requester: Some(common::ContactInfo {
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
        let reply_0 = (self.list_pending_rate_limits_0_port)(outgoing_0).await?;
        let outgoing_1 = configuration::SignalConfigChangeWorkflowRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: (match reply_0.notification.clone().unwrap_or_default().origin.clone() {
                        Some(notifications::notification::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .display_name
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::signal_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: reply_0
                        .notification
                        .clone()
                        .unwrap_or_default()
                        .assignee
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: reply_0
                        .notification
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            self.list_pending_rate_limits_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = ratelimiting::ListPendingRateLimitsResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .notification
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
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run DelegateRateLimitApproval after validating the tenant and durably recording the command.
    async fn run_delegate_rate_limit_approval(
        &self,
        req: ratelimiting::DelegateRateLimitApprovalRequest,
    ) -> Result<ratelimiting::DelegateRateLimitApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateRateLimitApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateRateLimitApproval", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SubmitConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
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
                        manager: Some(identity::UserRef {
                            display_name: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .approver
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
                            .approver
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::SubmitConfigSetRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigSetApprovalServiceClient::new(channel);
                let input: tonic::Request<configuration::SubmitConfigSetRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.submit_config_set(input).await?;
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
        let mut response = ratelimiting::DelegateRateLimitApprovalResponse::default();
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.rate_limit.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run RecallRateLimit after validating the tenant and durably recording the command.
    async fn run_recall_rate_limit(
        &self,
        req: ratelimiting::RecallRateLimitRequest,
    ) -> Result<ratelimiting::RecallRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = venues::CloneVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
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
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
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
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.recall_rate_limit_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = ratelimiting::RetryThrottleImportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::retry_throttle_import_request::Details {
                notify: Some(common::ContactInfo {
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
            throttle: Some(ratelimiting::Throttle {
                measurements: vec![ratelimiting::ThrottleMeasurement {
                    unit: reply_0
                        .venue
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .unit
                        .clone(),
                    ..Default::default()
                }],
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: req.rate_limit.clone().unwrap_or_default().version.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<ratelimiting::RetryThrottleImportRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleImportServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RetryThrottleImportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.retry_throttle_import(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = RecallRateLimitDispatch2::from(req.clone());
        command_2.config_change_subject_display_name = reply_1
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        let outgoing_2: configuration::AdvanceConfigChangeWorkflowRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::AdvanceConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::RecallRateLimitResponse::default();
        let contribution: String = reply_0
            .venue
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.rate_limit.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run GetRateLimitApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_rate_limit_approval_status(
        &self,
        req: ratelimiting::GetRateLimitApprovalStatusRequest,
    ) -> Result<ratelimiting::GetRateLimitApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetRateLimitApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRateLimitApprovalStatus", "running workflow");
        let outgoing_0 = ratelimiting::ReconcileThrottlesRequest {
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                correlation_id: req
                    .context
                    .clone()
                    .unwrap_or_default()
                    .actor
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                occurred_at: req.context.clone().unwrap_or_default().deadline.clone(),
                subject: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: req.rate_limit_id.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.get_rate_limit_approval_status_0_port.as_ref(), outgoing_0).await?;
        let mut response = ratelimiting::GetRateLimitApprovalStatusResponse::default();
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
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
    /// Run CommentRateLimitApproval after validating the tenant and durably recording the command.
    async fn run_comment_rate_limit_approval(
        &self,
        req: ratelimiting::CommentRateLimitApprovalRequest,
    ) -> Result<ratelimiting::CommentRateLimitApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CommentRateLimitApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CommentRateLimitApproval", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let command_0 = CommentRateLimitApprovalDispatch0::from(req.clone());
        let outgoing_0: configuration::CommentConfigSetApprovalRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::CommentConfigSetApprovalRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.comment_config_set_approval(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::SyncQuotaBucketsRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
                    etag: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_at: reply_0
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.ratelimiting.v1.SyncQuotaBucketsRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SyncQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.sync_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CommentRateLimitApprovalResponse::default();
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.rate_limit.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by RecallRateLimit for AdvanceConfigChangeWorkflow.
struct RecallRateLimitDispatch2 {
    context: Option<common::RequestContext>,
    details_notify_email: String,
    config_change_subject_display_name: String,
}
impl From<ratelimiting::RecallRateLimitRequest> for RecallRateLimitDispatch2 {
    fn from(req: ratelimiting::RecallRateLimitRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_notify_email: req
                .details
                .clone()
                .unwrap_or_default()
                .approver
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            config_change_subject_display_name: Default::default(),
        }
    }
}
impl From<RecallRateLimitDispatch2> for configuration::AdvanceConfigChangeWorkflowRequest {
    fn from(value: RecallRateLimitDispatch2) -> Self {
        configuration::AdvanceConfigChangeWorkflowRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: value.config_change_subject_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            details: Some(configuration::advance_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: value.details_notify_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by CommentRateLimitApproval for CommentConfigSetApproval.
struct CommentRateLimitApprovalDispatch0 {
    context: Option<common::RequestContext>,
    config_set_children_children_manager_email: String,
    config_set_children_audit_created_at: Option<prost_types::Timestamp>,
    config_set_children_children_audit_created_by: String,
}
impl From<ratelimiting::CommentRateLimitApprovalRequest> for CommentRateLimitApprovalDispatch0 {
    fn from(req: ratelimiting::CommentRateLimitApprovalRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_children_manager_email: req
                .rate_limit
                .clone()
                .unwrap_or_default()
                .owner
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            config_set_children_audit_created_at: req
                .rate_limit
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
            config_set_children_children_audit_created_by: req
                .rate_limit
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_by
                .clone(),
        }
    }
}
impl From<CommentRateLimitApprovalDispatch0> for configuration::CommentConfigSetApprovalRequest {
    fn from(value: CommentRateLimitApprovalDispatch0) -> Self {
        configuration::CommentConfigSetApprovalRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        created_at: value.config_set_children_audit_created_at,
                        ..Default::default()
                    }),
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_by: value.config_set_children_children_audit_created_by,
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            email: value.config_set_children_children_manager_email,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl RateLimitApprovalService for RateLimitApprovalHandler {
    /// Submits a rate limit.
    async fn submit_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::SubmitRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::SubmitRateLimitResponse>, tonic::Status> {
        self.run_submit_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a rate limit.
    async fn approve_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::ApproveRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::ApproveRateLimitResponse>, tonic::Status> {
        self.run_approve_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a rate limit.
    async fn reject_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::RejectRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::RejectRateLimitResponse>, tonic::Status> {
        self.run_reject_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a rate limit.
    async fn escalate_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::EscalateRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::EscalateRateLimitResponse>, tonic::Status> {
        self.run_escalate_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists rate limits a page at a time.
    async fn list_pending_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::ListPendingRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListPendingRateLimitsResponse>, tonic::Status> {
        self.run_list_pending_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates a rate limit.
    async fn delegate_rate_limit_approval(
        &self,
        request: tonic::Request<ratelimiting::DelegateRateLimitApprovalRequest>,
    ) -> Result<tonic::Response<ratelimiting::DelegateRateLimitApprovalResponse>, tonic::Status> {
        self.run_delegate_rate_limit_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls a rate limit.
    async fn recall_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::RecallRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::RecallRateLimitResponse>, tonic::Status> {
        self.run_recall_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a rate limit.
    async fn get_rate_limit_approval_status(
        &self,
        request: tonic::Request<ratelimiting::GetRateLimitApprovalStatusRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetRateLimitApprovalStatusResponse>, tonic::Status> {
        self.run_get_rate_limit_approval_status(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Comments a rate limit.
    async fn comment_rate_limit_approval(
        &self,
        request: tonic::Request<ratelimiting::CommentRateLimitApprovalRequest>,
    ) -> Result<tonic::Response<ratelimiting::CommentRateLimitApprovalResponse>, tonic::Status> {
        self.run_comment_rate_limit_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
