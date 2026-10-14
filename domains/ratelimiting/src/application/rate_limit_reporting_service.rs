use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_validation_service_client::PermissionCheckValidationServiceClient;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::chat::v1::chat_room_query_service_client::ChatRoomQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_service_client::LedgerAccountServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_client::RateLimitQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_server::RateLimitReportingService;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;

/// Builds and schedules reports about rate limits.
#[derive(Clone)]
pub struct RateLimitReportingHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    fetch_rate_limit_report_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<featureflags::LintFlagRolloutsRequest, featureflags::LintFlagRolloutsResponse>,
    >,
    fetch_rate_limit_report_1_port: acme_platform::ports::Callback<
        authorization::ValidatePermissionCheckRequest,
        authorization::ValidatePermissionCheckResponse,
    >,
    generate_rate_limit_report_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::PullRateLimitsRequest, ratelimiting::PullRateLimitsResponse>,
    >,
}
impl RateLimitReportingHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
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
        let generate_rate_limit_report_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            featureflags::LintFlagRolloutsRequest,
            featureflags::LintFlagRolloutsResponse,
        > = {
            let client = FlagRolloutValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::LintFlagRolloutsRequest| -> acme_platform::ports::RpcFuture<featureflags::LintFlagRolloutsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_flag_rollouts(request).await?.into_inner())
})
})
        };
        let fetch_rate_limit_report_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            authorization::ValidatePermissionCheckRequest,
            authorization::ValidatePermissionCheckResponse,
        > = {
            let client = PermissionCheckValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::ValidatePermissionCheckRequest| -> acme_platform::ports::RpcFuture<authorization::ValidatePermissionCheckResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.validate_permission_check(request).await?.into_inner())
})
})
        };
        let fetch_rate_limit_report_1_port = callback;
        Self {
            channel,
            repository,
            fetch_rate_limit_report_0_port,
            fetch_rate_limit_report_1_port,
            generate_rate_limit_report_0_port,
        }
    }
    /// Run GenerateRateLimitReport after validating the tenant and durably recording the command.
    async fn run_generate_rate_limit_report(
        &self,
        req: ratelimiting::GenerateRateLimitReportRequest,
    ) -> Result<ratelimiting::GenerateRateLimitReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GenerateRateLimitReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateRateLimitReport", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PullRateLimitsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.generate_rate_limit_report_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = chat::CountChatRoomsRequest {
            context: req.context.clone(),
            filter: Some(chat::count_chat_rooms_request::Filter {
                created_after: reply_0
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: (reply_0.rate_limit.clone().unwrap_or_default().parameters.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: reply_0
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
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ChatRoomQueryServiceClient::new(channel);
            let input: tonic::Request<chat::CountChatRoomsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_chat_rooms(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::GenerateRateLimitReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run FetchRateLimitReport after validating the tenant and durably recording the command.
    async fn run_fetch_rate_limit_report(
        &self,
        req: ratelimiting::FetchRateLimitReportRequest,
    ) -> Result<ratelimiting::FetchRateLimitReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FetchRateLimitReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchRateLimitReport", "running workflow");
        let outgoing_0 = featureflags::LintFlagRolloutsRequest {
            context: req.context.clone(),
            details: Some(featureflags::lint_flag_rollouts_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            flag_rollout: Some(featureflags::FlagRollout {
                organizer: Some(identity::UserRef {
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
                participants: vec![featureflags::FlagRolloutParticipant {
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
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.fetch_rate_limit_report_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = authorization::ValidatePermissionCheckRequest {
            context: req.context.clone(),
            permission_check: Some(authorization::PermissionCheck {
                subject: Some(authorization::AccessPolicyRef {
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
        let reply_1 = (self.fetch_rate_limit_report_1_port)(outgoing_1).await?;
        let mut response = ratelimiting::FetchRateLimitReportResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .organizer
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        Ok(response)
    }
    /// Run StreamRateLimitMetrics after validating the tenant and durably recording the command.
    async fn run_stream_rate_limit_metrics(
        &self,
        req: ratelimiting::StreamRateLimitMetricsRequest,
    ) -> Result<ratelimiting::StreamRateLimitMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StreamRateLimitMetrics", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamRateLimitMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PullQuotaBucketsRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    phone: req.subscriber.clone().unwrap_or_default().phone.clone(),
                    preferred_channel: req.subscriber.clone().unwrap_or_default().preferred_channel.clone(),
                    ..Default::default()
                }),
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PullQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pull_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ledger::CreateLedgerAccountRequest {
            context: req.context.clone(),
            ledger_account: Some(ledger::LedgerAccount {
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
                funding: Some(ledger::ledger_account::Funding::Card(
                    ledger::LedgerAccountCardFunding {
                        token: (match reply_0.quota_bucket.clone().unwrap_or_default().funding.clone() {
                            Some(ratelimiting::quota_bucket::Funding::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .token
                        .clone(),
                        ..Default::default()
                    },
                )),
                contact: Some(common::ContactInfo {
                    email: reply_0
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = LedgerAccountServiceClient::new(channel);
            let input: tonic::Request<ledger::CreateLedgerAccountRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_ledger_account(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::StreamRateLimitMetricsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListRateLimitReports after validating the tenant and durably recording the command.
    async fn run_list_rate_limit_reports(
        &self,
        req: ratelimiting::ListRateLimitReportsRequest,
    ) -> Result<ratelimiting::ListRateLimitReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListRateLimitReports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRateLimitReports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = ratelimiting::ListRateLimitReportsResponse::default();
        Ok(response)
    }
    /// Run ShareRateLimitReport after validating the tenant and durably recording the command.
    async fn run_share_rate_limit_report(
        &self,
        req: ratelimiting::ShareRateLimitReportRequest,
    ) -> Result<ratelimiting::ShareRateLimitReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareRateLimitReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareRateLimitReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = featureflags::LintFlagEvaluationsRequest {
            context: req.context.clone(),
            details: Some(featureflags::lint_flag_evaluations_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
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
        let outgoing_0 = acme_platform::ports::RequestBuilder::<featureflags::LintFlagEvaluationsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::LintFlagEvaluationsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_flag_evaluations(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = ShareRateLimitReportDispatch1::from(req.clone());
        command_1.filter_created_after = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .starts_at
            .clone();
        let outgoing_1: featureflags::ListFlagRolloutConflictsRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ListFlagRolloutConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_flag_rollout_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ReconcileRateLimitsRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 =
            acme_platform::codec::through_any(&outgoing_2, "acme.ratelimiting.v1.ReconcileRateLimitsRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ReconcileRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::ApproveConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::approve_config_change_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_0
                        .flag_evaluation
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: reply_2
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::ApproveConfigChangeRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigChangeApprovalServiceClient::new(channel);
                let input: tonic::Request<configuration::ApproveConfigChangeRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.approve_config_change(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_3)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_3 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = ratelimiting::ShareRateLimitReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2
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
        let contribution: Vec<String> = reply_3.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
    /// Run CountRateLimitTotals after validating the tenant and durably recording the command.
    async fn run_count_rate_limit_totals(
        &self,
        req: ratelimiting::CountRateLimitTotalsRequest,
    ) -> Result<ratelimiting::CountRateLimitTotalsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountRateLimitTotals", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountRateLimitTotals", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ExplainThrottleDecisionRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_throttle_decision_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::ResolveRateLimitsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ThrottleValidationServiceClient::new(channel_0);
                let input: tonic::Request<ratelimiting::ExplainThrottleDecisionRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_throttle_decision(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = RateLimitQueryServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::ResolveRateLimitsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_rate_limits(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::CountRateLimitTotalsResponse::default();
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
        Ok(response)
    }
}
/// The domain command prepared by ShareRateLimitReport for ListFlagRolloutConflicts.
struct ShareRateLimitReportDispatch1 {
    context: Option<common::RequestContext>,
    filter_created_after: Option<prost_types::Timestamp>,
}
impl From<ratelimiting::ShareRateLimitReportRequest> for ShareRateLimitReportDispatch1 {
    fn from(req: ratelimiting::ShareRateLimitReportRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_created_after: Default::default(),
        }
    }
}
impl From<ShareRateLimitReportDispatch1> for featureflags::ListFlagRolloutConflictsRequest {
    fn from(value: ShareRateLimitReportDispatch1) -> Self {
        featureflags::ListFlagRolloutConflictsRequest {
            context: value.context,
            filter: Some(featureflags::list_flag_rollout_conflicts_request::Filter {
                created_after: value.filter_created_after,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl RateLimitReportingService for RateLimitReportingHandler {
    /// Generates a rate limit.
    async fn generate_rate_limit_report(
        &self,
        request: tonic::Request<ratelimiting::GenerateRateLimitReportRequest>,
    ) -> Result<tonic::Response<ratelimiting::GenerateRateLimitReportResponse>, tonic::Status> {
        self.run_generate_rate_limit_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a rate limit.
    async fn fetch_rate_limit_report(
        &self,
        request: tonic::Request<ratelimiting::FetchRateLimitReportRequest>,
    ) -> Result<tonic::Response<ratelimiting::FetchRateLimitReportResponse>, tonic::Status> {
        self.run_fetch_rate_limit_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamRateLimitMetricsStream =
        tokio_stream::wrappers::ReceiverStream<Result<ratelimiting::StreamRateLimitMetricsResponse, tonic::Status>>;
    /// Streams the changes of rate limits as they happen.
    async fn stream_rate_limit_metrics(
        &self,
        request: tonic::Request<ratelimiting::StreamRateLimitMetricsRequest>,
    ) -> Result<tonic::Response<Self::StreamRateLimitMetricsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_rate_limit_metrics(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Lists rate limits a page at a time.
    async fn list_rate_limit_reports(
        &self,
        request: tonic::Request<ratelimiting::ListRateLimitReportsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListRateLimitReportsResponse>, tonic::Status> {
        self.run_list_rate_limit_reports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Shares a rate limit.
    async fn share_rate_limit_report(
        &self,
        request: tonic::Request<ratelimiting::ShareRateLimitReportRequest>,
    ) -> Result<tonic::Response<ratelimiting::ShareRateLimitReportResponse>, tonic::Status> {
        self.run_share_rate_limit_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts rate limits by status.
    async fn count_rate_limit_totals(
        &self,
        request: tonic::Request<ratelimiting::CountRateLimitTotalsRequest>,
    ) -> Result<tonic::Response<ratelimiting::CountRateLimitTotalsResponse>, tonic::Status> {
        self.run_count_rate_limit_totals(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
