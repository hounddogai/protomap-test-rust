use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::admissions::v1::admission_service_client::AdmissionServiceClient;
use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_query_service_client::AccessPolicyQueryServiceClient;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::authorization::v1::permission_check_validation_service_client::PermissionCheckValidationServiceClient;
use acme_proto::acme::authorization::v1::role_binding_query_service_client::RoleBindingQueryServiceClient;
use acme_proto::acme::authorization::v1::role_binding_validation_service_client::RoleBindingValidationServiceClient;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::boxoffice::v1::box_office_sale_service_client::BoxOfficeSaleServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_calculation_service_server::ConfigSetCalculationService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_service_client::UserServiceClient;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_validation_service_client::MetricSeriesValidationServiceClient;

/// Computes the figures of a config set, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct ConfigSetCalculationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    estimate_config_set_2_port:
        std::sync::Arc<acme_platform::ports::Adapter<audit::CountAuditEventsRequest, audit::CountAuditEventsResponse>>,
    explain_config_set_calculation_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<admissions::ListAdmissionsRequest, admissions::ListAdmissionsResponse>,
    >,
    explain_config_set_calculation_1_port: acme_platform::ports::Callback<
        authorization::SummarizeAccessPoliciesRequest,
        authorization::SummarizeAccessPoliciesResponse,
    >,
    get_config_set_rates_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            metrics::ExplainMetricSeriesDecisionRequest,
            metrics::ExplainMetricSeriesDecisionResponse,
        >,
    >,
    get_config_set_rates_1_port: acme_platform::ports::Callback<
        authorization::CheckRoleBindingEligibilityRequest,
        authorization::CheckRoleBindingEligibilityResponse,
    >,
    simulate_config_set_1_port: acme_platform::ports::Callback<
        authorization::BrowseRoleBindingsRequest,
        authorization::BrowseRoleBindingsResponse,
    >,
}
impl ConfigSetCalculationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<audit::CountAuditEventsRequest, audit::CountAuditEventsResponse> = {
            let client = AuditEventQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: audit::CountAuditEventsRequest| -> acme_platform::ports::RpcFuture<audit::CountAuditEventsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_audit_events(request).await?.into_inner())
})
})
        };
        let estimate_config_set_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            authorization::BrowseRoleBindingsRequest,
            authorization::BrowseRoleBindingsResponse,
        > = {
            let client = RoleBindingQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::BrowseRoleBindingsRequest| -> acme_platform::ports::RpcFuture<authorization::BrowseRoleBindingsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_role_bindings(request).await?.into_inner())
})
})
        };
        let simulate_config_set_1_port = callback;
        let callback: acme_platform::ports::Callback<
            admissions::ListAdmissionsRequest,
            admissions::ListAdmissionsResponse,
        > = {
            let client = AdmissionServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: admissions::ListAdmissionsRequest| -> acme_platform::ports::RpcFuture<admissions::ListAdmissionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_admissions(request).await?.into_inner())
})
})
        };
        let explain_config_set_calculation_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            authorization::SummarizeAccessPoliciesRequest,
            authorization::SummarizeAccessPoliciesResponse,
        > = {
            let client = AccessPolicyQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::SummarizeAccessPoliciesRequest| -> acme_platform::ports::RpcFuture<authorization::SummarizeAccessPoliciesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.summarize_access_policies(request).await?.into_inner())
})
})
        };
        let explain_config_set_calculation_1_port = callback;
        let callback: acme_platform::ports::Callback<
            metrics::ExplainMetricSeriesDecisionRequest,
            metrics::ExplainMetricSeriesDecisionResponse,
        > = {
            let client = MetricSeriesValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: metrics::ExplainMetricSeriesDecisionRequest| -> acme_platform::ports::RpcFuture<metrics::ExplainMetricSeriesDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_metric_series_decision(request).await?.into_inner())
})
})
        };
        let get_config_set_rates_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            authorization::CheckRoleBindingEligibilityRequest,
            authorization::CheckRoleBindingEligibilityResponse,
        > = {
            let client = RoleBindingValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::CheckRoleBindingEligibilityRequest| -> acme_platform::ports::RpcFuture<authorization::CheckRoleBindingEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_role_binding_eligibility(request).await?.into_inner())
})
})
        };
        let get_config_set_rates_1_port = callback;
        Self {
            channel,
            repository,
            estimate_config_set_2_port,
            explain_config_set_calculation_0_port,
            explain_config_set_calculation_1_port,
            get_config_set_rates_0_port,
            get_config_set_rates_1_port,
            simulate_config_set_1_port,
        }
    }
    /// Run CalculateConfigSet after validating the tenant and durably recording the command.
    async fn run_calculate_config_set(
        &self,
        req: configuration::CalculateConfigSetRequest,
    ) -> Result<configuration::CalculateConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let command_0 = CalculateConfigSetDispatch0::from(req.clone());
        let outgoing_0: authorization::LintRoleBindingsRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RoleBindingValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::LintRoleBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_role_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::BatchCreateAccessPoliciesRequest {
            context: req.context.clone(),
            chunk: Some(authorization::batch_create_access_policies_request::Chunk::Record(
                authorization::AccessPolicy {
                    audit: Some(common::AuditInfo {
                        created_at: reply_0
                            .role_binding
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        updated_by: req
                            .config_set
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
                        user_id: req
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
                        created_by: reply_0
                            .role_binding
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        updated_at: reply_0
                            .role_binding
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        ..Default::default()
                    }),
                    owner: Some(identity::UserRef {
                        display_name: reply_0
                            .role_binding
                            .clone()
                            .unwrap_or_default()
                            .counterparty
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        user_id: req
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
        let mut response = configuration::CalculateConfigSetResponse::default();
        let contribution: String = reply_0
            .role_binding
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .locality
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: String = req.config_set.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).parent_id = contribution;
        }
        Ok(response)
    }
    /// Run EstimateConfigSet after validating the tenant and durably recording the command.
    async fn run_estimate_config_set(
        &self,
        req: configuration::EstimateConfigSetRequest,
    ) -> Result<configuration::EstimateConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CompareFlagEvaluationVersionsRequest {
            context: req.context.clone(),
            details: Some(featureflags::compare_flag_evaluation_versions_request::Details {
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                attributes: req.config_set.clone().unwrap_or_default().attributes.clone(),
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: req
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::CompareFlagEvaluationVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_flag_evaluation_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::ExplainRoleBindingDecisionRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
                billing_address: Some(common::PostalAddress {
                    lines: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RoleBindingValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::ExplainRoleBindingDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_role_binding_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = audit::CountAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::count_audit_events_request::Filter {
                created_after: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: reply_1.role_binding.clone().unwrap_or_default().role_binding_id.clone(),
                region: reply_1
                    .role_binding
                    .clone()
                    .unwrap_or_default()
                    .billing_address
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.estimate_config_set_2_port.as_ref(), outgoing_2).await?;
        let mut response = configuration::EstimateConfigSetResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.flag_evaluation.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.config_set.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            if response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].children[0].children[0]
                .attributes
                .extend(contribution);
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run QuoteConfigSet after validating the tenant and durably recording the command.
    async fn run_quote_config_set(
        &self,
        req: configuration::QuoteConfigSetRequest,
    ) -> Result<configuration::QuoteConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = identity::UpdateUserRequest {
            context: req.context.clone(),
            user: Some(identity::User {
                primary_address: Some(common::PostalAddress {
                    country_code: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    locality: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    recipient: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    region: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
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
        let outgoing_0 = acme_platform::ports::RequestBuilder::<identity::UpdateUserRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::UpdateUserRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_user(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_1 = QuoteConfigSetDispatch1::from(req.clone());
        let outgoing_1: metrics::LintMetricSeriesesRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = MetricSeriesValidationServiceClient::new(channel);
            let input: tonic::Request<metrics::LintMetricSeriesesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_metric_serieses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::SendFeatureFlagTestNoticeRequest {
            context: req.context.clone(),
            details: Some(featureflags::send_feature_flag_test_notice_request::Details {
                recipient: Some(common::ContactInfo {
                    email: reply_0
                        .user
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: reply_0
                        .user
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: reply_0
                        .user
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: reply_0
                        .user
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
        let outgoing_2 =
            acme_platform::codec::through_any(&outgoing_2, "acme.featureflags.v1.SendFeatureFlagTestNoticeRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FeatureFlagNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::SendFeatureFlagTestNoticeRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.send_feature_flag_test_notice(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::SyncFeatureFlagsRequest {
            context: req.context.clone(),
            details: Some(featureflags::sync_feature_flags_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    updated_at: reply_1
                        .metric_series
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: reply_1
                        .metric_series
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
                    user_id: reply_1
                        .metric_series
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                action: Some(featureflags::feature_flag::Action::Route(
                    featureflags::FeatureFlagRouteAction {
                        weight: (match reply_2.feature_flag.clone().unwrap_or_default().action.clone() {
                            Some(featureflags::feature_flag::Action::Route(value)) => value,
                            _ => Default::default(),
                        })
                        .weight
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::SyncFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.sync_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::QuoteConfigSetResponse::default();
        let contribution: String = reply_0
            .user
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .user
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = (match reply_1.metric_series.clone().unwrap_or_default().locator.clone() {
            Some(metrics::metric_series::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .postal_code
        .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: String = reply_2
            .feature_flag
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_3
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: String = req.config_set.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run CompareConfigSetOptions after validating the tenant and durably recording the command.
    async fn run_compare_config_set_options(
        &self,
        req: configuration::CompareConfigSetOptionsRequest,
    ) -> Result<configuration::CompareConfigSetOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareConfigSetOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareConfigSetOptions", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CompareFlagEvaluationVersionsRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: req
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
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = FlagEvaluationHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::CompareFlagEvaluationVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_flag_evaluation_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = configuration::CompareConfigSetOptionsResponse::default();
        let contribution: String = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.config_set.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.config_set.get_or_insert_with(Default::default).manager = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateConfigSets after validating the tenant and durably recording the command.
    async fn run_recalculate_config_sets(
        &self,
        req: configuration::RecalculateConfigSetsRequest,
    ) -> Result<configuration::RecalculateConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecalculateConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CompareFlagEvaluationVersionsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                attributes: req.config_set.clone().unwrap_or_default().attributes.clone(),
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: req
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = FlagEvaluationHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::CompareFlagEvaluationVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_flag_evaluation_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = authorization::NormalizePermissionCheckRequest {
            context: req.context.clone(),
            details: Some(authorization::normalize_permission_check_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
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
            let mut client = PermissionCheckValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::NormalizePermissionCheckRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_permission_check(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = metrics::ValidateMetricSeriesRequest {
            context: req.context.clone(),
            metric_series: Some(metrics::MetricSeries {
                locator: Some(metrics::metric_series::Locator::Address(common::PostalAddress {
                    lines: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
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
                })),
                audit: Some(common::AuditInfo {
                    created_by: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = MetricSeriesValidationServiceClient::new(channel);
            let input: tonic::Request<metrics::ValidateMetricSeriesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_metric_series(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = authorization::ExportRoleBindingsRequest {
            context: req.context.clone(),
            filter: Some(authorization::export_role_bindings_request::Filter {
                created_after: reply_1.outcome.clone().unwrap_or_default().effective_at.clone(),
                owner_id: reply_1
                    .permission_check
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .access_policy_id
                    .clone(),
                region: (match reply_2.metric_series.clone().unwrap_or_default().locator.clone() {
                    Some(metrics::metric_series::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_3 = async move {
            let mut client = RoleBindingQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::ExportRoleBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
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
        let mut response = configuration::RecalculateConfigSetsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.permission_check.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.config_set.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            if response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].children[0].children[0]
                .attributes
                .extend(contribution);
        }
        let contribution: String = (match reply_2.metric_series.clone().unwrap_or_default().locator.clone() {
            Some(metrics::metric_series::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .country_code
        .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: String = reply_2
            .metric_series
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.config_set.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: i32 = req.config_set.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.config_set.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run SimulateConfigSet after validating the tenant and durably recording the command.
    async fn run_simulate_config_set(
        &self,
        req: configuration::SimulateConfigSetRequest,
    ) -> Result<configuration::SimulateConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = boxoffice::WatchBoxOfficeSalesRequest {
            context: req.context.clone(),
            filter: Some(boxoffice::watch_box_office_sales_request::Filter {
                created_after: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .location
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
                email: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .manager
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
            let mut client = BoxOfficeSaleServiceClient::new(channel);
            let input: tonic::Request<boxoffice::WatchBoxOfficeSalesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_box_office_sales(input).await?;
            let mut frames = response.into_inner();
            let mut last = boxoffice::WatchBoxOfficeSalesResponse::default();
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
        let outgoing_1 = authorization::BrowseRoleBindingsRequest {
            context: req.context.clone(),
            filter: Some(authorization::browse_role_bindings_request::Filter {
                created_after: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.config_set.clone().unwrap_or_default().attributes.clone(),
                owner_id: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .location
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = (self.simulate_config_set_1_port)(outgoing_1).await?;
        let mut response = configuration::SimulateConfigSetResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.config_set.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.config_set.get_or_insert_with(Default::default).manager = contribution;
        }
        Ok(response)
    }
    /// Run ExplainConfigSetCalculation after validating the tenant and durably recording the command.
    async fn run_explain_config_set_calculation(
        &self,
        req: configuration::ExplainConfigSetCalculationRequest,
    ) -> Result<configuration::ExplainConfigSetCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainConfigSetCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainConfigSetCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = admissions::ListAdmissionsRequest {
            context: req.context.clone(),
            filter: Some(admissions::list_admissions_request::Filter {
                created_after: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .location
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.explain_config_set_calculation_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = authorization::SummarizeAccessPoliciesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(authorization::summarize_access_policies_request::Details {
                filters: reply_0.result.clone().unwrap_or_default().details.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_1 = (self.explain_config_set_calculation_1_port)(outgoing_1).await?;
        let mut response = configuration::ExplainConfigSetCalculationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.config_set.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.config_set.get_or_insert_with(Default::default).manager = contribution;
        }
        Ok(response)
    }
    /// Run GetConfigSetRates after validating the tenant and durably recording the command.
    async fn run_get_config_set_rates(
        &self,
        req: configuration::GetConfigSetRatesRequest,
    ) -> Result<configuration::GetConfigSetRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetConfigSetRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetConfigSetRates", "running workflow");
        let outgoing_0 = metrics::ExplainMetricSeriesDecisionRequest {
            context: req.context.clone(),
            details: Some(metrics::explain_metric_series_decision_request::Details {
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
            metric_series: Some(metrics::MetricSeries {
                audit: Some(common::AuditInfo {
                    created_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
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
        let reply_0 =
            self.get_config_set_rates_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = authorization::CheckRoleBindingEligibilityRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .metric_series
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                counterparty: Some(authorization::AccessPolicyRef {
                    display_name: reply_0
                        .metric_series
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                lines: vec![authorization::RoleBindingLine {
                    quantity: reply_0
                        .metric_series
                        .clone()
                        .unwrap_or_default()
                        .components
                        .clone()
                        .get("primary")
                        .cloned()
                        .unwrap_or_default()
                        .quantity
                        .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.get_config_set_rates_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = configuration::GetConfigSetRatesResponse::default();
        let contribution: String = (match reply_0.metric_series.clone().unwrap_or_default().locator.clone() {
            Some(metrics::metric_series::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .recipient
        .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .recipient = contribution;
        }
        let contribution: String = reply_0
            .metric_series
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_1
            .role_binding
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0]
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: String = reply_1
            .role_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CalculateConfigSet for LintRoleBindings.
struct CalculateConfigSetDispatch0 {
    context: Option<common::RequestContext>,
    role_binding_billing_address_lines: Vec<String>,
    role_binding_audit_created_at: Option<prost_types::Timestamp>,
}
impl From<configuration::CalculateConfigSetRequest> for CalculateConfigSetDispatch0 {
    fn from(req: configuration::CalculateConfigSetRequest) -> Self {
        Self {
            context: req.context.clone(),
            role_binding_billing_address_lines: req
                .config_set
                .clone()
                .unwrap_or_default()
                .location
                .clone()
                .unwrap_or_default()
                .lines
                .clone(),
            role_binding_audit_created_at: req
                .config_set
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
        }
    }
}
impl From<CalculateConfigSetDispatch0> for authorization::LintRoleBindingsRequest {
    fn from(value: CalculateConfigSetDispatch0) -> Self {
        authorization::LintRoleBindingsRequest {
            context: value.context,
            role_binding: Some(authorization::RoleBinding {
                audit: Some(common::AuditInfo {
                    created_at: value.role_binding_audit_created_at,
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    lines: value.role_binding_billing_address_lines,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by QuoteConfigSet for LintMetricSerieses.
struct QuoteConfigSetDispatch1 {
    context: Option<common::RequestContext>,
    metric_series_address_locality: String,
    metric_series_audit_created_by: String,
    metric_series_owner_user_id: String,
}
impl From<configuration::QuoteConfigSetRequest> for QuoteConfigSetDispatch1 {
    fn from(req: configuration::QuoteConfigSetRequest) -> Self {
        Self {
            context: req.context.clone(),
            metric_series_address_locality: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .locality
                .clone(),
            metric_series_audit_created_by: req
                .config_set
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_by
                .clone(),
            metric_series_owner_user_id: req
                .config_set
                .clone()
                .unwrap_or_default()
                .manager
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
        }
    }
}
impl From<QuoteConfigSetDispatch1> for metrics::LintMetricSeriesesRequest {
    fn from(value: QuoteConfigSetDispatch1) -> Self {
        metrics::LintMetricSeriesesRequest {
            context: value.context,
            metric_series: Some(metrics::MetricSeries {
                locator: Some(metrics::metric_series::Locator::Address(common::PostalAddress {
                    locality: value.metric_series_address_locality,
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    created_by: value.metric_series_audit_created_by,
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    user_id: value.metric_series_owner_user_id,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigSetCalculationService for ConfigSetCalculationHandler {
    /// Calculates a config set.
    async fn calculate_config_set(
        &self,
        request: tonic::Request<configuration::CalculateConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::CalculateConfigSetResponse>, tonic::Status> {
        self.run_calculate_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a config set.
    async fn estimate_config_set(
        &self,
        request: tonic::Request<configuration::EstimateConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::EstimateConfigSetResponse>, tonic::Status> {
        self.run_estimate_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a config set.
    async fn quote_config_set(
        &self,
        request: tonic::Request<configuration::QuoteConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::QuoteConfigSetResponse>, tonic::Status> {
        self.run_quote_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a config set.
    async fn compare_config_set_options(
        &self,
        request: tonic::Request<configuration::CompareConfigSetOptionsRequest>,
    ) -> Result<tonic::Response<configuration::CompareConfigSetOptionsResponse>, tonic::Status> {
        self.run_compare_config_set_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a config set.
    async fn recalculate_config_sets(
        &self,
        request: tonic::Request<configuration::RecalculateConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::RecalculateConfigSetsResponse>, tonic::Status> {
        self.run_recalculate_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a config set.
    async fn simulate_config_set(
        &self,
        request: tonic::Request<configuration::SimulateConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::SimulateConfigSetResponse>, tonic::Status> {
        self.run_simulate_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a config set.
    async fn explain_config_set_calculation(
        &self,
        request: tonic::Request<configuration::ExplainConfigSetCalculationRequest>,
    ) -> Result<tonic::Response<configuration::ExplainConfigSetCalculationResponse>, tonic::Status> {
        self.run_explain_config_set_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config set.
    async fn get_config_set_rates(
        &self,
        request: tonic::Request<configuration::GetConfigSetRatesRequest>,
    ) -> Result<tonic::Response<configuration::GetConfigSetRatesResponse>, tonic::Status> {
        self.run_get_config_set_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
