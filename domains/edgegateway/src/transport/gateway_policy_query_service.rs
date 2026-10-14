use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::authorization::v1::role_binding_validation_service_client::RoleBindingValidationServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::csr_approval_service_client::CsrApprovalServiceClient;
use acme_proto::acme::certificates::v1::csr_workflow_service_client::CsrWorkflowServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_integration_service_client::ApiKeyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_notification_service_client::ApiKeyNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_client::ApiKeyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_integration_service_client::GatewayPolicyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_query_service_server::GatewayPolicyQueryService;
use acme_proto::acme::edgegateway::v1::route_binding_notification_service_client::RouteBindingNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_service_client::RouteBindingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_export_service_client::FeatureFlagExportServiceClient;
use acme_proto::acme::identity::v1 as identity;

/// Answers read-only questions about gateway policies, for screens and reports.
#[derive(Clone)]
pub struct GatewayPolicyQueryController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
}
impl GatewayPolicyQueryController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run LookupGatewayPolicy after validating the tenant and durably recording the command.
    async fn run_lookup_gateway_policy(
        &self,
        req: edgegateway::LookupGatewayPolicyRequest,
    ) -> Result<edgegateway::LookupGatewayPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupGatewayPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupGatewayPolicy", "running workflow");
        let outgoing_0 = authorization::RevertAccessPolicyRequest {
            access_policy: Some(authorization::AccessPolicy {
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
            context: req.context.clone(),
            details: Some(authorization::revert_access_policy_request::Details {
                author_email: req
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
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessPolicyHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::RevertAccessPolicyRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.revert_access_policy(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::IngestApiKeyEventsRequest {
            context: req.context.clone(),
            chunk: Some(edgegateway::ingest_api_key_events_request::Chunk::Record(
                edgegateway::ApiKey {
                    audit: Some(common::AuditInfo {
                        created_by: reply_0
                            .access_policy
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        updated_at: reply_0
                            .access_policy
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        ..Default::default()
                    }),
                    limits: vec![edgegateway::ApiKeyLimit {
                        amount: (match reply_0.access_policy.clone().unwrap_or_default().action.clone() {
                            Some(authorization::access_policy::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .amount
                        .clone(),
                        name: reply_0.access_policy.clone().unwrap_or_default().name.clone(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ApiKeyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::IngestApiKeyEventsRequest> =
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
            let response = client.ingest_api_key_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::RelayGatewayPolicyMessagesRequest {
            frame: Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(
                edgegateway::GatewayPolicy {
                    audit: Some(common::AuditInfo {
                        updated_by: reply_0
                            .access_policy
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_by
                            .clone(),
                        ..Default::default()
                    }),
                    action: Some(edgegateway::gateway_policy::Action::Block(
                        edgegateway::GatewayPolicyBlockAction {
                            reason: (match reply_0.access_policy.clone().unwrap_or_default().action.clone() {
                                Some(authorization::access_policy::Action::Block(value)) => value,
                                _ => Default::default(),
                            })
                            .reason
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
            let mut client = GatewayPolicyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::RelayGatewayPolicyMessagesRequest> =
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
            let response = client.relay_gateway_policy_messages(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::RelayGatewayPolicyMessagesResponse::default();
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
        let mut response = edgegateway::LookupGatewayPolicyResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseGatewayPolicies after validating the tenant and durably recording the command.
    async fn run_browse_gateway_policies(
        &self,
        req: edgegateway::BrowseGatewayPoliciesRequest,
    ) -> Result<edgegateway::BrowseGatewayPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseGatewayPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseGatewayPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ShareFeatureFlagExportRequest {
            context: req.context.clone(),
            details: Some(featureflags::share_feature_flag_export_request::Details {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::ShareFeatureFlagExportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_feature_flag_export(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::BrowseGatewayPoliciesResponse::default();
        let contribution: String = reply_0.feature_flag.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response.gateway_policies.is_empty() {
                response.gateway_policies.push(Default::default());
            }
            response.gateway_policies[0].name = contribution;
        }
        Ok(response)
    }
    /// Run FindGatewayPolicies after validating the tenant and durably recording the command.
    async fn run_find_gateway_policies(
        &self,
        req: edgegateway::FindGatewayPoliciesRequest,
    ) -> Result<edgegateway::FindGatewayPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindGatewayPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindGatewayPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::SignalCsrWorkflowRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
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
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                requester: Some(common::ContactInfo {
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
            details: Some(certificates::signal_csr_workflow_request::Details {
                parameters: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CsrWorkflowServiceClient::new(channel);
            let input: tonic::Request<certificates::SignalCsrWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_csr_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::FindGatewayPoliciesResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run CountGatewayPolicies after validating the tenant and durably recording the command.
    async fn run_count_gateway_policies(
        &self,
        req: edgegateway::CountGatewayPoliciesRequest,
    ) -> Result<edgegateway::CountGatewayPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountGatewayPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountGatewayPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::SubmitCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
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
                })),
                requester: Some(common::ContactInfo {
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
            let mut client = CsrApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::SubmitCsrRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.submit_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::AdvanceApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .assignee
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    preferred_channel: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(edgegateway::GatewayPolicyRef {
                    display_name: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .display_name
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(edgegateway::advance_api_key_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
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
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AdvanceApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::CountGatewayPoliciesResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeGatewayPolicies after validating the tenant and durably recording the command.
    async fn run_summarize_gateway_policies(
        &self,
        req: edgegateway::SummarizeGatewayPoliciesRequest,
    ) -> Result<edgegateway::SummarizeGatewayPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SummarizeGatewayPolicies", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeGatewayPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::SubscribeRouteBindingUpdatesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SubscribeRouteBindingUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_route_binding_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::NotifyApiKeyWatchersRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::notify_api_key_watchers_request::Details {
                recipient: Some(common::ContactInfo {
                    email: reply_0
                        .route_binding
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ApiKeyNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::NotifyApiKeyWatchersRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.notify_api_key_watchers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = authorization::CheckRoleBindingEligibilityRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
                settlement: Some(authorization::role_binding::Settlement::Card(
                    authorization::RoleBindingCardSettlement {
                        holder_name: (match reply_1.api_key.clone().unwrap_or_default().funding.clone() {
                            Some(edgegateway::api_key::Funding::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .holder_name
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
            let mut client = RoleBindingValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::CheckRoleBindingEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_role_binding_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::SummarizeGatewayPoliciesResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestGatewayPolicies after validating the tenant and durably recording the command.
    async fn run_suggest_gateway_policies(
        &self,
        req: edgegateway::SuggestGatewayPoliciesRequest,
    ) -> Result<edgegateway::SuggestGatewayPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestGatewayPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestGatewayPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::SearchRouteBindingsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::search_route_bindings_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_token: req.page.clone().unwrap_or_default().page_token.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RouteBindingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SearchRouteBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_route_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::SuggestGatewayPoliciesResponse::default();
        let contribution: i64 = reply_0.page_info.clone().unwrap_or_default().total_size.clone();
        if contribution != <i64>::default() {
            response.page_info.get_or_insert_with(Default::default).total_size = contribution;
        }
        Ok(response)
    }
    /// Run PeekGatewayPolicy after validating the tenant and durably recording the command.
    async fn run_peek_gateway_policy(
        &self,
        req: edgegateway::PeekGatewayPolicyRequest,
    ) -> Result<edgegateway::PeekGatewayPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekGatewayPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekGatewayPolicy", "running workflow");
        let mut response = edgegateway::PeekGatewayPolicyResponse::default();
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { GatewayPolicyQueryService, GatewayPolicyQueryController;
unary {
lookup_gateway_policy (edgegateway::LookupGatewayPolicyRequest) -> edgegateway::LookupGatewayPolicyResponse => run_lookup_gateway_policy;
browse_gateway_policies (edgegateway::BrowseGatewayPoliciesRequest) -> edgegateway::BrowseGatewayPoliciesResponse => run_browse_gateway_policies;
find_gateway_policies (edgegateway::FindGatewayPoliciesRequest) -> edgegateway::FindGatewayPoliciesResponse => run_find_gateway_policies;
count_gateway_policies (edgegateway::CountGatewayPoliciesRequest) -> edgegateway::CountGatewayPoliciesResponse => run_count_gateway_policies;
summarize_gateway_policies (edgegateway::SummarizeGatewayPoliciesRequest) -> edgegateway::SummarizeGatewayPoliciesResponse => run_summarize_gateway_policies;
suggest_gateway_policies (edgegateway::SuggestGatewayPoliciesRequest) -> edgegateway::SuggestGatewayPoliciesResponse => run_suggest_gateway_policies;
peek_gateway_policy (edgegateway::PeekGatewayPolicyRequest) -> edgegateway::PeekGatewayPolicyResponse => run_peek_gateway_policy;
}
server {
}
client {
}
bidi {
}
}
