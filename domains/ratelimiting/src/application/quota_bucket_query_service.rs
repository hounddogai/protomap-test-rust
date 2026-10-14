use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_service_client::DocumentServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_server::QuotaBucketQueryService;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_client::RateLimitQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Answers read-only questions about quota buckets, for screens and reports.
#[derive(Clone)]
pub struct QuotaBucketQueryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    count_quota_buckets_0_port: acme_platform::ports::Callback<
        featureflags::ListFeatureFlagSubscriptionsRequest,
        featureflags::ListFeatureFlagSubscriptionsResponse,
    >,
    summarize_quota_buckets_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<ratelimiting::EstimateRateLimitRequest, ratelimiting::EstimateRateLimitResponse>,
    >,
    summarize_quota_buckets_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                featureflags::RecordFlagRolloutVersionRequest,
                featureflags::RecordFlagRolloutVersionResponse,
            >,
    >,
}
impl QuotaBucketQueryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::ListFeatureFlagSubscriptionsRequest,
            featureflags::ListFeatureFlagSubscriptionsResponse,
        > = {
            let client = FeatureFlagNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ListFeatureFlagSubscriptionsRequest| -> acme_platform::ports::RpcFuture<featureflags::ListFeatureFlagSubscriptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_feature_flag_subscriptions(request).await?.into_inner())
})
})
        };
        let count_quota_buckets_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::EstimateRateLimitRequest,
            ratelimiting::EstimateRateLimitResponse,
        > = {
            let client = RateLimitCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::EstimateRateLimitRequest| -> acme_platform::ports::RpcFuture<ratelimiting::EstimateRateLimitResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_rate_limit(request).await?.into_inner())
})
})
        };
        let summarize_quota_buckets_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::RecordFlagRolloutVersionRequest,
            featureflags::RecordFlagRolloutVersionResponse,
        > = {
            let client = FlagRolloutHistoryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::RecordFlagRolloutVersionRequest| -> acme_platform::ports::RpcFuture<featureflags::RecordFlagRolloutVersionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.record_flag_rollout_version(request).await?.into_inner())
})
})
        };
        let summarize_quota_buckets_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            count_quota_buckets_0_port,
            summarize_quota_buckets_1_port,
            summarize_quota_buckets_2_port,
        }
    }
    /// Run LookupQuotaBucket after validating the tenant and durably recording the command.
    async fn run_lookup_quota_bucket(
        &self,
        req: ratelimiting::LookupQuotaBucketRequest,
    ) -> Result<ratelimiting::LookupQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupQuotaBucket", "running workflow");
        let command_0 = LookupQuotaBucketDispatch0::from(req.clone());
        let outgoing_0: featureflags::UnsubscribeFeatureFlagUpdatesRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::UnsubscribeFeatureFlagUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.unsubscribe_feature_flag_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = documents::BatchCreateDocumentsRequest {
            context: req.context.clone(),
            chunk: Some(documents::batch_create_documents_request::Chunk::Record(
                documents::Document {
                    audit: Some(common::AuditInfo {
                        created_at: reply_0
                            .feature_flag
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        ..Default::default()
                    }),
                    lines: vec![documents::DocumentLine {
                        tax: Some(documents::DocumentLineTax {
                            amount: (match reply_0.feature_flag.clone().unwrap_or_default().action.clone() {
                                Some(featureflags::feature_flag::Action::Discount(value)) => value,
                                _ => Default::default(),
                            })
                            .amount
                            .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = DocumentServiceClient::new(channel);
            let input: tonic::Request<documents::BatchCreateDocumentsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_documents(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::LookupQuotaBucketResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_browse_quota_buckets(
        &self,
        req: ratelimiting::BrowseQuotaBucketsRequest,
    ) -> Result<ratelimiting::BrowseQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseQuotaBuckets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = authorization::BatchCreateAccessPoliciesRequest {
            context: req.context.clone(),
            chunk: Some(authorization::batch_create_access_policies_request::Chunk::Record(
                authorization::AccessPolicy {
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
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = AccessPolicyServiceClient::new(channel);
            let input: tonic::Request<authorization::BatchCreateAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let response = ratelimiting::BrowseQuotaBucketsResponse::default();
        Ok(response)
    }
    /// Run FindQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_find_quota_buckets(
        &self,
        req: ratelimiting::FindQuotaBucketsRequest,
    ) -> Result<ratelimiting::FindQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindQuotaBuckets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = venues::SuggestVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::suggest_venues_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_token: req.page.clone().unwrap_or_default().page_token.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<venues::SuggestVenuesRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
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
        let mut response = ratelimiting::FindQuotaBucketsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CountQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_count_quota_buckets(
        &self,
        req: ratelimiting::CountQuotaBucketsRequest,
    ) -> Result<ratelimiting::CountQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountQuotaBuckets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFeatureFlagSubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_subscriptions_request::Filter {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_token: req.page.clone().unwrap_or_default().page_token.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.count_quota_buckets_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = ratelimiting::CountQuotaBucketsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ResolveQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_resolve_quota_buckets(
        &self,
        req: ratelimiting::ResolveQuotaBucketsRequest,
    ) -> Result<ratelimiting::ResolveQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveQuotaBuckets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveQuotaBuckets", "running workflow");
        let outgoing_0 = ratelimiting::PreviewThrottleRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::preview_throttle_request::Details {
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
                source: req
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
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PreviewThrottleRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ResolveQuotaBucketsResponse::default();
        let contribution: String = reply_0
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response.quota_buckets.is_empty() {
                response.quota_buckets.push(Default::default());
            }
            response.quota_buckets[0]
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_summarize_quota_buckets(
        &self,
        req: ratelimiting::SummarizeQuotaBucketsRequest,
    ) -> Result<ratelimiting::SummarizeQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeQuotaBuckets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::GetQuotaBucketRatesRequest {
            context: req.context.clone(),
            quota_bucket_id: req.quota_bucket_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::GetQuotaBucketRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_quota_bucket_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::EstimateRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
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
                    display_name: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
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
        let reply_1 = acme_platform::ports::execute(self.summarize_quota_buckets_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = featureflags::RecordFlagRolloutVersionRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                audit: Some(common::AuditInfo {
                    created_by: reply_1
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: reply_0
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
                participants: vec![featureflags::FlagRolloutParticipant {
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.summarize_quota_buckets_2_port.as_ref(), outgoing_2).await?;
        let mut response = ratelimiting::SummarizeQuotaBucketsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run PeekQuotaBucket after validating the tenant and durably recording the command.
    async fn run_peek_quota_bucket(
        &self,
        req: ratelimiting::PeekQuotaBucketRequest,
    ) -> Result<ratelimiting::PeekQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekQuotaBucket", "running workflow");
        let outgoing_0 = ratelimiting::BrowseRateLimitsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::browse_rate_limits_request::Filter {
                owner_id: req.quota_bucket_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ratelimiting::BrowseRateLimitsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = RateLimitQueryServiceClient::new(channel);
                let input: tonic::Request<ratelimiting::BrowseRateLimitsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.browse_rate_limits(input).await?;
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
        let outgoing_1 = ratelimiting::SuggestRateLimitsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::suggest_rate_limits_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = RateLimitQueryServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SuggestRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = ratelimiting::PeekQuotaBucketResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
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
        Ok(response)
    }
}
/// The domain command prepared by LookupQuotaBucket for UnsubscribeFeatureFlagUpdates.
struct LookupQuotaBucketDispatch0 {
    context: Option<common::RequestContext>,
    details_recipient_email: String,
    annotations: std::collections::BTreeMap<String, String>,
}
impl From<ratelimiting::LookupQuotaBucketRequest> for LookupQuotaBucketDispatch0 {
    fn from(req: ratelimiting::LookupQuotaBucketRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_recipient_email: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
        }
    }
}
impl From<LookupQuotaBucketDispatch0> for featureflags::UnsubscribeFeatureFlagUpdatesRequest {
    fn from(value: LookupQuotaBucketDispatch0) -> Self {
        featureflags::UnsubscribeFeatureFlagUpdatesRequest {
            annotations: value.annotations,
            context: value.context,
            details: Some(featureflags::unsubscribe_feature_flag_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    email: value.details_recipient_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl QuotaBucketQueryService for QuotaBucketQueryEndpoint {
    /// Returns a quota bucket.
    async fn lookup_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::LookupQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::LookupQuotaBucketResponse>, tonic::Status> {
        self.run_lookup_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists quota buckets a page at a time.
    async fn browse_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::BrowseQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::BrowseQuotaBucketsResponse>, tonic::Status> {
        self.run_browse_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists quota buckets a page at a time.
    async fn find_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::FindQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::FindQuotaBucketsResponse>, tonic::Status> {
        self.run_find_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts quota buckets by status.
    async fn count_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::CountQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::CountQuotaBucketsResponse>, tonic::Status> {
        self.run_count_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several quota buckets at once.
    async fn resolve_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::ResolveQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ResolveQuotaBucketsResponse>, tonic::Status> {
        self.run_resolve_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Summarizes a quota bucket.
    async fn summarize_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::SummarizeQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::SummarizeQuotaBucketsResponse>, tonic::Status> {
        self.run_summarize_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a quota bucket.
    async fn peek_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::PeekQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::PeekQuotaBucketResponse>, tonic::Status> {
        self.run_peek_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
