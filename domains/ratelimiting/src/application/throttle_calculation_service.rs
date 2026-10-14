use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_query_service_client::DocumentQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_validation_service_client::UserValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_server::ThrottleCalculationService;

/// Computes the figures of a throttle, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct ThrottleCalculationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    estimate_throttle_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<ratelimiting::QuoteQuotaBucketRequest, ratelimiting::QuoteQuotaBucketResponse>,
    >,
    quote_throttle_0_port:
        acme_platform::ports::Callback<ratelimiting::QuoteQuotaBucketRequest, ratelimiting::QuoteQuotaBucketResponse>,
    recalculate_throttles_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::QuoteRateLimitRequest, ratelimiting::QuoteRateLimitResponse>,
    >,
    recalculate_throttles_1_port:
        acme_platform::ports::Callback<identity::ValidateUserRequest, identity::ValidateUserResponse>,
}
impl ThrottleCalculationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::QuoteQuotaBucketRequest,
            ratelimiting::QuoteQuotaBucketResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::QuoteQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::QuoteQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_quota_bucket(request).await?.into_inner())
})
})
        };
        let estimate_throttle_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::QuoteQuotaBucketRequest,
            ratelimiting::QuoteQuotaBucketResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::QuoteQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::QuoteQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_quota_bucket(request).await?.into_inner())
})
})
        };
        let quote_throttle_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::QuoteRateLimitRequest,
            ratelimiting::QuoteRateLimitResponse,
        > = {
            let client = RateLimitCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::QuoteRateLimitRequest| -> acme_platform::ports::RpcFuture<ratelimiting::QuoteRateLimitResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_rate_limit(request).await?.into_inner())
})
})
        };
        let recalculate_throttles_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<identity::ValidateUserRequest, identity::ValidateUserResponse> = {
            let client = UserValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: identity::ValidateUserRequest| -> acme_platform::ports::RpcFuture<identity::ValidateUserResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.validate_user(request).await?.into_inner())
})
})
        };
        let recalculate_throttles_1_port = callback;
        Self {
            channel,
            repository,
            estimate_throttle_2_port,
            quote_throttle_0_port,
            recalculate_throttles_0_port,
            recalculate_throttles_1_port,
        }
    }
    /// Run CalculateThrottle after validating the tenant and durably recording the command.
    async fn run_calculate_throttle(
        &self,
        req: ratelimiting::CalculateThrottleRequest,
    ) -> Result<ratelimiting::CalculateThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let command_0 = CalculateThrottleDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::RecalculateQuotaBucketsRequest = command_0.into();
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
        let outgoing_1 = configuration::CreateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            postal_code: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .postal_code
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
            acme_platform::codec::through_any(&outgoing_1, "acme.configuration.v1.CreateConfigSetRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::CreateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::SummarizeConfigChangesRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(configuration::summarize_config_changes_request::Details {
                filters: (reply_1.result.clone().unwrap_or_default().details.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigChangeQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::SummarizeConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CalculateThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .config_set
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
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
    /// Run EstimateThrottle after validating the tenant and durably recording the command.
    async fn run_estimate_throttle(
        &self,
        req: ratelimiting::EstimateThrottleRequest,
    ) -> Result<ratelimiting::EstimateThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SummarizeConfigChangesRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::SummarizeConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .throttle
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = ratelimiting::SimulateQuotaBucketRequest {
                        context: req.context.clone(),
                        details: Some(ratelimiting::simulate_quota_bucket_request::Details {
                            destination: Some(common::PostalAddress {
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
                        }),
                        quota_bucket: Some(ratelimiting::QuotaBucket {
                            audit: Some(common::AuditInfo {
                                created_at: item.observed_at.clone(),
                                ..Default::default()
                            }),
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
                    async move {
                        let mut client = QuotaBucketCalculationServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::SimulateQuotaBucketRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.simulate_quota_bucket(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                funding: Some(ratelimiting::quota_bucket::Funding::Bank(
                    ratelimiting::QuotaBucketBankFunding {
                        account_holder: (match reply_1.quota_bucket.clone().unwrap_or_default().funding.clone() {
                            Some(ratelimiting::quota_bucket::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .account_holder
                        .clone(),
                        iban: (match reply_1.quota_bucket.clone().unwrap_or_default().funding.clone() {
                            Some(ratelimiting::quota_bucket::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .iban
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.estimate_throttle_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = ratelimiting::ExplainQuotaBucketCalculationRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_quota_bucket_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    phone: reply_2
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
                        bic: (match reply_2.quota_bucket.clone().unwrap_or_default().funding.clone() {
                            Some(ratelimiting::quota_bucket::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .bic
                        .clone(),
                        ..Default::default()
                    },
                )),
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainQuotaBucketCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_quota_bucket_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::EstimateThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
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
    /// Run QuoteThrottle after validating the tenant and durably recording the command.
    async fn run_quote_throttle(
        &self,
        req: ratelimiting::QuoteThrottleRequest,
    ) -> Result<ratelimiting::QuoteThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::quote_quota_bucket_request::Details {
                destination: Some(common::PostalAddress {
                    lines: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.quote_throttle_0_port)(outgoing_0).await?;
        let mut response = ratelimiting::QuoteThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: String = req.throttle.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run CompareThrottleOptions after validating the tenant and durably recording the command.
    async fn run_compare_throttle_options(
        &self,
        req: ratelimiting::CompareThrottleOptionsRequest,
    ) -> Result<ratelimiting::CompareThrottleOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CompareThrottleOptions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareThrottleOptions", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::QuoteRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::quote_rate_limit_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    recipient: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::QuoteRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = ratelimiting::CompareThrottleOptionsResponse::default();
        let contribution: String = reply_0.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: String = req.throttle.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateThrottles after validating the tenant and durably recording the command.
    async fn run_recalculate_throttles(
        &self,
        req: ratelimiting::RecalculateThrottlesRequest,
    ) -> Result<ratelimiting::RecalculateThrottlesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecalculateThrottles", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateThrottles", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::QuoteRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::quote_rate_limit_request::Details {
                destination: Some(common::PostalAddress {
                    lines: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    postal_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    recipient: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.recalculate_throttles_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = identity::ValidateUserRequest {
            context: req.context.clone(),
            user: Some(identity::User {
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
                    ..Default::default()
                }),
                primary_address: Some(common::PostalAddress {
                    locality: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .locality
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
                relationship_manager: Some(identity::UserRef {
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
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.recalculate_throttles_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = ratelimiting::RecalculateThrottlesResponse::default();
        let contribution: Option<common::Money> =
            (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                _ => Default::default(),
            })
            .amount
            .clone();
        if contribution != <Option<common::Money>>::default() {
            if response.breakdown.get_or_insert_with(Default::default).lines.is_empty() {
                response
                    .breakdown
                    .get_or_insert_with(Default::default)
                    .lines
                    .push(Default::default());
            }
            response.breakdown.get_or_insert_with(Default::default).lines[0].amount = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run SimulateThrottle after validating the tenant and durably recording the command.
    async fn run_simulate_throttle(
        &self,
        req: ratelimiting::SimulateThrottleRequest,
    ) -> Result<ratelimiting::SimulateThrottleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateThrottle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateThrottle", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::RecalculateQuotaBucketsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::recalculate_quota_buckets_request::Details {
                destination: Some(common::PostalAddress {
                    lines: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<ratelimiting::RecalculateQuotaBucketsRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
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
        let outgoing_1 = configuration::SearchConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::search_config_sets_request::Filter {
                created_after: reply_0
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: (reply_0.quota_bucket.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::SearchConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::LintRateLimitsRequest {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::codec::through_any(&outgoing_2, "acme.ratelimiting.v1.LintRateLimitsRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::LintRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = documents::SummarizeDocumentsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(documents::summarize_documents_request::Details {
                filters: req.throttle.clone().unwrap_or_default().attributes.clone(),
                period: reply_2.rate_limit.clone().unwrap_or_default().effective.clone(),
                ..Default::default()
            }),
            reason: (match reply_2.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<documents::SummarizeDocumentsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = DocumentQueryServiceClient::new(channel);
                let input: tonic::Request<documents::SummarizeDocumentsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.summarize_documents(input).await?;
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
        let mut response = ratelimiting::SimulateThrottleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Option<common::Money> =
            (match reply_2.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                _ => Default::default(),
            })
            .amount
            .clone();
        if contribution != <Option<common::Money>>::default() {
            if response.breakdown.get_or_insert_with(Default::default).lines.is_empty() {
                response
                    .breakdown
                    .get_or_insert_with(Default::default)
                    .lines
                    .push(Default::default());
            }
            response.breakdown.get_or_insert_with(Default::default).lines[0].amount = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
    /// Run GetThrottleRates after validating the tenant and durably recording the command.
    async fn run_get_throttle_rates(
        &self,
        req: ratelimiting::GetThrottleRatesRequest,
    ) -> Result<ratelimiting::GetThrottleRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetThrottleRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetThrottleRates", "running workflow");
        let mut response = ratelimiting::GetThrottleRatesResponse::default();
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CalculateThrottle for RecalculateQuotaBuckets.
struct CalculateThrottleDispatch0 {
    context: Option<common::RequestContext>,
    details_destination_region: String,
    quota_bucket_holder_display_name: String,
}
impl From<ratelimiting::CalculateThrottleRequest> for CalculateThrottleDispatch0 {
    fn from(req: ratelimiting::CalculateThrottleRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_destination_region: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .region
                .clone(),
            quota_bucket_holder_display_name: req
                .throttle
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .display_name
                .clone(),
        }
    }
}
impl From<CalculateThrottleDispatch0> for ratelimiting::RecalculateQuotaBucketsRequest {
    fn from(value: CalculateThrottleDispatch0) -> Self {
        ratelimiting::RecalculateQuotaBucketsRequest {
            context: value.context,
            details: Some(ratelimiting::recalculate_quota_buckets_request::Details {
                destination: Some(common::PostalAddress {
                    region: value.details_destination_region,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: value.quota_bucket_holder_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ThrottleCalculationService for ThrottleCalculationApi {
    /// Calculates a throttle.
    async fn calculate_throttle(
        &self,
        request: tonic::Request<ratelimiting::CalculateThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::CalculateThrottleResponse>, tonic::Status> {
        self.run_calculate_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a throttle.
    async fn estimate_throttle(
        &self,
        request: tonic::Request<ratelimiting::EstimateThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::EstimateThrottleResponse>, tonic::Status> {
        self.run_estimate_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a throttle.
    async fn quote_throttle(
        &self,
        request: tonic::Request<ratelimiting::QuoteThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::QuoteThrottleResponse>, tonic::Status> {
        self.run_quote_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a throttle.
    async fn compare_throttle_options(
        &self,
        request: tonic::Request<ratelimiting::CompareThrottleOptionsRequest>,
    ) -> Result<tonic::Response<ratelimiting::CompareThrottleOptionsResponse>, tonic::Status> {
        self.run_compare_throttle_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a throttle.
    async fn recalculate_throttles(
        &self,
        request: tonic::Request<ratelimiting::RecalculateThrottlesRequest>,
    ) -> Result<tonic::Response<ratelimiting::RecalculateThrottlesResponse>, tonic::Status> {
        self.run_recalculate_throttles(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a throttle.
    async fn simulate_throttle(
        &self,
        request: tonic::Request<ratelimiting::SimulateThrottleRequest>,
    ) -> Result<tonic::Response<ratelimiting::SimulateThrottleResponse>, tonic::Status> {
        self.run_simulate_throttle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a throttle.
    async fn get_throttle_rates(
        &self,
        request: tonic::Request<ratelimiting::GetThrottleRatesRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetThrottleRatesResponse>, tonic::Status> {
        self.run_get_throttle_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
