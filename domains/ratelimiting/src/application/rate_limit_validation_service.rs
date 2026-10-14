use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_validation_service_client::PermissionCheckValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_service_client::DocumentServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_query_service_client::MetricSeriesQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_server::RateLimitValidationService;
use acme_proto::acme::shows::v1 as shows;
use acme_proto::acme::shows::v1::show_service_client::ShowServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Checks rate limits against the rules before anything acts on them.
#[derive(Clone)]
pub struct RateLimitValidationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    check_rate_limit_eligibility_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<documents::CreateDocumentRequest, documents::CreateDocumentResponse>,
    >,
    check_rate_limit_eligibility_3_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<featureflags::SyncFlagRolloutsRequest, featureflags::SyncFlagRolloutsResponse>,
    >,
    preview_rate_limit_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ValidateEnvironmentRequest,
            configuration::ValidateEnvironmentResponse,
        >,
    >,
    validate_rate_limit_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::SimulateQuotaBucketRequest,
                ratelimiting::SimulateQuotaBucketResponse,
            >,
    >,
    validate_rate_limit_1_port:
        acme_platform::ports::Callback<configuration::ScoreEnvironmentRequest, configuration::ScoreEnvironmentResponse>,
    validate_rate_limit_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<configuration::LintConfigSetsRequest, configuration::LintConfigSetsResponse>,
    >,
}
impl RateLimitValidationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::SimulateQuotaBucketRequest,
            ratelimiting::SimulateQuotaBucketResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::SimulateQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::SimulateQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_quota_bucket(request).await?.into_inner())
})
})
        };
        let validate_rate_limit_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ScoreEnvironmentRequest,
            configuration::ScoreEnvironmentResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ScoreEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::ScoreEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_environment(request).await?.into_inner())
})
})
        };
        let validate_rate_limit_1_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::LintConfigSetsRequest,
            configuration::LintConfigSetsResponse,
        > = {
            let client = ConfigSetValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::LintConfigSetsRequest| -> acme_platform::ports::RpcFuture<configuration::LintConfigSetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_config_sets(request).await?.into_inner())
})
})
        };
        let validate_rate_limit_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::ValidateEnvironmentRequest,
            configuration::ValidateEnvironmentResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ValidateEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::ValidateEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.validate_environment(request).await?.into_inner())
})
})
        };
        let preview_rate_limit_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            documents::CreateDocumentRequest,
            documents::CreateDocumentResponse,
        > = {
            let client = DocumentServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: documents::CreateDocumentRequest| -> acme_platform::ports::RpcFuture<documents::CreateDocumentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.create_document(request).await?.into_inner())
})
})
        };
        let check_rate_limit_eligibility_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::SyncFlagRolloutsRequest,
            featureflags::SyncFlagRolloutsResponse,
        > = {
            let client = FlagRolloutSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::SyncFlagRolloutsRequest| -> acme_platform::ports::RpcFuture<featureflags::SyncFlagRolloutsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.sync_flag_rollouts(request).await?.into_inner())
})
})
        };
        let check_rate_limit_eligibility_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            check_rate_limit_eligibility_2_port,
            check_rate_limit_eligibility_3_port,
            preview_rate_limit_2_port,
            validate_rate_limit_0_port,
            validate_rate_limit_1_port,
            validate_rate_limit_2_port,
        }
    }
    /// Run ValidateRateLimit after validating the tenant and durably recording the command.
    async fn run_validate_rate_limit(
        &self,
        req: ratelimiting::ValidateRateLimitRequest,
    ) -> Result<ratelimiting::ValidateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::SimulateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.validate_rate_limit_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::ScoreEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
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
        let reply_1 = (self.validate_rate_limit_1_port)(outgoing_1).await?;
        let outgoing_2 = configuration::LintConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            lines: (match reply_1.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .lines
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
        let reply_2 =
            self.validate_rate_limit_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let mut response = ratelimiting::ValidateRateLimitResponse::default();
        let contribution: String = reply_0
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
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .environment
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
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_2
            .config_set
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
    /// Run PreviewRateLimit after validating the tenant and durably recording the command.
    async fn run_preview_rate_limit(
        &self,
        req: ratelimiting::PreviewRateLimitRequest,
    ) -> Result<ratelimiting::PreviewRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = metrics::SummarizeMetricSeriesesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::ExplainQuotaBucketCalculationRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = MetricSeriesQueryServiceClient::new(channel_0);
                let input: tonic::Request<metrics::SummarizeMetricSeriesesRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.summarize_metric_serieses(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = QuotaBucketCalculationServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::ExplainQuotaBucketCalculationRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_quota_bucket_calculation(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = configuration::ValidateEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
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
        let reply_2 = acme_platform::ports::execute(self.preview_rate_limit_2_port.as_ref(), outgoing_2).await?;
        let mut response = ratelimiting::PreviewRateLimitResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i64 = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .rate_limit_id
            .clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run CheckRateLimitEligibility after validating the tenant and durably recording the command.
    async fn run_check_rate_limit_eligibility(
        &self,
        req: ratelimiting::CheckRateLimitEligibilityRequest,
    ) -> Result<ratelimiting::CheckRateLimitEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckRateLimitEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckRateLimitEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::ScorePermissionCheckRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            permission_check: Some(authorization::PermissionCheck {
                subject: Some(authorization::AccessPolicyRef {
                    display_name: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: req.rate_limit.clone().unwrap_or_default().version.clone(),
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
        let outgoing_1 = featureflags::ReconcileFlagRolloutsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
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
                participants: vec![featureflags::FlagRolloutParticipant {
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
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = PermissionCheckValidationServiceClient::new(channel_0);
                let input: tonic::Request<authorization::ScorePermissionCheckRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.score_permission_check(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = FlagRolloutSyncServiceClient::new(channel_1);
                let input: tonic::Request<featureflags::ReconcileFlagRolloutsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.reconcile_flag_rollouts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = documents::CreateDocumentRequest {
            context: req.context.clone(),
            document: Some(documents::Document {
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
                billing_address: Some(common::PostalAddress {
                    region: reply_1
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .region
                        .clone(),
                    ..Default::default()
                }),
                lines: vec![documents::DocumentLine {
                    tax: Some(documents::DocumentLineTax {
                        amount: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                            Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .amount
                        .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 =
            acme_platform::ports::execute(self.check_rate_limit_eligibility_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = featureflags::SyncFlagRolloutsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
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
                    updated_by: reply_1
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                location: Some(common::PostalAddress {
                    locality: reply_2
                        .document
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_3 =
            acme_platform::ports::execute(self.check_rate_limit_eligibility_3_port.as_ref(), outgoing_3).await?;
        let mut response = ratelimiting::CheckRateLimitEligibilityResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1
            .flag_rollout
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3
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
        let contribution: Option<common::TimeWindow> = req.rate_limit.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeRateLimit after validating the tenant and durably recording the command.
    async fn run_normalize_rate_limit(
        &self,
        req: ratelimiting::NormalizeRateLimitRequest,
    ) -> Result<ratelimiting::NormalizeRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFlagRolloutConflictsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_rollout_conflicts_request::Filter {
                created_after: req
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
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::CalculateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = FlagRolloutSyncServiceClient::new(channel_0);
                let input: tonic::Request<featureflags::ListFlagRolloutConflictsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_flag_rollout_conflicts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = QuotaBucketCalculationServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::CalculateQuotaBucketRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.calculate_quota_bucket(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::NormalizeRateLimitResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .rate_limit_id
            .clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i32 = req.rate_limit.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).priority = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.rate_limit.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run ExplainRateLimitDecision after validating the tenant and durably recording the command.
    async fn run_explain_rate_limit_decision(
        &self,
        req: ratelimiting::ExplainRateLimitDecisionRequest,
    ) -> Result<ratelimiting::ExplainRateLimitDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainRateLimitDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainRateLimitDecision", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::ExplainPermissionCheckDecisionRequest {
            context: req.context.clone(),
            details: Some(authorization::explain_permission_check_decision_request::Details {
                contact: Some(common::ContactInfo {
                    phone: req
                        .details
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::ExplainQuotaBucketCalculationRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_quota_bucket_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = PermissionCheckValidationServiceClient::new(channel_0);
                let input: tonic::Request<authorization::ExplainPermissionCheckDecisionRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_permission_check_decision(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = QuotaBucketCalculationServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::ExplainQuotaBucketCalculationRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_quota_bucket_calculation(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = ratelimiting::CompareQuotaBucketOptionsRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_at: reply_1
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
                funding: Some(ratelimiting::quota_bucket::Funding::Bank(
                    ratelimiting::QuotaBucketBankFunding {
                        iban: (match reply_1.quota_bucket.clone().unwrap_or_default().funding.clone() {
                            Some(ratelimiting::quota_bucket::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .iban
                        .clone(),
                        ..Default::default()
                    },
                )),
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
                holder: Some(ratelimiting::RateLimitRef {
                    version: reply_0
                        .permission_check
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
            let input: tonic::Request<ratelimiting::CompareQuotaBucketOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_quota_bucket_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ExplainRateLimitDecisionResponse::default();
        let contribution: i64 = reply_0
            .permission_check
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
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
    /// Run LintRateLimits after validating the tenant and durably recording the command.
    async fn run_lint_rate_limits(
        &self,
        req: ratelimiting::LintRateLimitsRequest,
    ) -> Result<ratelimiting::LintRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = venues::SuggestVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::suggest_venues_request::Filter {
                created_after: req
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.annotations.clone(),
                owner_id: req
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
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.venues.v1.SuggestVenuesRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueQueryServiceClient::new(channel);
            let input: tonic::Request<venues::SuggestVenuesRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::LintRateLimitsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
    /// Run GetRateLimitRules after validating the tenant and durably recording the command.
    async fn run_get_rate_limit_rules(
        &self,
        req: ratelimiting::GetRateLimitRulesRequest,
    ) -> Result<ratelimiting::GetRateLimitRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRateLimitRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRateLimitRules", "running workflow");
        let outgoing_0 = shows::BatchGetShowsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
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
                holder: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: req.rate_limit_id.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ShowServiceClient::new(channel_0);
                let input: tonic::Request<shows::BatchGetShowsRequest> = tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.batch_get_shows(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = QuotaBucketCalculationServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::QuoteQuotaBucketRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.quote_quota_bucket(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::GetRateLimitRulesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl RateLimitValidationService for RateLimitValidationEndpoint {
    /// Validates a rate limit.
    async fn validate_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::ValidateRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::ValidateRateLimitResponse>, tonic::Status> {
        self.run_validate_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a rate limit.
    async fn preview_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::PreviewRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::PreviewRateLimitResponse>, tonic::Status> {
        self.run_preview_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a rate limit.
    async fn check_rate_limit_eligibility(
        &self,
        request: tonic::Request<ratelimiting::CheckRateLimitEligibilityRequest>,
    ) -> Result<tonic::Response<ratelimiting::CheckRateLimitEligibilityResponse>, tonic::Status> {
        self.run_check_rate_limit_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a rate limit.
    async fn normalize_rate_limit(
        &self,
        request: tonic::Request<ratelimiting::NormalizeRateLimitRequest>,
    ) -> Result<tonic::Response<ratelimiting::NormalizeRateLimitResponse>, tonic::Status> {
        self.run_normalize_rate_limit(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a rate limit.
    async fn explain_rate_limit_decision(
        &self,
        request: tonic::Request<ratelimiting::ExplainRateLimitDecisionRequest>,
    ) -> Result<tonic::Response<ratelimiting::ExplainRateLimitDecisionResponse>, tonic::Status> {
        self.run_explain_rate_limit_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a rate limit.
    async fn lint_rate_limits(
        &self,
        request: tonic::Request<ratelimiting::LintRateLimitsRequest>,
    ) -> Result<tonic::Response<ratelimiting::LintRateLimitsResponse>, tonic::Status> {
        self.run_lint_rate_limits(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a rate limit.
    async fn get_rate_limit_rules(
        &self,
        request: tonic::Request<ratelimiting::GetRateLimitRulesRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetRateLimitRulesResponse>, tonic::Status> {
        self.run_get_rate_limit_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
