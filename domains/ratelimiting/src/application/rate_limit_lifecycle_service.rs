use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::appointments::v1::appointment_query_service_client::AppointmentQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_query_service_client::RoleBindingQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_lifecycle_service_server::RateLimitLifecycleService;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_query_service_client::SearchIndexQueryServiceClient;

/// Moves a rate limit between the states of its lifecycle.
#[derive(Clone)]
pub struct RateLimitLifecycleApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    get_rate_limit_lifecycle_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::FetchRateLimitReportRequest,
            ratelimiting::FetchRateLimitReportResponse,
        >,
    >,
    renew_rate_limit_0_port:
        acme_platform::ports::Callback<appointments::CountAppointmentsRequest, appointments::CountAppointmentsResponse>,
    terminate_rate_limit_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::CountRateLimitTotalsRequest,
            ratelimiting::CountRateLimitTotalsResponse,
        >,
    >,
    terminate_rate_limit_3_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                authorization::ResolveRoleBindingsRequest,
                authorization::ResolveRoleBindingsResponse,
            >,
    >,
}
impl RateLimitLifecycleApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::CountRateLimitTotalsRequest,
            ratelimiting::CountRateLimitTotalsResponse,
        > = {
            let client = RateLimitReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CountRateLimitTotalsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CountRateLimitTotalsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_rate_limit_totals(request).await?.into_inner())
})
})
        };
        let terminate_rate_limit_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            authorization::ResolveRoleBindingsRequest,
            authorization::ResolveRoleBindingsResponse,
        > = {
            let client = RoleBindingQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::ResolveRoleBindingsRequest| -> acme_platform::ports::RpcFuture<authorization::ResolveRoleBindingsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_role_bindings(request).await?.into_inner())
})
})
        };
        let terminate_rate_limit_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            appointments::CountAppointmentsRequest,
            appointments::CountAppointmentsResponse,
        > = {
            let client = AppointmentQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: appointments::CountAppointmentsRequest| -> acme_platform::ports::RpcFuture<appointments::CountAppointmentsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_appointments(request).await?.into_inner())
})
})
        };
        let renew_rate_limit_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::FetchRateLimitReportRequest,
            ratelimiting::FetchRateLimitReportResponse,
        > = {
            let client = RateLimitReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::FetchRateLimitReportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::FetchRateLimitReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.fetch_rate_limit_report(request).await?.into_inner())
})
})
        };
        let get_rate_limit_lifecycle_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            get_rate_limit_lifecycle_0_port,
            renew_rate_limit_0_port,
            terminate_rate_limit_2_port,
            terminate_rate_limit_3_port,
        }
    }
    /// Run ActivateRateLimit after validating the tenant and durably recording the command.
    async fn run_activate_rate_limit(
        &self,
        req: ratelimiting::ActivateRateLimitRequest,
    ) -> Result<ratelimiting::ActivateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ActivateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ActivateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::NormalizeRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::normalize_rate_limit_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = search::ResolveSearchIndexesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = RateLimitValidationServiceClient::new(channel_0);
                let input: tonic::Request<ratelimiting::NormalizeRateLimitRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.normalize_rate_limit(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = SearchIndexQueryServiceClient::new(channel_1);
                let input: tonic::Request<search::ResolveSearchIndexesRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_search_indexes(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::ActivateRateLimitResponse::default();
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
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
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run SuspendRateLimit after validating the tenant and durably recording the command.
    async fn run_suspend_rate_limit(
        &self,
        req: ratelimiting::SuspendRateLimitRequest,
    ) -> Result<ratelimiting::SuspendRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuspendRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuspendRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = authorization::CountRoleBindingsRequest {
            context: req.context.clone(),
            filter: Some(authorization::count_role_bindings_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::PreviewThrottleRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::preview_throttle_request::Details {
                contact: Some(common::ContactInfo {
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = RoleBindingQueryServiceClient::new(channel_0);
                let input: tonic::Request<authorization::CountRoleBindingsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_role_bindings(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ThrottleValidationServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::PreviewThrottleRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.preview_throttle(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::SuspendRateLimitResponse::default();
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
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
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
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TerminateRateLimit after validating the tenant and durably recording the command.
    async fn run_terminate_rate_limit(
        &self,
        req: ratelimiting::TerminateRateLimitRequest,
    ) -> Result<ratelimiting::TerminateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TerminateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TerminateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PushRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::push_rate_limit_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::PreviewQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PreviewQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::CountRateLimitTotalsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::count_rate_limit_totals_request::Filter {
                created_after: reply_1
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_1.quota_bucket.clone().unwrap_or_default().labels.clone(),
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
        let reply_2 = acme_platform::ports::execute(self.terminate_rate_limit_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = authorization::ResolveRoleBindingsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_3 = acme_platform::ports::execute(self.terminate_rate_limit_3_port.as_ref(), outgoing_3).await?;
        let mut response = ratelimiting::TerminateRateLimitResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i64 = reply_0.rate_limit.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RenewRateLimit after validating the tenant and durably recording the command.
    async fn run_renew_rate_limit(
        &self,
        req: ratelimiting::RenewRateLimitRequest,
    ) -> Result<ratelimiting::RenewRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RenewRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RenewRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = appointments::CountAppointmentsRequest {
            context: req.context.clone(),
            filter: Some(appointments::count_appointments_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: req.annotations.clone(),
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
        let port = acme_platform::ports::opaque(self.renew_rate_limit_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = ratelimiting::RenewRateLimitResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
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
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ExpireRateLimit after validating the tenant and durably recording the command.
    async fn run_expire_rate_limit(
        &self,
        req: ratelimiting::ExpireRateLimitRequest,
    ) -> Result<ratelimiting::ExpireRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExpireRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExpireRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = authorization::ExportRoleBindingsRequest {
            context: req.context.clone(),
            filter: Some(authorization::export_role_bindings_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
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
        let _reply_0 = async move {
            let mut client = RoleBindingQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::ExportRoleBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_role_bindings(input).await?;
            let mut frames = response.into_inner();
            let mut last = authorization::ExportRoleBindingsResponse::default();
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
        let outgoing_1 = ratelimiting::ReconcileRateLimitsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::reconcile_rate_limits_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
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
                        .notify_contact
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
            rate_limit_id: req.rate_limit_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ReconcileRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ExpireRateLimitResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .new_owner
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
        Ok(response)
    }
    /// Run GetRateLimitLifecycle after validating the tenant and durably recording the command.
    async fn run_get_rate_limit_lifecycle(
        &self,
        req: ratelimiting::GetRateLimitLifecycleRequest,
    ) -> Result<ratelimiting::GetRateLimitLifecycleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRateLimitLifecycle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRateLimitLifecycle", "running workflow");
        let outgoing_0 = ratelimiting::FetchRateLimitReportRequest {
            context: req.context.clone(),
            rate_limit_id: req.rate_limit_id.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.get_rate_limit_lifecycle_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = ratelimiting::GetRateLimitLifecycleResponse::default();
        let contribution: i64 = reply_0.rate_limit.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = reply_0.rate_limit.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleRateLimitExpiry after validating the tenant and durably recording the command.
    async fn run_schedule_rate_limit_expiry(
        &self,
        req: ratelimiting::ScheduleRateLimitExpiryRequest,
    ) -> Result<ratelimiting::ScheduleRateLimitExpiryResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleRateLimitExpiry", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleRateLimitExpiry", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::SyncQuotaBucketsRequest {
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
                limits: vec![ratelimiting::QuotaBucketLimit {
                    name: req.rate_limit.clone().unwrap_or_default().name.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::CompareCryptoKeyOptionsRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
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
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = QuotaBucketSyncServiceClient::new(channel_0);
                let input: tonic::Request<ratelimiting::SyncQuotaBucketsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.sync_quota_buckets(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CryptoKeyCalculationServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::CompareCryptoKeyOptionsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.compare_crypto_key_options(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = search::FindSearchIndexesRequest {
            context: req.context.clone(),
            filter: Some(search::find_search_indexes_request::Filter {
                created_after: req
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: reply_1.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_0
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
        };
        let outgoing_2 = acme_platform::codec::through_any(&outgoing_2, "acme.search.v1.FindSearchIndexesRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = SearchIndexQueryServiceClient::new(channel);
            let input: tonic::Request<search::FindSearchIndexesRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_search_indexes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ScheduleRateLimitExpiryResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
}
#[tonic::async_trait]
impl RateLimitLifecycleService for RateLimitLifecycleApi {
    /// Activates a rate limit.
    async fn activate_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::ActivateRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::ActivateRateLimitResponse>, tonic::Status> {
        self.run_activate_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Suspends a rate limit.
    async fn suspend_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::SuspendRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::SuspendRateLimitResponse>, tonic::Status> {
        self.run_suspend_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Terminates a rate limit.
    async fn terminate_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::TerminateRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::TerminateRateLimitResponse>, tonic::Status> {
        self.run_terminate_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Renews a rate limit.
    async fn renew_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::RenewRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::RenewRateLimitResponse>, tonic::Status> {
        self.run_renew_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Expires a rate limit.
    async fn expire_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::ExpireRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::ExpireRateLimitResponse>, tonic::Status> {
        self.run_expire_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a rate limit.
    async fn get_rate_limit_lifecycle(
        &self,
        request: tonic::Request<ratelimiting::GetRateLimitLifecycleRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetRateLimitLifecycleResponse>, tonic::Status> {
        self.run_get_rate_limit_lifecycle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a rate limit.
    async fn schedule_rate_limit_expiry(
        &self,
        request: tonic::Request<ratelimiting::ScheduleRateLimitExpiryRequest>,
    ) -> Result<tonic::Response<ratelimiting::ScheduleRateLimitExpiryResponse>, tonic::Status> {
        self.run_schedule_rate_limit_expiry(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
