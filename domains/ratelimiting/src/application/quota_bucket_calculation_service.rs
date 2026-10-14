use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::appointments::v1::appointment_service_client::AppointmentServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_query_service_client::AccessPolicyQueryServiceClient;
use acme_proto::acme::authorization::v1::role_binding_query_service_client::RoleBindingQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_server::QuotaBucketCalculationService;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;

/// Computes the figures of a quota bucket, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct QuotaBucketCalculationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    calculate_quota_bucket_0_port: acme_platform::ports::Callback<
        ratelimiting::ExplainRateLimitCalculationRequest,
        ratelimiting::ExplainRateLimitCalculationResponse,
    >,
    estimate_quota_bucket_3_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::ListFlagRolloutSubscriptionsRequest,
            featureflags::ListFlagRolloutSubscriptionsResponse,
        >,
    >,
    explain_quota_bucket_calculation_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::CompareRateLimitOptionsRequest,
            ratelimiting::CompareRateLimitOptionsResponse,
        >,
    >,
    get_quota_bucket_rates_0_port: acme_platform::ports::Callback<
        configuration::SimulateConfigSetRequest,
        configuration::SimulateConfigSetResponse,
    >,
}
impl QuotaBucketCalculationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::ExplainRateLimitCalculationRequest,
            ratelimiting::ExplainRateLimitCalculationResponse,
        > = {
            let client = RateLimitCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ExplainRateLimitCalculationRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ExplainRateLimitCalculationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_rate_limit_calculation(request).await?.into_inner())
})
})
        };
        let calculate_quota_bucket_0_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::ListFlagRolloutSubscriptionsRequest,
            featureflags::ListFlagRolloutSubscriptionsResponse,
        > = {
            let client = FlagRolloutNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ListFlagRolloutSubscriptionsRequest| -> acme_platform::ports::RpcFuture<featureflags::ListFlagRolloutSubscriptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_flag_rollout_subscriptions(request).await?.into_inner())
})
})
        };
        let estimate_quota_bucket_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::CompareRateLimitOptionsRequest,
            ratelimiting::CompareRateLimitOptionsResponse,
        > = {
            let client = RateLimitCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CompareRateLimitOptionsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CompareRateLimitOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_rate_limit_options(request).await?.into_inner())
})
})
        };
        let explain_quota_bucket_calculation_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::SimulateConfigSetRequest,
            configuration::SimulateConfigSetResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SimulateConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::SimulateConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_config_set(request).await?.into_inner())
})
})
        };
        let get_quota_bucket_rates_0_port = callback;
        Self {
            channel,
            repository,
            calculate_quota_bucket_0_port,
            estimate_quota_bucket_3_port,
            explain_quota_bucket_calculation_2_port,
            get_quota_bucket_rates_0_port,
        }
    }
    /// Run CalculateQuotaBucket after validating the tenant and durably recording the command.
    async fn run_calculate_quota_bucket(
        &self,
        req: ratelimiting::CalculateQuotaBucketRequest,
    ) -> Result<ratelimiting::CalculateQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ExplainRateLimitCalculationRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_at: req
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
        let port = acme_platform::ports::opaque(self.calculate_quota_bucket_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = appointments::ListAppointmentsRequest {
            context: req.context.clone(),
            filter: Some(appointments::list_appointments_request::Filter {
                labels: req.quota_bucket.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<appointments::ListAppointmentsRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AppointmentServiceClient::new(channel);
            let input: tonic::Request<appointments::ListAppointmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_appointments(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_2 = CalculateQuotaBucketDispatch2::from(req.clone());
        let outgoing_2: ratelimiting::QuoteRateLimitRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::QuoteRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CalculateQuotaBucketResponse::default();
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
        let contribution: String = reply_0.rate_limit.clone().unwrap_or_default().name.clone();
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
        let contribution: String = reply_2.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.quota_bucket.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).contact = contribution;
        }
        Ok(response)
    }
    /// Run EstimateQuotaBucket after validating the tenant and durably recording the command.
    async fn run_estimate_quota_bucket(
        &self,
        req: ratelimiting::EstimateQuotaBucketRequest,
    ) -> Result<ratelimiting::EstimateQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::EstimateRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::estimate_rate_limit_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .postal_code
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = configuration::SimulateConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::simulate_config_change_request::Details {
                destination: Some(common::PostalAddress {
                    lines: (req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
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
                    locality: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::SimulateConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::SimulateRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
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
                    etag: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
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
                action: Some(ratelimiting::rate_limit::Action::Route(
                    ratelimiting::RateLimitRouteAction {
                        target: (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                            Some(ratelimiting::rate_limit::Action::Route(value)) => value,
                            _ => Default::default(),
                        })
                        .target
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SimulateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::ListFlagRolloutSubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_rollout_subscriptions_request::Filter {
                labels: req.quota_bucket.clone().unwrap_or_default().labels.clone(),
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
        let reply_3 = acme_platform::ports::execute(self.estimate_quota_bucket_3_port.as_ref(), outgoing_3).await?;
        let mut response = ratelimiting::EstimateQuotaBucketResponse::default();
        let contribution: Option<common::Money> = reply_0.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: String = reply_0.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_1.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.rate_limit.clone().unwrap_or_default().rate_limit_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .rate_limit_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
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
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
    /// Run QuoteQuotaBucket after validating the tenant and durably recording the command.
    async fn run_quote_quota_bucket(
        &self,
        req: ratelimiting::QuoteQuotaBucketRequest,
    ) -> Result<ratelimiting::QuoteQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::WatchConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::watch_config_sets_request::Filter {
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
            subscriber: Some(common::ContactInfo {
                email: req
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
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::WatchConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_config_sets(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::WatchConfigSetsResponse::default();
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
        let command_1 = QuoteQuotaBucketDispatch1::from(req.clone());
        let outgoing_1: authorization::SummarizeRoleBindingsRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RoleBindingQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::SummarizeRoleBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_role_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::QuoteQuotaBucketResponse::default();
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
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
        let contribution: Option<ratelimiting::RateLimitRef> =
            req.quota_bucket.clone().unwrap_or_default().holder.clone();
        if contribution != <Option<ratelimiting::RateLimitRef>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).holder = contribution;
        }
        Ok(response)
    }
    /// Run CompareQuotaBucketOptions after validating the tenant and durably recording the command.
    async fn run_compare_quota_bucket_options(
        &self,
        req: ratelimiting::CompareQuotaBucketOptionsRequest,
    ) -> Result<ratelimiting::CompareQuotaBucketOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareQuotaBucketOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareQuotaBucketOptions", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CompareRateLimitOptionsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::compare_rate_limit_options_request::Details {
                destination: Some(common::PostalAddress {
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
                audit: Some(common::AuditInfo {
                    updated_at: req
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
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.ratelimiting.v1.CompareRateLimitOptionsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CompareRateLimitOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_rate_limit_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::ExportAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::export_access_policies_request::Filter {
                labels: req.quota_bucket.clone().unwrap_or_default().labels.clone(),
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
        let _reply_1 = async move {
            let mut client = AccessPolicyQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::ExportAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_access_policies(input).await?;
            let mut frames = response.into_inner();
            let mut last = authorization::ExportAccessPoliciesResponse::default();
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
        let mut response = ratelimiting::CompareQuotaBucketOptionsResponse::default();
        let contribution: String = reply_0
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
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
    /// Run RecalculateQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_recalculate_quota_buckets(
        &self,
        req: ratelimiting::RecalculateQuotaBucketsRequest,
    ) -> Result<ratelimiting::RecalculateQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecalculateQuotaBuckets", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SimulateEnvironmentRequest {
            context: req.context.clone(),
            details: Some(configuration::simulate_environment_request::Details {
                destination: Some(common::PostalAddress {
                    locality: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            environment: Some(configuration::Environment {
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
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.SimulateEnvironmentRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::SimulateEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = appointments::CloneAppointmentRequest {
            appointment: Some(appointments::Appointment {
                location: Some(common::PostalAddress {
                    recipient: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .recipient
                    .clone(),
                    ..Default::default()
                }),
                participants: vec![appointments::AppointmentParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
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
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<appointments::CloneAppointmentRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = AppointmentServiceClient::new(channel);
                let input: tonic::Request<appointments::CloneAppointmentRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.clone_appointment(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_1)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_1 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = ratelimiting::RecalculateQuotaBucketsResponse::default();
        let contribution: String = reply_0.environment.clone().unwrap_or_default().name.clone();
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
        let contribution: String = reply_1
            .appointment
            .clone()
            .unwrap_or_default()
            .organizer
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
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<ratelimiting::RateLimitRef> =
            req.quota_bucket.clone().unwrap_or_default().holder.clone();
        if contribution != <Option<ratelimiting::RateLimitRef>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).holder = contribution;
        }
        Ok(response)
    }
    /// Run SimulateQuotaBucket after validating the tenant and durably recording the command.
    async fn run_simulate_quota_bucket(
        &self,
        req: ratelimiting::SimulateQuotaBucketRequest,
    ) -> Result<ratelimiting::SimulateQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let jobs_0 = req
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
                    let outgoing_0 = ratelimiting::QuoteRateLimitRequest {
                        context: req.context.clone(),
                        details: Some(ratelimiting::quote_rate_limit_request::Details {
                            destination: Some(common::PostalAddress {
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
                            action: Some(ratelimiting::rate_limit::Action::Discount(
                                ratelimiting::RateLimitDiscountAction {
                                    amount: item.amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            name: item.name.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = RateLimitCalculationServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::QuoteRateLimitRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.quote_rate_limit(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = ratelimiting::SimulateQuotaBucketResponse::default();
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run ExplainQuotaBucketCalculation after validating the tenant and durably recording the command.
    async fn run_explain_quota_bucket_calculation(
        &self,
        req: ratelimiting::ExplainQuotaBucketCalculationRequest,
    ) -> Result<ratelimiting::ExplainQuotaBucketCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainQuotaBucketCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainQuotaBucketCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let jobs_0 = req
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
                    let outgoing_0 = ratelimiting::RecalculateRateLimitsRequest {
                        context: req.context.clone(),
                        rate_limit: Some(ratelimiting::RateLimit {
                            audit: Some(common::AuditInfo {
                                updated_at: req
                                    .quota_bucket
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
                            action: Some(ratelimiting::rate_limit::Action::Discount(
                                ratelimiting::RateLimitDiscountAction {
                                    amount: item.amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            name: item.name.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = RateLimitCalculationServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::RecalculateRateLimitsRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.recalculate_rate_limits(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = configuration::WatchConfigSetsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: reply_0
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
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
                    .details
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .phone
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
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::WatchConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_config_sets(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::WatchConfigSetsResponse::default();
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
        let outgoing_2 = ratelimiting::CompareRateLimitOptionsRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 =
            self.explain_quota_bucket_calculation_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let mut response = ratelimiting::ExplainQuotaBucketCalculationResponse::default();
        let contribution: String = reply_2
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
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_2.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.quota_bucket.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).contact = contribution;
        }
        Ok(response)
    }
    /// Run GetQuotaBucketRates after validating the tenant and durably recording the command.
    async fn run_get_quota_bucket_rates(
        &self,
        req: ratelimiting::GetQuotaBucketRatesRequest,
    ) -> Result<ratelimiting::GetQuotaBucketRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetQuotaBucketRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetQuotaBucketRates", "running workflow");
        let outgoing_0 = configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
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
                        parent_id: req.quota_bucket_id.clone(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.get_quota_bucket_rates_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = ratelimiting::GetQuotaBucketRatesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.config_set.clone().unwrap_or_default().name.clone();
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
        Ok(response)
    }
}
/// The domain command prepared by CalculateQuotaBucket for QuoteRateLimit.
struct CalculateQuotaBucketDispatch2 {
    context: Option<common::RequestContext>,
    rate_limit_owner_email: String,
    details_destination_recipient: String,
}
impl From<ratelimiting::CalculateQuotaBucketRequest> for CalculateQuotaBucketDispatch2 {
    fn from(req: ratelimiting::CalculateQuotaBucketRequest) -> Self {
        Self {
            context: req.context.clone(),
            rate_limit_owner_email: req
                .quota_bucket
                .clone()
                .unwrap_or_default()
                .contact
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            details_destination_recipient: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .recipient
                .clone(),
        }
    }
}
impl From<CalculateQuotaBucketDispatch2> for ratelimiting::QuoteRateLimitRequest {
    fn from(value: CalculateQuotaBucketDispatch2) -> Self {
        ratelimiting::QuoteRateLimitRequest {
            context: value.context,
            details: Some(ratelimiting::quote_rate_limit_request::Details {
                destination: Some(common::PostalAddress {
                    recipient: value.details_destination_recipient,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
                    email: value.rate_limit_owner_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by QuoteQuotaBucket for SummarizeRoleBindings.
struct QuoteQuotaBucketDispatch1 {
    context: Option<common::RequestContext>,
    reason: String,
    annotations: std::collections::BTreeMap<String, String>,
    details_filters: std::collections::BTreeMap<String, String>,
}
impl From<ratelimiting::QuoteQuotaBucketRequest> for QuoteQuotaBucketDispatch1 {
    fn from(req: ratelimiting::QuoteQuotaBucketRequest) -> Self {
        Self {
            context: req.context.clone(),
            reason: req.reason.clone(),
            annotations: req.annotations.clone(),
            details_filters: req.quota_bucket.clone().unwrap_or_default().labels.clone(),
        }
    }
}
impl From<QuoteQuotaBucketDispatch1> for authorization::SummarizeRoleBindingsRequest {
    fn from(value: QuoteQuotaBucketDispatch1) -> Self {
        authorization::SummarizeRoleBindingsRequest {
            annotations: value.annotations,
            context: value.context,
            details: Some(authorization::summarize_role_bindings_request::Details {
                filters: value.details_filters,
                ..Default::default()
            }),
            reason: value.reason,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl QuotaBucketCalculationService for QuotaBucketCalculationController {
    /// Calculates a quota bucket.
    async fn calculate_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::CalculateQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::CalculateQuotaBucketResponse>, tonic::Status> {
        self.run_calculate_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a quota bucket.
    async fn estimate_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::EstimateQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::EstimateQuotaBucketResponse>, tonic::Status> {
        self.run_estimate_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a quota bucket.
    async fn quote_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::QuoteQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::QuoteQuotaBucketResponse>, tonic::Status> {
        self.run_quote_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a quota bucket.
    async fn compare_quota_bucket_options(
        &self,
        request: tonic::Request<ratelimiting::CompareQuotaBucketOptionsRequest>,
    ) -> Result<tonic::Response<ratelimiting::CompareQuotaBucketOptionsResponse>, tonic::Status> {
        self.run_compare_quota_bucket_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a quota bucket.
    async fn recalculate_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::RecalculateQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::RecalculateQuotaBucketsResponse>, tonic::Status> {
        self.run_recalculate_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a quota bucket.
    async fn simulate_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::SimulateQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::SimulateQuotaBucketResponse>, tonic::Status> {
        self.run_simulate_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a quota bucket.
    async fn explain_quota_bucket_calculation(
        &self,
        request: tonic::Request<ratelimiting::ExplainQuotaBucketCalculationRequest>,
    ) -> Result<tonic::Response<ratelimiting::ExplainQuotaBucketCalculationResponse>, tonic::Status> {
        self.run_explain_quota_bucket_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a quota bucket.
    async fn get_quota_bucket_rates(
        &self,
        request: tonic::Request<ratelimiting::GetQuotaBucketRatesRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetQuotaBucketRatesResponse>, tonic::Status> {
        self.run_get_quota_bucket_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
