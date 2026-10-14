use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::community::v1::forum_service_client::ForumServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_integration_service_client::ApiKeyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_notification_service_server::ApiKeyNotificationService;
use acme_proto::acme::edgegateway::v1::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_client::RouteBindingIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_notification_service_client::RouteBindingNotificationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_client::FlagRolloutQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_service_client::UserServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_service_client::SearchIndexServiceClient;
use acme_proto::acme::subscriptions::v1 as subscriptions;
use acme_proto::acme::subscriptions::v1::subscription_service_client::SubscriptionServiceClient;

/// Tells the people who follow an api key what changed.
#[derive(Clone)]
pub struct ApiKeyNotificationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    notify_api_key_watchers_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            authorization::ListAccessPolicyVersionsRequest,
            authorization::ListAccessPolicyVersionsResponse,
        >,
    >,
    notify_api_key_watchers_2_port:
        std::sync::Arc<dyn acme_platform::ports::RpcPort<identity::CreateUserRequest, identity::CreateUserResponse>>,
    notify_api_key_watchers_3_port: acme_platform::ports::Callback<
        featureflags::SummarizeFlagRolloutsRequest,
        featureflags::SummarizeFlagRolloutsResponse,
    >,
    send_api_key_test_notice_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::NotifyFlagEvaluationWatchersRequest,
            featureflags::NotifyFlagEvaluationWatchersResponse,
        >,
    >,
    send_api_key_test_notice_1_port: acme_platform::ports::Callback<
        edgegateway::ListGatewayPolicyReportsRequest,
        edgegateway::ListGatewayPolicyReportsResponse,
    >,
    subscribe_api_key_updates_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::BrowseFlagRolloutsRequest,
            featureflags::BrowseFlagRolloutsResponse,
        >,
    >,
    subscribe_api_key_updates_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<search::SearchSearchIndexesRequest, search::SearchSearchIndexesResponse>,
    >,
    unsubscribe_api_key_updates_0_port: acme_platform::ports::Callback<
        certificates::ResumeCertificateAuthorityWorkflowRequest,
        certificates::ResumeCertificateAuthorityWorkflowResponse,
    >,
}
impl ApiKeyNotificationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::BrowseFlagRolloutsRequest,
            featureflags::BrowseFlagRolloutsResponse,
        > = {
            let client = FlagRolloutQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::BrowseFlagRolloutsRequest| -> acme_platform::ports::RpcFuture<featureflags::BrowseFlagRolloutsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_flag_rollouts(request).await?.into_inner())
})
})
        };
        let subscribe_api_key_updates_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            search::SearchSearchIndexesRequest,
            search::SearchSearchIndexesResponse,
        > = {
            let client = SearchIndexServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: search::SearchSearchIndexesRequest| -> acme_platform::ports::RpcFuture<search::SearchSearchIndexesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.search_search_indexes(request).await?.into_inner())
})
})
        };
        let subscribe_api_key_updates_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::ResumeCertificateAuthorityWorkflowRequest,
            certificates::ResumeCertificateAuthorityWorkflowResponse,
        > = {
            let client = CertificateAuthorityWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::ResumeCertificateAuthorityWorkflowRequest| -> acme_platform::ports::RpcFuture<certificates::ResumeCertificateAuthorityWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resume_certificate_authority_workflow(request).await?.into_inner())
})
})
        };
        let unsubscribe_api_key_updates_0_port = callback;
        let callback: acme_platform::ports::Callback<
            authorization::ListAccessPolicyVersionsRequest,
            authorization::ListAccessPolicyVersionsResponse,
        > = {
            let client = AccessPolicyHistoryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::ListAccessPolicyVersionsRequest| -> acme_platform::ports::RpcFuture<authorization::ListAccessPolicyVersionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_access_policy_versions(request).await?.into_inner())
})
})
        };
        let notify_api_key_watchers_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<identity::CreateUserRequest, identity::CreateUserResponse> = {
            let client = UserServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: identity::CreateUserRequest| -> acme_platform::ports::RpcFuture<identity::CreateUserResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.create_user(request).await?.into_inner())
})
})
        };
        let notify_api_key_watchers_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::SummarizeFlagRolloutsRequest,
            featureflags::SummarizeFlagRolloutsResponse,
        > = {
            let client = FlagRolloutQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::SummarizeFlagRolloutsRequest| -> acme_platform::ports::RpcFuture<featureflags::SummarizeFlagRolloutsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.summarize_flag_rollouts(request).await?.into_inner())
})
})
        };
        let notify_api_key_watchers_3_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::NotifyFlagEvaluationWatchersRequest,
            featureflags::NotifyFlagEvaluationWatchersResponse,
        > = {
            let client = FlagEvaluationNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::NotifyFlagEvaluationWatchersRequest| -> acme_platform::ports::RpcFuture<featureflags::NotifyFlagEvaluationWatchersResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.notify_flag_evaluation_watchers(request).await?.into_inner())
})
})
        };
        let send_api_key_test_notice_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            edgegateway::ListGatewayPolicyReportsRequest,
            edgegateway::ListGatewayPolicyReportsResponse,
        > = {
            let client = GatewayPolicyReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::ListGatewayPolicyReportsRequest| -> acme_platform::ports::RpcFuture<edgegateway::ListGatewayPolicyReportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_gateway_policy_reports(request).await?.into_inner())
})
})
        };
        let send_api_key_test_notice_1_port = callback;
        Self {
            channel,
            repository,
            notify_api_key_watchers_1_port,
            notify_api_key_watchers_2_port,
            notify_api_key_watchers_3_port,
            send_api_key_test_notice_0_port,
            send_api_key_test_notice_1_port,
            subscribe_api_key_updates_1_port,
            subscribe_api_key_updates_2_port,
            unsubscribe_api_key_updates_0_port,
        }
    }
    /// Run SubscribeApiKeyUpdates after validating the tenant and durably recording the command.
    async fn run_subscribe_api_key_updates(
        &self,
        req: edgegateway::SubscribeApiKeyUpdatesRequest,
    ) -> Result<edgegateway::SubscribeApiKeyUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubscribeApiKeyUpdates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubscribeApiKeyUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::RelayRouteBindingMessagesRequest {
            frame: Some(edgegateway::relay_route_binding_messages_request::Frame::Change(
                edgegateway::RouteBinding {
                    action: Some(edgegateway::route_binding::Action::Block(
                        edgegateway::RouteBindingBlockAction {
                            reason: req.reason.clone(),
                            ..Default::default()
                        },
                    )),
                    owner: Some(identity::UserRef {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .recipient
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::RelayRouteBindingMessagesRequest> =
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
            let response = client.relay_route_binding_messages(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::RelayRouteBindingMessagesResponse::default();
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
        let outgoing_1 = featureflags::BrowseFlagRolloutsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::browse_flag_rollouts_request::Filter {
                owner_id: req.api_key_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.subscribe_api_key_updates_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = search::SearchSearchIndexesRequest {
            context: req.context.clone(),
            filter: Some(search::search_search_indexes_request::Filter {
                labels: req.annotations.clone(),
                owner_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.subscribe_api_key_updates_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = subscriptions::BatchCreateSubscriptionsRequest {
            context: req.context.clone(),
            chunk: Some(subscriptions::batch_create_subscriptions_request::Chunk::Header(
                subscriptions::batch_create_subscriptions_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .recipient
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .recipient
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        preferred_channel: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .recipient
                            .clone()
                            .unwrap_or_default()
                            .preferred_channel
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = SubscriptionServiceClient::new(channel);
            let input: tonic::Request<subscriptions::BatchCreateSubscriptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_subscriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = ratelimiting::StreamThrottleChangesRequest {
            frame: Some(ratelimiting::stream_throttle_changes_request::Frame::Change(
                ratelimiting::Throttle {
                    attributes: reply_1.result.clone().unwrap_or_default().details.clone(),
                    correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                    measurements: vec![ratelimiting::ThrottleMeasurement {
                        observed_at: req.context.clone().unwrap_or_default().deadline.clone(),
                        ..Default::default()
                    }],
                    source: req.api_key_id.clone(),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamThrottleChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_4);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_throttle_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamThrottleChangesResponse::default();
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
        let mut response = edgegateway::SubscribeApiKeyUpdatesResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: Option<common::PersonName> = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .name
            .clone();
        if contribution != <Option<common::PersonName>>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .name = contribution;
        }
        Ok(response)
    }
    /// Run UnsubscribeApiKeyUpdates after validating the tenant and durably recording the command.
    async fn run_unsubscribe_api_key_updates(
        &self,
        req: edgegateway::UnsubscribeApiKeyUpdatesRequest,
    ) -> Result<edgegateway::UnsubscribeApiKeyUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UnsubscribeApiKeyUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnsubscribeApiKeyUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = certificates::ResumeCertificateAuthorityWorkflowRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    manager: Some(identity::UserRef {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .recipient
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
        let port = acme_platform::ports::opaque(self.unsubscribe_api_key_updates_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = edgegateway::SendRouteBindingTestNoticeRequest {
            context: req.context.clone(),
            details: Some(edgegateway::send_route_binding_test_notice_request::Details {
                recipient: Some(common::ContactInfo {
                    email: reply_0
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                template_id: req.details.clone().unwrap_or_default().template_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<edgegateway::SendRouteBindingTestNoticeRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SendRouteBindingTestNoticeRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.send_route_binding_test_notice(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::UnsubscribeApiKeyUpdatesResponse::default();
        let contribution: String = reply_0.certificate_authority.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response.api_key.get_or_insert_with(Default::default).limits.is_empty() {
                response
                    .api_key
                    .get_or_insert_with(Default::default)
                    .limits
                    .push(Default::default());
            }
            response.api_key.get_or_insert_with(Default::default).limits[0].name = contribution;
        }
        let contribution: Option<common::Money> =
            (match reply_1.route_binding.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::route_binding::Action::Discount(value)) => value,
                _ => Default::default(),
            })
            .amount
            .clone();
        if contribution != <Option<common::Money>>::default() {
            if response.api_key.get_or_insert_with(Default::default).limits.is_empty() {
                response
                    .api_key
                    .get_or_insert_with(Default::default)
                    .limits
                    .push(Default::default());
            }
            response.api_key.get_or_insert_with(Default::default).limits[0].amount = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run NotifyApiKeyWatchers after validating the tenant and durably recording the command.
    async fn run_notify_api_key_watchers(
        &self,
        req: edgegateway::NotifyApiKeyWatchersRequest,
    ) -> Result<edgegateway::NotifyApiKeyWatchersResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NotifyApiKeyWatchers", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NotifyApiKeyWatchers", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::DisconnectApiKeySourceRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ApiKeyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::DisconnectApiKeySourceRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.disconnect_api_key_source(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::ListAccessPolicyVersionsRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_access_policy_versions_request::Filter {
                created_after: reply_0
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_0.api_key.clone().unwrap_or_default().labels.clone(),
                owner_id: req.details.clone().unwrap_or_default().template_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.notify_api_key_watchers_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = identity::CreateUserRequest {
            context: req.context.clone(),
            user: Some(identity::User {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                relationship_manager: Some(identity::UserRef {
                    email: reply_0
                        .api_key
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
        let reply_2 = acme_platform::ports::execute(self.notify_api_key_watchers_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = featureflags::SummarizeFlagRolloutsRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_3 = (self.notify_api_key_watchers_3_port)(outgoing_3).await?;
        let mut response = edgegateway::NotifyApiKeyWatchersResponse::default();
        let contribution: String = reply_0
            .api_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_0
            .api_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .gateway_policy_id
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
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
        let contribution: String = reply_2
            .user
            .clone()
            .unwrap_or_default()
            .relationship_manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .preferred_channel
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .preferred_channel = contribution;
        }
        Ok(response)
    }
    /// Run ListApiKeySubscriptions after validating the tenant and durably recording the command.
    async fn run_list_api_key_subscriptions(
        &self,
        req: edgegateway::ListApiKeySubscriptionsRequest,
    ) -> Result<edgegateway::ListApiKeySubscriptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListApiKeySubscriptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListApiKeySubscriptions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = edgegateway::ListApiKeySubscriptionsResponse::default();
        Ok(response)
    }
    /// Run SendApiKeyTestNotice after validating the tenant and durably recording the command.
    async fn run_send_api_key_test_notice(
        &self,
        req: edgegateway::SendApiKeyTestNoticeRequest,
    ) -> Result<edgegateway::SendApiKeyTestNoticeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SendApiKeyTestNotice", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SendApiKeyTestNotice", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = featureflags::NotifyFlagEvaluationWatchersRequest {
            context: req.context.clone(),
            details: Some(featureflags::notify_flag_evaluation_watchers_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
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
        let reply_0 =
            self.send_api_key_test_notice_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = edgegateway::ListGatewayPolicyReportsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_gateway_policy_reports_request::Filter {
                created_after: reply_0.flag_evaluation.clone().unwrap_or_default().occurred_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.send_api_key_test_notice_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let outgoing_2 = community::SearchForumsRequest {
            context: req.context.clone(),
            filter: Some(community::search_forums_request::Filter {
                created_after: reply_0.flag_evaluation.clone().unwrap_or_default().occurred_at.clone(),
                labels: reply_0.flag_evaluation.clone().unwrap_or_default().attributes.clone(),
                owner_id: reply_0.outcome.clone().unwrap_or_default().reference_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<community::SearchForumsRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ForumServiceClient::new(channel);
            let input: tonic::Request<community::SearchForumsRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_forums(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_3 = SendApiKeyTestNoticeDispatch3::from(req.clone());
        command_3.details_message_id = reply_1.result.clone().unwrap_or_default().message.clone();
        let outgoing_3: edgegateway::DisconnectApiKeySourceRequest = command_3.into();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ApiKeyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::DisconnectApiKeySourceRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.disconnect_api_key_source(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::SendApiKeyTestNoticeResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = reply_3
            .api_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: Option<common::PersonName> = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .name
            .clone();
        if contribution != <Option<common::PersonName>>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .name = contribution;
        }
        Ok(response)
    }
    /// Run GetApiKeyPreferences after validating the tenant and durably recording the command.
    async fn run_get_api_key_preferences(
        &self,
        req: edgegateway::GetApiKeyPreferencesRequest,
    ) -> Result<edgegateway::GetApiKeyPreferencesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetApiKeyPreferences", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetApiKeyPreferences", "running workflow");
        let outgoing_0 = edgegateway::TestRouteBindingConnectionRequest {
            annotations: (req.context.clone().unwrap_or_default().baggage.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(edgegateway::test_route_binding_connection_request::Details {
                message_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                received_at: req.context.clone().unwrap_or_default().deadline.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::TestRouteBindingConnectionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.test_route_binding_connection(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::GetApiKeyPreferencesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamApiKeyNotices after validating the tenant and durably recording the command.
    async fn run_stream_api_key_notices(
        &self,
        req: edgegateway::StreamApiKeyNoticesRequest,
    ) -> Result<edgegateway::StreamApiKeyNoticesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StreamApiKeyNotices", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamApiKeyNotices", "running workflow");
        let outgoing_0 = edgegateway::SignalGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            details: Some(edgegateway::signal_gateway_policy_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    preferred_channel: req.subscriber.clone().unwrap_or_default().preferred_channel.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = edgegateway::ListRouteBindingSourcesRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_route_binding_sources_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = GatewayPolicyWorkflowServiceClient::new(channel_0);
                let input: tonic::Request<edgegateway::SignalGatewayPolicyWorkflowRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.signal_gateway_policy_workflow(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = RouteBindingIntegrationServiceClient::new(channel_1);
                let input: tonic::Request<edgegateway::ListRouteBindingSourcesRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_route_binding_sources(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = edgegateway::GenerateGatewayPolicyReportRequest {
            context: req.context.clone(),
            gateway_policy_id: reply_0
                .gateway_policy
                .clone()
                .unwrap_or_default()
                .gateway_policy_id
                .clone(),
            reason: (match reply_0.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::GenerateGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_3 = edgegateway::CountGatewayPolicyTotalsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::count_gateway_policy_totals_request::Filter {
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::CountGatewayPolicyTotalsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_gateway_policy_totals(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::StreamApiKeyNoticesResponse::default();
        let contribution: String = reply_0
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().phone.clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .phone = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by SendApiKeyTestNotice for DisconnectApiKeySource.
struct SendApiKeyTestNoticeDispatch3 {
    context: Option<common::RequestContext>,
    annotations: std::collections::BTreeMap<String, String>,
    details_message_id: String,
}
impl From<edgegateway::SendApiKeyTestNoticeRequest> for SendApiKeyTestNoticeDispatch3 {
    fn from(req: edgegateway::SendApiKeyTestNoticeRequest) -> Self {
        Self {
            context: req.context.clone(),
            annotations: req.annotations.clone(),
            details_message_id: Default::default(),
        }
    }
}
impl From<SendApiKeyTestNoticeDispatch3> for edgegateway::DisconnectApiKeySourceRequest {
    fn from(value: SendApiKeyTestNoticeDispatch3) -> Self {
        edgegateway::DisconnectApiKeySourceRequest {
            annotations: value.annotations,
            context: value.context,
            details: Some(edgegateway::disconnect_api_key_source_request::Details {
                message_id: value.details_message_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ApiKeyNotificationService for ApiKeyNotificationEndpoint {
    /// Subscribes an api key.
    async fn subscribe_api_key_updates(
        &self,
        request: tonic::Request<edgegateway::SubscribeApiKeyUpdatesRequest>,
    ) -> Result<tonic::Response<edgegateway::SubscribeApiKeyUpdatesResponse>, tonic::Status> {
        self.run_subscribe_api_key_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unsubscribes an api key.
    async fn unsubscribe_api_key_updates(
        &self,
        request: tonic::Request<edgegateway::UnsubscribeApiKeyUpdatesRequest>,
    ) -> Result<tonic::Response<edgegateway::UnsubscribeApiKeyUpdatesResponse>, tonic::Status> {
        self.run_unsubscribe_api_key_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Notifys an api key.
    async fn notify_api_key_watchers(
        &self,
        request: tonic::Request<edgegateway::NotifyApiKeyWatchersRequest>,
    ) -> Result<tonic::Response<edgegateway::NotifyApiKeyWatchersResponse>, tonic::Status> {
        self.run_notify_api_key_watchers(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists api keys a page at a time.
    async fn list_api_key_subscriptions(
        &self,
        request: tonic::Request<edgegateway::ListApiKeySubscriptionsRequest>,
    ) -> Result<tonic::Response<edgegateway::ListApiKeySubscriptionsResponse>, tonic::Status> {
        self.run_list_api_key_subscriptions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Sends an api key.
    async fn send_api_key_test_notice(
        &self,
        request: tonic::Request<edgegateway::SendApiKeyTestNoticeRequest>,
    ) -> Result<tonic::Response<edgegateway::SendApiKeyTestNoticeResponse>, tonic::Status> {
        self.run_send_api_key_test_notice(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an api key.
    async fn get_api_key_preferences(
        &self,
        request: tonic::Request<edgegateway::GetApiKeyPreferencesRequest>,
    ) -> Result<tonic::Response<edgegateway::GetApiKeyPreferencesResponse>, tonic::Status> {
        self.run_get_api_key_preferences(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamApiKeyNoticesStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::StreamApiKeyNoticesResponse, tonic::Status>>;
    /// Streams the changes of api keys as they happen.
    async fn stream_api_key_notices(
        &self,
        request: tonic::Request<edgegateway::StreamApiKeyNoticesRequest>,
    ) -> Result<tonic::Response<Self::StreamApiKeyNoticesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_api_key_notices(request).await });
        Ok(tonic::Response::new(stream))
    }
}
