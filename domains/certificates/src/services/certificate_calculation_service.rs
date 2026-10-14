use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::authorization::v1::role_binding_history_service_client::RoleBindingHistoryServiceClient;
use acme_proto::acme::authorization::v1::role_binding_notification_service_client::RoleBindingNotificationServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_calculation_service_server::CertificateCalculationService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_query_service_client::ConfigSetQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_client::FeatureFlagHistoryServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_query_service_client::LedgerAccountQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::shows::v1 as shows;
use acme_proto::acme::shows::v1::show_service_client::ShowServiceClient;

/// Computes the figures of a certificate, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct CertificateCalculationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    estimate_certificate_2_port: acme_platform::ports::Callback<
        configuration::CheckEnvironmentEligibilityRequest,
        configuration::CheckEnvironmentEligibilityResponse,
    >,
    simulate_certificate_0_port:
        std::sync::Arc<acme_platform::ports::Adapter<shows::SearchShowsRequest, shows::SearchShowsResponse>>,
    simulate_certificate_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::CalculateConfigSetRequest,
                configuration::CalculateConfigSetResponse,
            >,
    >,
    simulate_certificate_2_port: acme_platform::ports::Callback<
        configuration::ExplainConfigSetDecisionRequest,
        configuration::ExplainConfigSetDecisionResponse,
    >,
}
impl CertificateCalculationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::CheckEnvironmentEligibilityRequest,
            configuration::CheckEnvironmentEligibilityResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CheckEnvironmentEligibilityRequest| -> acme_platform::ports::RpcFuture<configuration::CheckEnvironmentEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_environment_eligibility(request).await?.into_inner())
})
})
        };
        let estimate_certificate_2_port = callback;
        let callback: acme_platform::ports::Callback<shows::SearchShowsRequest, shows::SearchShowsResponse> = {
            let client = ShowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: shows::SearchShowsRequest| -> acme_platform::ports::RpcFuture<shows::SearchShowsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.search_shows(request).await?.into_inner())
})
})
        };
        let simulate_certificate_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::CalculateConfigSetRequest,
            configuration::CalculateConfigSetResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CalculateConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::CalculateConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_config_set(request).await?.into_inner())
})
})
        };
        let simulate_certificate_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ExplainConfigSetDecisionRequest,
            configuration::ExplainConfigSetDecisionResponse,
        > = {
            let client = ConfigSetValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ExplainConfigSetDecisionRequest| -> acme_platform::ports::RpcFuture<configuration::ExplainConfigSetDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_config_set_decision(request).await?.into_inner())
})
})
        };
        let simulate_certificate_2_port = callback;
        Self {
            channel,
            repository,
            estimate_certificate_2_port,
            simulate_certificate_0_port,
            simulate_certificate_1_port,
            simulate_certificate_2_port,
        }
    }
    /// Run CalculateCertificate after validating the tenant and durably recording the command.
    async fn run_calculate_certificate(
        &self,
        req: certificates::CalculateCertificateRequest,
    ) -> Result<certificates::CalculateCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::RecalculateRateLimitsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::recalculate_rate_limits_request::Details {
                destination: Some(common::PostalAddress {
                    lines: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .certificate
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ratelimiting::RecalculateRateLimitsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = RateLimitCalculationServiceClient::new(channel);
                let input: tonic::Request<ratelimiting::RecalculateRateLimitsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.recalculate_rate_limits(input).await?;
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
        let outgoing_1 = authorization::BatchCreateAccessPoliciesRequest {
            context: req.context.clone(),
            chunk: Some(authorization::batch_create_access_policies_request::Chunk::Record(
                authorization::AccessPolicy {
                    action: Some(authorization::access_policy::Action::Discount(
                        authorization::AccessPolicyDiscountAction {
                            amount: (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                                Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                                _ => Default::default(),
                            })
                            .amount
                            .clone(),
                            percent: (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                                Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                                _ => Default::default(),
                            })
                            .percent
                            .clone(),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessPolicyServiceClient::new(channel);
            let input: tonic::Request<authorization::BatchCreateAccessPoliciesRequest> =
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
            let response = client.batch_create_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        created_by: req
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        etag: req
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .etag
                            .clone(),
                        ..Default::default()
                    }),
                    owner: Some(identity::UserRef {
                        display_name: req
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .counterparty
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        ..Default::default()
                    }),
                    action: Some(featureflags::feature_flag::Action::Route(
                        featureflags::FeatureFlagRouteAction {
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_feature_flag_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFeatureFlagChangesResponse::default();
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
        let mut response = certificates::CalculateCertificateResponse::default();
        let contribution: Option<common::Money> = reply_0.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.certificate.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: Option<common::Money> =
            (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                _ => Default::default(),
            })
            .amount
            .clone();
        if contribution != <Option<common::Money>>::default() {
            if response
                .certificate
                .get_or_insert_with(Default::default)
                .lines
                .is_empty()
            {
                response
                    .certificate
                    .get_or_insert_with(Default::default)
                    .lines
                    .push(Default::default());
            }
            response.certificate.get_or_insert_with(Default::default).lines[0]
                .tax
                .get_or_insert_with(Default::default)
                .amount = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<common::PostalAddress> =
            req.certificate.clone().unwrap_or_default().billing_address.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address = contribution;
        }
        Ok(response)
    }
    /// Run EstimateCertificate after validating the tenant and durably recording the command.
    async fn run_estimate_certificate(
        &self,
        req: certificates::EstimateCertificateRequest,
    ) -> Result<certificates::EstimateCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let jobs_0 = req
            .certificate
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = authorization::RecordRoleBindingVersionRequest {
                        context: req.context.clone(),
                        role_binding: Some(authorization::RoleBinding {
                            lines: vec![authorization::RoleBindingLine {
                                amount: item.amount.clone(),
                                tax: Some(authorization::RoleBindingLineTax {
                                    amount: item.tax.clone().unwrap_or_default().amount.clone(),
                                    jurisdiction: item.tax.clone().unwrap_or_default().jurisdiction.clone(),
                                    ..Default::default()
                                }),
                                unit_price: item.unit_price.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = RoleBindingHistoryServiceClient::new(channel);
                        let input: tonic::Request<authorization::RecordRoleBindingVersionRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.record_role_binding_version(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = featureflags::StreamFeatureFlagNoticesRequest {
            context: req.context.clone(),
            filter: Some(featureflags::stream_feature_flag_notices_request::Filter {
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
            let mut client = FeatureFlagNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagNoticesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_feature_flag_notices(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFeatureFlagNoticesResponse::default();
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
        let outgoing_2 = configuration::CheckEnvironmentEligibilityRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    country_code: reply_0
                        .role_binding
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
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
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = (self.estimate_certificate_2_port)(outgoing_2).await?;
        let mut response = certificates::EstimateCertificateResponse::default();
        let contribution: String = (match reply_2.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .region
        .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = (match reply_2.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .country_code
        .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<common::Money> = req.certificate.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.certificate.get_or_insert_with(Default::default).total = contribution;
        }
        Ok(response)
    }
    /// Run QuoteCertificate after validating the tenant and durably recording the command.
    async fn run_quote_certificate(
        &self,
        req: certificates::QuoteCertificateRequest,
    ) -> Result<certificates::QuoteCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let command_0 = QuoteCertificateDispatch0::from(req.clone());
        let outgoing_0: configuration::RecalculateEnvironmentsRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateEnvironmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_environments(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .certificate
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = ratelimiting::NormalizeRateLimitRequest {
                        context: req.context.clone(),
                        rate_limit: Some(ratelimiting::RateLimit {
                            description: item.description.clone(),
                            action: Some(ratelimiting::rate_limit::Action::Discount(
                                ratelimiting::RateLimitDiscountAction {
                                    amount: item.tax.clone().unwrap_or_default().amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = RateLimitValidationServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::NormalizeRateLimitRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.normalize_rate_limit(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = certificates::QuoteCertificateResponse::default();
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run CompareCertificateOptions after validating the tenant and durably recording the command.
    async fn run_compare_certificate_options(
        &self,
        req: certificates::CompareCertificateOptionsRequest,
    ) -> Result<certificates::CompareCertificateOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareCertificateOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareCertificateOptions", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::RecalculateConfigChangesRequest {
            context: req.context.clone(),
            details: Some(configuration::recalculate_config_changes_request::Details {
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
        let reply_0 = async move {
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::CompareCertificateOptionsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_change.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            if response
                .certificate
                .get_or_insert_with(Default::default)
                .lines
                .is_empty()
            {
                response
                    .certificate
                    .get_or_insert_with(Default::default)
                    .lines
                    .push(Default::default());
            }
            response.certificate.get_or_insert_with(Default::default).lines[0]
                .attributes
                .extend(contribution);
        }
        let contribution: String = reply_0.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<common::PostalAddress> =
            req.certificate.clone().unwrap_or_default().billing_address.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateCertificates after validating the tenant and durably recording the command.
    async fn run_recalculate_certificates(
        &self,
        req: certificates::RecalculateCertificatesRequest,
    ) -> Result<certificates::RecalculateCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecalculateCertificates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SummarizeConfigSetsRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::SummarizeConfigSetsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigSetQueryServiceClient::new(channel);
                let input: tonic::Request<configuration::SummarizeConfigSetsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.summarize_config_sets(input).await?;
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
        let outgoing_1 = ledger::SummarizeLedgerAccountsRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(ledger::summarize_ledger_accounts_request::Details {
                period: reply_0.period.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = LedgerAccountQueryServiceClient::new(channel);
            let input: tonic::Request<ledger::SummarizeLedgerAccountsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_ledger_accounts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::ListFeatureFlagVersionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_versions_request::Filter {
                created_after: req.certificate.clone().unwrap_or_default().issued_at.clone(),
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
        let reply_2 = async move {
            let mut client = FeatureFlagHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::ListFeatureFlagVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_feature_flag_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = shows::CloneShowRequest {
            context: req.context.clone(),
            show: Some(shows::Show {
                location: Some(common::PostalAddress {
                    country_code: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
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
        let reply_3 = async move {
            let mut client = ShowServiceClient::new(channel);
            let input: tonic::Request<shows::CloneShowRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_show(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::RecalculateCertificatesResponse::default();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_3.show.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<common::Money> = req.certificate.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        Ok(response)
    }
    /// Run SimulateCertificate after validating the tenant and durably recording the command.
    async fn run_simulate_certificate(
        &self,
        req: certificates::SimulateCertificateRequest,
    ) -> Result<certificates::SimulateCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().number.clone(), "recorded audit field");
        let outgoing_0 = shows::SearchShowsRequest {
            context: req.context.clone(),
            filter: Some(shows::search_shows_request::Filter {
                created_after: req
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.certificate.clone().unwrap_or_default().metadata.clone(),
                owner_id: req.certificate.clone().unwrap_or_default().certificate_id.clone(),
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
        let reply_0 = acme_platform::ports::execute(self.simulate_certificate_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::CalculateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            lines: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .lines
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.simulate_certificate_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::ExplainConfigSetDecisionRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            user_id: reply_1
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .manager
                                .clone()
                                .unwrap_or_default()
                                .user_id
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
        let reply_2 = (self.simulate_certificate_2_port)(outgoing_2).await?;
        let mut response = certificates::SimulateCertificateResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.certificate.clone().unwrap_or_default().issued_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).issued_at = contribution;
        }
        Ok(response)
    }
    /// Run ExplainCertificateCalculation after validating the tenant and durably recording the command.
    async fn run_explain_certificate_calculation(
        &self,
        req: certificates::ExplainCertificateCalculationRequest,
    ) -> Result<certificates::ExplainCertificateCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainCertificateCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainCertificateCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let jobs_0 = req
            .certificate
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = authorization::CompareAccessPolicyVersionsRequest {
                        access_policy: Some(authorization::AccessPolicy {
                            audit: Some(common::AuditInfo {
                                created_by: req
                                    .certificate
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .created_by
                                    .clone(),
                                updated_by: req
                                    .certificate
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .updated_by
                                    .clone(),
                                ..Default::default()
                            }),
                            description: item.description.clone(),
                            action: Some(authorization::access_policy::Action::Discount(
                                authorization::AccessPolicyDiscountAction {
                                    amount: item.amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AccessPolicyHistoryServiceClient::new(channel);
                        let input: tonic::Request<authorization::CompareAccessPolicyVersionsRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.compare_access_policy_versions(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = featureflags::RevertFeatureFlagRequest {
            context: req.context.clone(),
            details: Some(featureflags::revert_feature_flag_request::Details {
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                action: Some(featureflags::feature_flag::Action::Discount(
                    featureflags::FeatureFlagDiscountAction {
                        amount: (match reply_0.access_policy.clone().unwrap_or_default().action.clone() {
                            Some(authorization::access_policy::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .amount
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FeatureFlagHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::RevertFeatureFlagRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.revert_feature_flag(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ExplainCertificateCalculationResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: i32 = req.certificate.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.certificate.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run GetCertificateRates after validating the tenant and durably recording the command.
    async fn run_get_certificate_rates(
        &self,
        req: certificates::GetCertificateRatesRequest,
    ) -> Result<certificates::GetCertificateRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCertificateRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCertificateRates", "running workflow");
        let outgoing_0 = authorization::SubscribeRoleBindingUpdatesRequest {
            context: req.context.clone(),
            details: Some(authorization::subscribe_role_binding_updates_request::Details {
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
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<authorization::SubscribeRoleBindingUpdatesRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RoleBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::SubscribeRoleBindingUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_role_binding_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::GetCertificateRatesResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .role_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_0.role_binding.clone().unwrap_or_default().issued_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).issued_at = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by QuoteCertificate for RecalculateEnvironments.
struct QuoteCertificateDispatch0 {
    context: Option<common::RequestContext>,
    environment_address_region: String,
}
impl From<certificates::QuoteCertificateRequest> for QuoteCertificateDispatch0 {
    fn from(req: certificates::QuoteCertificateRequest) -> Self {
        Self {
            context: req.context.clone(),
            environment_address_region: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .region
                .clone(),
        }
    }
}
impl From<QuoteCertificateDispatch0> for configuration::RecalculateEnvironmentsRequest {
    fn from(value: QuoteCertificateDispatch0) -> Self {
        configuration::RecalculateEnvironmentsRequest {
            context: value.context,
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    region: value.environment_address_region,
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CertificateCalculationService for CertificateCalculationApi {
    /// Calculates a certificate.
    async fn calculate_certificate(
        &self,
        request: tonic::Request<certificates::CalculateCertificateRequest>,
    ) -> Result<tonic::Response<certificates::CalculateCertificateResponse>, tonic::Status> {
        self.run_calculate_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a certificate.
    async fn estimate_certificate(
        &self,
        request: tonic::Request<certificates::EstimateCertificateRequest>,
    ) -> Result<tonic::Response<certificates::EstimateCertificateResponse>, tonic::Status> {
        self.run_estimate_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a certificate.
    async fn quote_certificate(
        &self,
        request: tonic::Request<certificates::QuoteCertificateRequest>,
    ) -> Result<tonic::Response<certificates::QuoteCertificateResponse>, tonic::Status> {
        self.run_quote_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a certificate.
    async fn compare_certificate_options(
        &self,
        request: tonic::Request<certificates::CompareCertificateOptionsRequest>,
    ) -> Result<tonic::Response<certificates::CompareCertificateOptionsResponse>, tonic::Status> {
        self.run_compare_certificate_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a certificate.
    async fn recalculate_certificates(
        &self,
        request: tonic::Request<certificates::RecalculateCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::RecalculateCertificatesResponse>, tonic::Status> {
        self.run_recalculate_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a certificate.
    async fn simulate_certificate(
        &self,
        request: tonic::Request<certificates::SimulateCertificateRequest>,
    ) -> Result<tonic::Response<certificates::SimulateCertificateResponse>, tonic::Status> {
        self.run_simulate_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a certificate.
    async fn explain_certificate_calculation(
        &self,
        request: tonic::Request<certificates::ExplainCertificateCalculationRequest>,
    ) -> Result<tonic::Response<certificates::ExplainCertificateCalculationResponse>, tonic::Status> {
        self.run_explain_certificate_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a certificate.
    async fn get_certificate_rates(
        &self,
        request: tonic::Request<certificates::GetCertificateRatesRequest>,
    ) -> Result<tonic::Response<certificates::GetCertificateRatesResponse>, tonic::Status> {
        self.run_get_certificate_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
