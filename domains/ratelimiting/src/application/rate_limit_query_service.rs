use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::boxoffice::v1::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::patients::v1::patient_query_service_client::PatientQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_server::RateLimitQueryService;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_client::ThrottleCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::telehealth::v1 as telehealth;
use acme_proto::acme::telehealth::v1::video_visit_query_service_client::VideoVisitQueryServiceClient;

/// Answers read-only questions about rate limits, for screens and reports.
#[derive(Clone)]
pub struct RateLimitQueryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    browse_rate_limits_0_port:
        acme_platform::ports::Callback<ratelimiting::CalculateThrottleRequest, ratelimiting::CalculateThrottleResponse>,
    browse_rate_limits_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::LintThrottlesRequest, ratelimiting::LintThrottlesResponse>,
    >,
}
impl RateLimitQueryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::CalculateThrottleRequest,
            ratelimiting::CalculateThrottleResponse,
        > = {
            let client = ThrottleCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CalculateThrottleRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CalculateThrottleResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_throttle(request).await?.into_inner())
})
})
        };
        let browse_rate_limits_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::LintThrottlesRequest,
            ratelimiting::LintThrottlesResponse,
        > = {
            let client = ThrottleValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::LintThrottlesRequest| -> acme_platform::ports::RpcFuture<ratelimiting::LintThrottlesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_throttles(request).await?.into_inner())
})
})
        };
        let browse_rate_limits_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            browse_rate_limits_0_port,
            browse_rate_limits_1_port,
        }
    }
    /// Run LookupRateLimit after validating the tenant and durably recording the command.
    async fn run_lookup_rate_limit(
        &self,
        req: ratelimiting::LookupRateLimitRequest,
    ) -> Result<ratelimiting::LookupRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupRateLimit", "running workflow");
        let mut response = ratelimiting::LookupRateLimitResponse::default();
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseRateLimits after validating the tenant and durably recording the command.
    async fn run_browse_rate_limits(
        &self,
        req: ratelimiting::BrowseRateLimitsRequest,
    ) -> Result<ratelimiting::BrowseRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CalculateThrottleRequest {
            annotations: req.filter.clone().unwrap_or_default().labels.clone(),
            context: req.context.clone(),
            details: Some(ratelimiting::calculate_throttle_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                correlation_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                occurred_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.browse_rate_limits_0_port)(outgoing_0).await?;
        let outgoing_1 = ratelimiting::LintThrottlesRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: reply_0
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
        let reply_1 =
            self.browse_rate_limits_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = ratelimiting::BrowseRateLimitsResponse::default();
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
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: String = reply_1.throttle.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run FindRateLimits after validating the tenant and durably recording the command.
    async fn run_find_rate_limits(
        &self,
        req: ratelimiting::FindRateLimitsRequest,
    ) -> Result<ratelimiting::FindRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = telehealth::ExportVideoVisitsRequest {
            context: req.context.clone(),
            filter: Some(telehealth::export_video_visits_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = VideoVisitQueryServiceClient::new(channel);
            let input: tonic::Request<telehealth::ExportVideoVisitsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_video_visits(input).await?;
            let mut frames = response.into_inner();
            let mut last = telehealth::ExportVideoVisitsResponse::default();
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
        let outgoing_1 = featureflags::NotifyFlagEvaluationWatchersRequest {
            annotations: req.filter.clone().unwrap_or_default().labels.clone(),
            context: req.context.clone(),
            details: Some(featureflags::notify_flag_evaluation_watchers_request::Details {
                recipient: Some(common::ContactInfo {
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
                variables: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.featureflags.v1.NotifyFlagEvaluationWatchersRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagEvaluationNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::NotifyFlagEvaluationWatchersRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.notify_flag_evaluation_watchers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::FindRateLimitsResponse::default();
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
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
        Ok(response)
    }
    /// Run ResolveRateLimits after validating the tenant and durably recording the command.
    async fn run_resolve_rate_limits(
        &self,
        req: ratelimiting::ResolveRateLimitsRequest,
    ) -> Result<ratelimiting::ResolveRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveRateLimits", "running workflow");
        let outgoing_0 = ratelimiting::EstimateThrottleRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                attributes: req.context.clone().unwrap_or_default().baggage.clone(),
                correlation_id: req
                    .context
                    .clone()
                    .unwrap_or_default()
                    .actor
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                measurements: vec![ratelimiting::ThrottleMeasurement {
                    observed_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }],
                source: req.context.clone().unwrap_or_default().tenant_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = ThrottleCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateThrottleRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = ratelimiting::ResolveRateLimitsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ExportRateLimits after validating the tenant and durably recording the command.
    async fn run_export_rate_limits(
        &self,
        req: ratelimiting::ExportRateLimitsRequest,
    ) -> Result<ratelimiting::ExportRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = configuration::NormalizeEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    created_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = boxoffice::FindBoxOfficeSalesRequest {
            context: req.context.clone(),
            filter: Some(boxoffice::find_box_office_sales_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = EnvironmentValidationServiceClient::new(channel_0);
                let input: tonic::Request<configuration::NormalizeEnvironmentRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.normalize_environment(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = BoxOfficeSaleQueryServiceClient::new(channel_1);
                let input: tonic::Request<boxoffice::FindBoxOfficeSalesRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.find_box_office_sales(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::ExportRateLimitsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
        Ok(response)
    }
    /// Run SuggestRateLimits after validating the tenant and durably recording the command.
    async fn run_suggest_rate_limits(
        &self,
        req: ratelimiting::SuggestRateLimitsRequest,
    ) -> Result<ratelimiting::SuggestRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::RecalculateQuotaBucketsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::recalculate_quota_buckets_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
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
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RecalculateQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::SuggestRateLimitsResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .quota_bucket
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
        Ok(response)
    }
    /// Run PeekRateLimit after validating the tenant and durably recording the command.
    async fn run_peek_rate_limit(
        &self,
        req: ratelimiting::PeekRateLimitRequest,
    ) -> Result<ratelimiting::PeekRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekRateLimit", "running workflow");
        let outgoing_0 = patients::FindPatientsRequest {
            context: req.context.clone(),
            filter: Some(patients::find_patients_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.rate_limit_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<patients::FindPatientsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = PatientQueryServiceClient::new(channel);
                let input: tonic::Request<patients::FindPatientsRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.find_patients(input).await?;
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
        let mut response = ratelimiting::PeekRateLimitResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl RateLimitQueryService for RateLimitQueryEndpoint {
    /// Returns a rate limit.
    async fn lookup_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::LookupRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::LookupRateLimitResponse>, tonic::Status> {
        self.run_lookup_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists rate limits a page at a time.
    async fn browse_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::BrowseRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::BrowseRateLimitsResponse>, tonic::Status> {
        self.run_browse_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists rate limits a page at a time.
    async fn find_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::FindRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::FindRateLimitsResponse>, tonic::Status> {
        self.run_find_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several rate limits at once.
    async fn resolve_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::ResolveRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ResolveRateLimitsResponse>, tonic::Status> {
        self.run_resolve_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportRateLimitsStream =
        tokio_stream::wrappers::ReceiverStream<Result<ratelimiting::ExportRateLimitsResponse, tonic::Status>>;
    /// Streams every matching rate limit.
    async fn export_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::ExportRateLimitsRequest>,
    ) -> Result<tonic::Response<Self::ExportRateLimitsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_export_rate_limits(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Lists rate limits a page at a time.
    async fn suggest_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::SuggestRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::SuggestRateLimitsResponse>, tonic::Status> {
        self.run_suggest_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a rate limit.
    async fn peek_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::PeekRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::PeekRateLimitResponse>, tonic::Status> {
        self.run_peek_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
