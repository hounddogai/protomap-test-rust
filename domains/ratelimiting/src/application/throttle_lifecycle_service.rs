use acme_proto::acme::accounting::v1 as accounting;
use acme_proto::acme::accounting::v1::chart_of_accounts_service_client::ChartOfAccountsServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_approval_service_client::ConfigSetApprovalServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_client::ThrottleImportServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_lifecycle_service_server::ThrottleLifecycleService;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;

/// Moves a throttle between the states of its lifecycle.
#[derive(Clone)]
pub struct ThrottleLifecycleHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    get_throttle_lifecycle_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<ratelimiting::CloneRateLimitRequest, ratelimiting::CloneRateLimitResponse>,
    >,
    get_throttle_lifecycle_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::GetThrottleSyncStateRequest,
                ratelimiting::GetThrottleSyncStateResponse,
            >,
    >,
    renew_throttle_0_port: acme_platform::ports::Callback<
        ratelimiting::ListRateLimitReportsRequest,
        ratelimiting::ListRateLimitReportsResponse,
    >,
    renew_throttle_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::GetThrottleImportStatusRequest,
            ratelimiting::GetThrottleImportStatusResponse,
        >,
    >,
    schedule_throttle_expiry_0_port: acme_platform::ports::Callback<
        certificates::CompareCertificateAuthorityOptionsRequest,
        certificates::CompareCertificateAuthorityOptionsResponse,
    >,
}
impl ThrottleLifecycleHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
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
        let renew_throttle_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::GetThrottleImportStatusRequest,
            ratelimiting::GetThrottleImportStatusResponse,
        > = {
            let client = ThrottleImportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::GetThrottleImportStatusRequest| -> acme_platform::ports::RpcFuture<ratelimiting::GetThrottleImportStatusResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.get_throttle_import_status(request).await?.into_inner())
})
})
        };
        let renew_throttle_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::CloneRateLimitRequest,
            ratelimiting::CloneRateLimitResponse,
        > = {
            let client = RateLimitServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CloneRateLimitRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CloneRateLimitResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_rate_limit(request).await?.into_inner())
})
})
        };
        let get_throttle_lifecycle_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::GetThrottleSyncStateRequest,
            ratelimiting::GetThrottleSyncStateResponse,
        > = {
            let client = ThrottleSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::GetThrottleSyncStateRequest| -> acme_platform::ports::RpcFuture<ratelimiting::GetThrottleSyncStateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.get_throttle_sync_state(request).await?.into_inner())
})
})
        };
        let get_throttle_lifecycle_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::CompareCertificateAuthorityOptionsRequest,
            certificates::CompareCertificateAuthorityOptionsResponse,
        > = {
            let client = CertificateAuthorityCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CompareCertificateAuthorityOptionsRequest| -> acme_platform::ports::RpcFuture<certificates::CompareCertificateAuthorityOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_certificate_authority_options(request).await?.into_inner())
})
})
        };
        let schedule_throttle_expiry_0_port = callback;
        Self {
            channel,
            repository,
            get_throttle_lifecycle_0_port,
            get_throttle_lifecycle_1_port,
            renew_throttle_0_port,
            renew_throttle_1_port,
            schedule_throttle_expiry_0_port,
        }
    }
    /// Run ActivateThrottle after validating the tenant and durably recording the command.
    async fn run_activate_throttle(
        &self,
        req: ratelimiting::ActivateThrottleRequest,
    ) -> Result<ratelimiting::ActivateThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ActivateThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ActivateThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .name
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
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let reply_0 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::QuoteQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ListPendingConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_pending_config_sets_request::Filter {
                created_after: reply_0.quota_bucket.clone().unwrap_or_default().opened_at.clone(),
                labels: reply_0.quota_bucket.clone().unwrap_or_default().labels.clone(),
                owner_id: (match reply_0.quota_bucket.clone().unwrap_or_default().funding.clone() {
                    Some(ratelimiting::quota_bucket::Funding::ExternalAccountId(value)) => value,
                    _ => Default::default(),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::ListPendingConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ActivateThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
    /// Run SuspendThrottle after validating the tenant and durably recording the command.
    async fn run_suspend_throttle(
        &self,
        req: ratelimiting::SuspendThrottleRequest,
    ) -> Result<ratelimiting::SuspendThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuspendThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuspendThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let command_0 = SuspendThrottleDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::ListQuotaBucketReportsRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListQuotaBucketReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_quota_bucket_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::SyncRateLimitsRequest {
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
        let outgoing_1 = acme_platform::codec::through_any(&outgoing_1, "acme.ratelimiting.v1.SyncRateLimitsRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SyncRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.sync_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::SuspendThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .rate_limit
            .clone()
            .unwrap_or_default()
            .owner
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
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TerminateThrottle after validating the tenant and durably recording the command.
    async fn run_terminate_throttle(
        &self,
        req: ratelimiting::TerminateThrottleRequest,
    ) -> Result<ratelimiting::TerminateThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TerminateThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TerminateThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = accounting::WatchChartOfAccountsesRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .new_owner
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
            let mut client = ChartOfAccountsServiceClient::new(channel);
            let input: tonic::Request<accounting::WatchChartOfAccountsesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_chart_of_accountses(input).await?;
            let mut frames = response.into_inner();
            let mut last = accounting::WatchChartOfAccountsesResponse::default();
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
        let outgoing_1 = ratelimiting::ShareRateLimitReportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::share_rate_limit_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = RateLimitReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ShareRateLimitReportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_rate_limit_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = ratelimiting::TerminateThrottleResponse::default();
        let contribution: i64 = reply_1.rate_limit.clone().unwrap_or_default().version.clone();
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
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
    /// Run RenewThrottle after validating the tenant and durably recording the command.
    async fn run_renew_throttle(
        &self,
        req: ratelimiting::RenewThrottleRequest,
    ) -> Result<ratelimiting::RenewThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RenewThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RenewThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ListRateLimitReportsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_rate_limit_reports_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.renew_throttle_0_port)(outgoing_0).await?;
        let outgoing_1 = ratelimiting::GetThrottleImportStatusRequest {
            context: req.context.clone(),
            throttle_id: req.throttle_id.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.renew_throttle_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = ratelimiting::RenewThrottleResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.throttle.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).kind = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run GetThrottleLifecycle after validating the tenant and durably recording the command.
    async fn run_get_throttle_lifecycle(
        &self,
        req: ratelimiting::GetThrottleLifecycleRequest,
    ) -> Result<ratelimiting::GetThrottleLifecycleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetThrottleLifecycle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetThrottleLifecycle", "running workflow");
        let outgoing_0 = ratelimiting::CloneRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
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
        let reply_0 = acme_platform::ports::execute(self.get_throttle_lifecycle_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = ratelimiting::GetThrottleSyncStateRequest {
            context: req.context.clone(),
            throttle_id: req.throttle_id.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.get_throttle_lifecycle_1_port.as_ref(), outgoing_1).await?;
        let mut response = ratelimiting::GetThrottleLifecycleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i64 = reply_1.throttle.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.throttle.get_or_insert_with(Default::default).sequence = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.throttle.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.throttle.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleThrottleExpiry after validating the tenant and durably recording the command.
    async fn run_schedule_throttle_expiry(
        &self,
        req: ratelimiting::ScheduleThrottleExpiryRequest,
    ) -> Result<ratelimiting::ScheduleThrottleExpiryResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScheduleThrottleExpiry", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleThrottleExpiry", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CompareCertificateAuthorityOptionsRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.schedule_throttle_expiry_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = featureflags::ExplainFlagEvaluationDecisionRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_flag_evaluation_decision_request::Details {
                contact: Some(common::ContactInfo {
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
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<featureflags::ExplainFlagEvaluationDecisionRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagEvaluationValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExplainFlagEvaluationDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_flag_evaluation_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ScheduleThrottleExpiryResponse::default();
        let contribution: String = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .manager
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
        let contribution: std::collections::BTreeMap<String, String> = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .attributes
            .clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
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
}
/// The domain command prepared by SuspendThrottle for ListQuotaBucketReports.
struct SuspendThrottleDispatch0 {
    context: Option<common::RequestContext>,
    filter_owner_id: String,
    filter_labels: std::collections::BTreeMap<String, String>,
    filter_created_after: Option<prost_types::Timestamp>,
}
impl From<ratelimiting::SuspendThrottleRequest> for SuspendThrottleDispatch0 {
    fn from(req: ratelimiting::SuspendThrottleRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_owner_id: req
                .details
                .clone()
                .unwrap_or_default()
                .new_owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            filter_labels: req.annotations.clone(),
            filter_created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
        }
    }
}
impl From<SuspendThrottleDispatch0> for ratelimiting::ListQuotaBucketReportsRequest {
    fn from(value: SuspendThrottleDispatch0) -> Self {
        ratelimiting::ListQuotaBucketReportsRequest {
            context: value.context,
            filter: Some(ratelimiting::list_quota_bucket_reports_request::Filter {
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
impl ThrottleLifecycleService for ThrottleLifecycleHandler {
    /// Activates a throttle.
    async fn activate_throttle(
        &self,
        request: tonic::Request<ratelimiting::ActivateThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::ActivateThrottleResponse>, tonic::Status> {
        self.run_activate_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Suspends a throttle.
    async fn suspend_throttle(
        &self,
        request: tonic::Request<ratelimiting::SuspendThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::SuspendThrottleResponse>, tonic::Status> {
        self.run_suspend_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Terminates a throttle.
    async fn terminate_throttle(
        &self,
        request: tonic::Request<ratelimiting::TerminateThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::TerminateThrottleResponse>, tonic::Status> {
        self.run_terminate_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Renews a throttle.
    async fn renew_throttle(
        &self,
        request: tonic::Request<ratelimiting::RenewThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::RenewThrottleResponse>, tonic::Status> {
        self.run_renew_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a throttle.
    async fn get_throttle_lifecycle(
        &self,
        request: tonic::Request<ratelimiting::GetThrottleLifecycleRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetThrottleLifecycleResponse>, tonic::Status> {
        self.run_get_throttle_lifecycle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a throttle.
    async fn schedule_throttle_expiry(
        &self,
        request: tonic::Request<ratelimiting::ScheduleThrottleExpiryRequest>,
    ) -> Result<tonic::Response<ratelimiting::ScheduleThrottleExpiryResponse>, tonic::Status> {
        self.run_schedule_throttle_expiry(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
