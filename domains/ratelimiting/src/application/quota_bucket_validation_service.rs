use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_history_service_client::PermissionCheckHistoryServiceClient;
use acme_proto::acme::authorization::v1::permission_check_validation_service_client::PermissionCheckValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_query_service_client::ConfigSetQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_validation_service_client::MetricSeriesValidationServiceClient;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::patients::v1::patient_service_client::PatientServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_server::QuotaBucketValidationService;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_client::ThrottleCalculationServiceClient;

/// Checks quota buckets against the rules before anything acts on them.
#[derive(Clone)]
pub struct QuotaBucketValidationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    check_quota_bucket_eligibility_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::CheckRateLimitEligibilityRequest,
                ratelimiting::CheckRateLimitEligibilityResponse,
            >,
    >,
    check_quota_bucket_eligibility_1_port: acme_platform::ports::Callback<
        configuration::ExplainConfigChangeCalculationRequest,
        configuration::ExplainConfigChangeCalculationResponse,
    >,
    check_quota_bucket_eligibility_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::EstimateThrottleRequest, ratelimiting::EstimateThrottleResponse>,
    >,
    explain_quota_bucket_decision_0_port: acme_platform::ports::Callback<
        ratelimiting::CheckRateLimitEligibilityRequest,
        ratelimiting::CheckRateLimitEligibilityResponse,
    >,
    score_quota_bucket_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<configuration::BrowseConfigSetsRequest, configuration::BrowseConfigSetsResponse>,
    >,
    score_quota_bucket_2_port: acme_platform::ports::Callback<
        authorization::ValidatePermissionCheckRequest,
        authorization::ValidatePermissionCheckResponse,
    >,
    validate_quota_bucket_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<patients::CreatePatientRequest, patients::CreatePatientResponse>,
    >,
}
impl QuotaBucketValidationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<patients::CreatePatientRequest, patients::CreatePatientResponse> = {
            let client = PatientServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: patients::CreatePatientRequest| -> acme_platform::ports::RpcFuture<patients::CreatePatientResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.create_patient(request).await?.into_inner())
})
})
        };
        let validate_quota_bucket_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::CheckRateLimitEligibilityRequest,
            ratelimiting::CheckRateLimitEligibilityResponse,
        > = {
            let client = RateLimitValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CheckRateLimitEligibilityRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CheckRateLimitEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_rate_limit_eligibility(request).await?.into_inner())
})
})
        };
        let check_quota_bucket_eligibility_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ExplainConfigChangeCalculationRequest,
            configuration::ExplainConfigChangeCalculationResponse,
        > = {
            let client = ConfigChangeCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ExplainConfigChangeCalculationRequest| -> acme_platform::ports::RpcFuture<configuration::ExplainConfigChangeCalculationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_config_change_calculation(request).await?.into_inner())
})
})
        };
        let check_quota_bucket_eligibility_1_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::EstimateThrottleRequest,
            ratelimiting::EstimateThrottleResponse,
        > = {
            let client = ThrottleCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::EstimateThrottleRequest| -> acme_platform::ports::RpcFuture<ratelimiting::EstimateThrottleResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_throttle(request).await?.into_inner())
})
})
        };
        let check_quota_bucket_eligibility_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::CheckRateLimitEligibilityRequest,
            ratelimiting::CheckRateLimitEligibilityResponse,
        > = {
            let client = RateLimitValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CheckRateLimitEligibilityRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CheckRateLimitEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_rate_limit_eligibility(request).await?.into_inner())
})
})
        };
        let explain_quota_bucket_decision_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::BrowseConfigSetsRequest,
            configuration::BrowseConfigSetsResponse,
        > = {
            let client = ConfigSetQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::BrowseConfigSetsRequest| -> acme_platform::ports::RpcFuture<configuration::BrowseConfigSetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_config_sets(request).await?.into_inner())
})
})
        };
        let score_quota_bucket_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let score_quota_bucket_2_port = callback;
        Self {
            channel,
            repository,
            check_quota_bucket_eligibility_0_port,
            check_quota_bucket_eligibility_1_port,
            check_quota_bucket_eligibility_2_port,
            explain_quota_bucket_decision_0_port,
            score_quota_bucket_0_port,
            score_quota_bucket_2_port,
            validate_quota_bucket_0_port,
        }
    }
    /// Run ValidateQuotaBucket after validating the tenant and durably recording the command.
    async fn run_validate_quota_bucket(
        &self,
        req: ratelimiting::ValidateQuotaBucketRequest,
    ) -> Result<ratelimiting::ValidateQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = patients::CreatePatientRequest {
            context: req.context.clone(),
            patient: Some(patients::Patient {
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
                    created_by: req
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
                contact: Some(common::ContactInfo {
                    name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
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
        let reply_0 =
            self.validate_quota_bucket_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = ratelimiting::ValidateQuotaBucketResponse::default();
        let contribution: String = reply_0
            .patient
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .preferred_channel
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .preferred_channel = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.quota_bucket.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PreviewQuotaBucket after validating the tenant and durably recording the command.
    async fn run_preview_quota_bucket(
        &self,
        req: ratelimiting::PreviewQuotaBucketRequest,
    ) -> Result<ratelimiting::PreviewQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::AnnotatePermissionCheckVersionRequest {
            context: req.context.clone(),
            details: Some(authorization::annotate_permission_check_version_request::Details {
                author_email: req
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            permission_check: Some(authorization::PermissionCheck {
                subject: Some(authorization::AccessPolicyRef {
                    display_name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = PermissionCheckHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::AnnotatePermissionCheckVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.annotate_permission_check_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::RecalculateConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        created_by: req
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
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            etag: req
                                .quota_bucket
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: reply_0
                                .permission_check
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::PreviewQuotaBucketResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_1
            .config_set
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
        let contribution: i32 = req.quota_bucket.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run CheckQuotaBucketEligibility after validating the tenant and durably recording the command.
    async fn run_check_quota_bucket_eligibility(
        &self,
        req: ratelimiting::CheckQuotaBucketEligibilityRequest,
    ) -> Result<ratelimiting::CheckQuotaBucketEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckQuotaBucketEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckQuotaBucketEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CheckRateLimitEligibilityRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::check_rate_limit_eligibility_request::Details {
                contact: Some(common::ContactInfo {
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.check_quota_bucket_eligibility_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::ExplainConfigChangeCalculationRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: reply_0
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
            ..Default::default()
        };
        let reply_1 = (self.check_quota_bucket_eligibility_1_port)(outgoing_1).await?;
        let outgoing_2 = ratelimiting::EstimateThrottleRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    version: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
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
        let reply_2 =
            self.check_quota_bucket_eligibility_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let mut response = ratelimiting::CheckQuotaBucketEligibilityResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
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
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::Money> = req.quota_bucket.clone().unwrap_or_default().available.clone();
        if contribution != <Option<common::Money>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).available = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeQuotaBucket after validating the tenant and durably recording the command.
    async fn run_normalize_quota_bucket(
        &self,
        req: ratelimiting::NormalizeQuotaBucketRequest,
    ) -> Result<ratelimiting::NormalizeQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::SubscribeFlagRolloutUpdatesRequest {
            context: req.context.clone(),
            details: Some(featureflags::subscribe_flag_rollout_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .details
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<featureflags::SubscribeFlagRolloutUpdatesRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = FlagRolloutNotificationServiceClient::new(channel);
                let input: tonic::Request<featureflags::SubscribeFlagRolloutUpdatesRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.subscribe_flag_rollout_updates(input).await?;
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
        let jobs_1 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = metrics::CheckMetricSeriesEligibilityRequest {
                        context: req.context.clone(),
                        metric_series: Some(metrics::MetricSeries {
                            audit: Some(common::AuditInfo {
                                etag: req
                                    .quota_bucket
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .etag
                                    .clone(),
                                ..Default::default()
                            }),
                            name: item.name.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = MetricSeriesValidationServiceClient::new(channel);
                        let input: tonic::Request<metrics::CheckMetricSeriesEligibilityRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.check_metric_series_eligibility(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = ratelimiting::NormalizeQuotaBucketResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::Money> = req.quota_bucket.clone().unwrap_or_default().available.clone();
        if contribution != <Option<common::Money>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).available = contribution;
        }
        Ok(response)
    }
    /// Run ExplainQuotaBucketDecision after validating the tenant and durably recording the command.
    async fn run_explain_quota_bucket_decision(
        &self,
        req: ratelimiting::ExplainQuotaBucketDecisionRequest,
    ) -> Result<ratelimiting::ExplainQuotaBucketDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainQuotaBucketDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainQuotaBucketDecision", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CheckRateLimitEligibilityRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
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
        let port = acme_platform::ports::opaque(self.explain_quota_bucket_decision_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = ratelimiting::PreviewRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_at: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<ratelimiting::PreviewRateLimitRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PreviewRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ExplainQuotaBucketDecisionResponse::default();
        let contribution: i64 = reply_0.rate_limit.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = req.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run ScoreQuotaBucket after validating the tenant and durably recording the command.
    async fn run_score_quota_bucket(
        &self,
        req: ratelimiting::ScoreQuotaBucketRequest,
    ) -> Result<ratelimiting::ScoreQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::BrowseConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::browse_config_sets_request::Filter {
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
        let reply_0 = acme_platform::ports::execute(self.score_quota_bucket_0_port.as_ref(), outgoing_0).await?;
        let jobs_1 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = configuration::PreviewConfigSetRequest {
                        config_set: Some(configuration::ConfigSet {
                            children: vec![configuration::ConfigSet {
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
                                    children: vec![configuration::ConfigSet {
                                        name: item.name.clone(),
                                        ..Default::default()
                                    }],
                                    location: Some(common::PostalAddress {
                                        recipient: req
                                            .details
                                            .clone()
                                            .unwrap_or_default()
                                            .destination
                                            .clone()
                                            .unwrap_or_default()
                                            .recipient
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
                                }],
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = ConfigSetValidationServiceClient::new(channel);
                        let input: tonic::Request<configuration::PreviewConfigSetRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.preview_config_set(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = authorization::ValidatePermissionCheckRequest {
            context: req.context.clone(),
            details: Some(authorization::validate_permission_check_request::Details {
                contact: Some(common::ContactInfo {
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
            permission_check: Some(authorization::PermissionCheck {
                subject: Some(authorization::AccessPolicyRef {
                    display_name: reply_1
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
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
        let reply_2 = (self.score_quota_bucket_2_port)(outgoing_2).await?;
        let mut response = ratelimiting::ScoreQuotaBucketResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run LintQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_lint_quota_buckets(
        &self,
        req: ratelimiting::LintQuotaBucketsRequest,
    ) -> Result<ratelimiting::LintQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintQuotaBuckets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let command_0 = LintQuotaBucketsDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::PreviewRateLimitRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PreviewRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::UpdateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            display_name: reply_0
                                .rate_limit
                                .clone()
                                .unwrap_or_default()
                                .owner
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: reply_0
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.configuration.v1.UpdateConfigSetRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::UpdateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::LintQuotaBucketsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
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
        let contribution: String = reply_1.config_set.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .limits
                .is_empty()
            {
                response
                    .quota_bucket
                    .get_or_insert_with(Default::default)
                    .limits
                    .push(Default::default());
            }
            response.quota_bucket.get_or_insert_with(Default::default).limits[0].name = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::Money> = req.quota_bucket.clone().unwrap_or_default().balance.clone();
        if contribution != <Option<common::Money>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).balance = contribution;
        }
        Ok(response)
    }
    /// Run GetQuotaBucketRules after validating the tenant and durably recording the command.
    async fn run_get_quota_bucket_rules(
        &self,
        req: ratelimiting::GetQuotaBucketRulesRequest,
    ) -> Result<ratelimiting::GetQuotaBucketRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetQuotaBucketRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetQuotaBucketRules", "running workflow");
        let mut response = ratelimiting::GetQuotaBucketRulesResponse::default();
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by LintQuotaBuckets for PreviewRateLimit.
struct LintQuotaBucketsDispatch0 {
    context: Option<common::RequestContext>,
    rate_limit_audit_updated_at: Option<prost_types::Timestamp>,
    rate_limit_audit_updated_by: String,
    rate_limit_audit_created_at: Option<prost_types::Timestamp>,
    details_contact_name: Option<common::PersonName>,
}
impl From<ratelimiting::LintQuotaBucketsRequest> for LintQuotaBucketsDispatch0 {
    fn from(req: ratelimiting::LintQuotaBucketsRequest) -> Self {
        Self {
            context: req.context.clone(),
            rate_limit_audit_updated_at: req
                .quota_bucket
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
            rate_limit_audit_updated_by: req
                .quota_bucket
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .updated_by
                .clone(),
            rate_limit_audit_created_at: req
                .quota_bucket
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
            details_contact_name: req
                .details
                .clone()
                .unwrap_or_default()
                .contact
                .clone()
                .unwrap_or_default()
                .name
                .clone(),
        }
    }
}
impl From<LintQuotaBucketsDispatch0> for ratelimiting::PreviewRateLimitRequest {
    fn from(value: LintQuotaBucketsDispatch0) -> Self {
        ratelimiting::PreviewRateLimitRequest {
            context: value.context,
            details: Some(ratelimiting::preview_rate_limit_request::Details {
                contact: Some(common::ContactInfo {
                    name: value.details_contact_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_at: value.rate_limit_audit_created_at,
                    updated_at: value.rate_limit_audit_updated_at,
                    updated_by: value.rate_limit_audit_updated_by,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl QuotaBucketValidationService for QuotaBucketValidationApi {
    /// Validates a quota bucket.
    async fn validate_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::ValidateQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::ValidateQuotaBucketResponse>, tonic::Status> {
        self.run_validate_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a quota bucket.
    async fn preview_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::PreviewQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::PreviewQuotaBucketResponse>, tonic::Status> {
        self.run_preview_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a quota bucket.
    async fn check_quota_bucket_eligibility(
        &self,
        request: tonic::Request<ratelimiting::CheckQuotaBucketEligibilityRequest>,
    ) -> Result<tonic::Response<ratelimiting::CheckQuotaBucketEligibilityResponse>, tonic::Status> {
        self.run_check_quota_bucket_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a quota bucket.
    async fn normalize_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::NormalizeQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::NormalizeQuotaBucketResponse>, tonic::Status> {
        self.run_normalize_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a quota bucket.
    async fn explain_quota_bucket_decision(
        &self,
        request: tonic::Request<ratelimiting::ExplainQuotaBucketDecisionRequest>,
    ) -> Result<tonic::Response<ratelimiting::ExplainQuotaBucketDecisionResponse>, tonic::Status> {
        self.run_explain_quota_bucket_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a quota bucket.
    async fn score_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::ScoreQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::ScoreQuotaBucketResponse>, tonic::Status> {
        self.run_score_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a quota bucket.
    async fn lint_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::LintQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::LintQuotaBucketsResponse>, tonic::Status> {
        self.run_lint_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a quota bucket.
    async fn get_quota_bucket_rules(
        &self,
        request: tonic::Request<ratelimiting::GetQuotaBucketRulesRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetQuotaBucketRulesResponse>, tonic::Status> {
        self.run_get_quota_bucket_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
