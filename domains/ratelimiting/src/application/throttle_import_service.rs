use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::giftcards::v1::gift_card_calculation_service_client::GiftCardCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_client::QuotaBucketQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_client::RateLimitQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_server::ThrottleImportService;
use acme_proto::acme::sellers::v1 as sellers;
use acme_proto::acme::sellers::v1::seller_query_service_client::SellerQueryServiceClient;

/// Takes throttles in bulk from partners and legacy systems.
#[derive(Clone)]
pub struct ThrottleImportEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
}
impl ThrottleImportEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ImportThrottles after validating the tenant and durably recording the command.
    async fn run_import_throttles(
        &self,
        req: ratelimiting::ImportThrottlesRequest,
    ) -> Result<ratelimiting::ImportThrottlesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ImportThrottles", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ImportThrottles", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(ratelimiting::import_throttles_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = sellers::FindSellersRequest {
            context: req.context.clone(),
            filter: Some(sellers::find_sellers_request::Filter {
                created_after: (match req.chunk.clone() {
                    Some(ratelimiting::import_throttles_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .occurred_at
                .clone(),
                owner_id: (match req.chunk.clone() {
                    Some(ratelimiting::import_throttles_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .correlation_id
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SellerQueryServiceClient::new(channel);
            let input: tonic::Request<sellers::FindSellersRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_sellers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::LintQuotaBucketsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::lint_quota_buckets_request::Details {
                contact: Some(common::ContactInfo {
                    email: (match req.chunk.clone() {
                        Some(ratelimiting::import_throttles_request::Chunk::Header(value)) => value,
                        _ => Default::default(),
                    })
                    .submitted_by
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
            let mut client = QuotaBucketValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::LintQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ExplainRateLimitCalculationRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_rate_limit_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: (match req.chunk.clone() {
                        Some(ratelimiting::import_throttles_request::Chunk::Header(value)) => value,
                        _ => Default::default(),
                    })
                    .submitted_by
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_at: reply_1
                        .quota_bucket
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainRateLimitCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_rate_limit_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ImportThrottlesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = (match req.chunk.clone() {
            Some(ratelimiting::import_throttles_request::Chunk::Record(value)) => value,
            _ => Default::default(),
        })
        .correlation_id
        .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run GetThrottleImportStatus after validating the tenant and durably recording the command.
    async fn run_get_throttle_import_status(
        &self,
        req: ratelimiting::GetThrottleImportStatusRequest,
    ) -> Result<ratelimiting::GetThrottleImportStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetThrottleImportStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetThrottleImportStatus", "running workflow");
        let outgoing_0 = ratelimiting::PushRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
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
        let mut response = ratelimiting::GetThrottleImportStatusResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
    /// Run CancelThrottleImport after validating the tenant and durably recording the command.
    async fn run_cancel_throttle_import(
        &self,
        req: ratelimiting::CancelThrottleImportRequest,
    ) -> Result<ratelimiting::CancelThrottleImportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CancelThrottleImport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelThrottleImport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
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
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::QuoteQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CancelThrottleImportResponse::default();
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
            response
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .rate_limit_id = contribution;
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
    /// Run ValidateThrottleImport after validating the tenant and durably recording the command.
    async fn run_validate_throttle_import(
        &self,
        req: ratelimiting::ValidateThrottleImportRequest,
    ) -> Result<ratelimiting::ValidateThrottleImportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateThrottleImport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateThrottleImport", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::SimulateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SimulateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::SuggestRateLimitsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::suggest_rate_limits_request::Filter {
                created_after: reply_0
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_0.quota_bucket.clone().unwrap_or_default().labels.clone(),
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitQueryServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SuggestRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ValidateThrottleImportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
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
            response
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .rate_limit_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: String = req.throttle.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run ListThrottleImports after validating the tenant and durably recording the command.
    async fn run_list_throttle_imports(
        &self,
        req: ratelimiting::ListThrottleImportsRequest,
    ) -> Result<ratelimiting::ListThrottleImportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListThrottleImports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListThrottleImports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::AdvanceConfigChangeWorkflowRequest {
            config_change: Some(configuration::ConfigChange {
                attributes: req.filter.clone().unwrap_or_default().labels.clone(),
                occurred_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::advance_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::AdvanceConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ListThrottleImportsResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.config_change.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.throttles.is_empty() {
                response.throttles.push(Default::default());
            }
            response.throttles[0].occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run RetryThrottleImport after validating the tenant and durably recording the command.
    async fn run_retry_throttle_import(
        &self,
        req: ratelimiting::RetryThrottleImportRequest,
    ) -> Result<ratelimiting::RetryThrottleImportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryThrottleImport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryThrottleImport", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = giftcards::EstimateGiftCardRequest {
            context: req.context.clone(),
            gift_card: Some(giftcards::GiftCard {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = GiftCardCalculationServiceClient::new(channel);
            let input: tonic::Request<giftcards::EstimateGiftCardRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_gift_card(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::RetryThrottleImportResponse::default();
        let contribution: String = reply_0
            .gift_card
            .clone()
            .unwrap_or_default()
            .holder
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
        let contribution: String = req.throttle.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run MapThrottleImportColumns after validating the tenant and durably recording the command.
    async fn run_map_throttle_import_columns(
        &self,
        req: ratelimiting::MapThrottleImportColumnsRequest,
    ) -> Result<ratelimiting::MapThrottleImportColumnsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("MapThrottleImportColumns", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "MapThrottleImportColumns", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ListPendingConfigChangesRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_pending_config_changes_request::Filter {
                created_after: req.throttle.clone().unwrap_or_default().occurred_at.clone(),
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::ListPendingConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::CountQuotaBucketsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::count_quota_buckets_request::Filter {
                created_after: req.throttle.clone().unwrap_or_default().occurred_at.clone(),
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketQueryServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CountQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::PreviewQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::preview_quota_bucket_request::Details {
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
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
            let mut client = QuotaBucketValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PreviewQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::SimulateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    email: reply_2
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    preferred_channel: reply_2
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
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: reply_2
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    rate_limit_id: reply_2
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SimulateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::MapThrottleImportColumnsResponse::default();
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
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
}
acme_platform::endpoints! { ThrottleImportService, ThrottleImportEndpoint;
unary {
get_throttle_import_status (ratelimiting::GetThrottleImportStatusRequest) -> ratelimiting::GetThrottleImportStatusResponse => run_get_throttle_import_status;
cancel_throttle_import (ratelimiting::CancelThrottleImportRequest) -> ratelimiting::CancelThrottleImportResponse => run_cancel_throttle_import;
validate_throttle_import (ratelimiting::ValidateThrottleImportRequest) -> ratelimiting::ValidateThrottleImportResponse => run_validate_throttle_import;
list_throttle_imports (ratelimiting::ListThrottleImportsRequest) -> ratelimiting::ListThrottleImportsResponse => run_list_throttle_imports;
retry_throttle_import (ratelimiting::RetryThrottleImportRequest) -> ratelimiting::RetryThrottleImportResponse => run_retry_throttle_import;
map_throttle_import_columns (ratelimiting::MapThrottleImportColumnsRequest) -> ratelimiting::MapThrottleImportColumnsResponse => run_map_throttle_import_columns;
}
server {
}
client {
import_throttles (ratelimiting::ImportThrottlesRequest) -> ratelimiting::ImportThrottlesResponse => run_import_throttles;
}
bidi {
}
}
