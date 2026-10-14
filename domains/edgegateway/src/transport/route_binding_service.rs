use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_history_service_client::RoleBindingHistoryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::contracts::v1 as contracts;
use acme_proto::acme::contracts::v1::contract_service_client::ContractServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_integration_service_client::ApiKeyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_notification_service_client::ApiKeyNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_integration_service_client::GatewayPolicyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_notification_service_client::RouteBindingNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_query_service_client::RouteBindingQueryServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_service_server::RouteBindingService;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::fanportal::v1::fan_session_query_service_client::FanSessionQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_query_service_client::NotificationQueryServiceClient;

/// Manages route bindings: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct RouteBindingHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    batch_create_route_bindings_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            edgegateway::ListRouteBindingSubscriptionsRequest,
            edgegateway::ListRouteBindingSubscriptionsResponse,
        >,
    >,
    batch_get_route_bindings_0_port: acme_platform::ports::Callback<
        edgegateway::AcknowledgeGatewayPolicyEventRequest,
        edgegateway::AcknowledgeGatewayPolicyEventResponse,
    >,
    clone_route_binding_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            notifications::ResolveNotificationsRequest,
            notifications::ResolveNotificationsResponse,
        >,
    >,
    update_route_binding_1_port:
        acme_platform::ports::Callback<edgegateway::PeekRouteBindingRequest, edgegateway::PeekRouteBindingResponse>,
    update_route_binding_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<fanportal::BrowseFanSessionsRequest, fanportal::BrowseFanSessionsResponse>,
    >,
    watch_route_bindings_0_port: acme_platform::ports::Callback<
        edgegateway::GenerateGatewayPolicyReportRequest,
        edgegateway::GenerateGatewayPolicyReportResponse,
    >,
}
impl RouteBindingHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            edgegateway::PeekRouteBindingRequest,
            edgegateway::PeekRouteBindingResponse,
        > = {
            let client = RouteBindingQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::PeekRouteBindingRequest| -> acme_platform::ports::RpcFuture<edgegateway::PeekRouteBindingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.peek_route_binding(request).await?.into_inner())
})
})
        };
        let update_route_binding_1_port = callback;
        let callback: acme_platform::ports::Callback<
            fanportal::BrowseFanSessionsRequest,
            fanportal::BrowseFanSessionsResponse,
        > = {
            let client = FanSessionQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: fanportal::BrowseFanSessionsRequest| -> acme_platform::ports::RpcFuture<fanportal::BrowseFanSessionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_fan_sessions(request).await?.into_inner())
})
})
        };
        let update_route_binding_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            edgegateway::AcknowledgeGatewayPolicyEventRequest,
            edgegateway::AcknowledgeGatewayPolicyEventResponse,
        > = {
            let client = GatewayPolicyIntegrationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::AcknowledgeGatewayPolicyEventRequest| -> acme_platform::ports::RpcFuture<edgegateway::AcknowledgeGatewayPolicyEventResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.acknowledge_gateway_policy_event(request).await?.into_inner())
})
})
        };
        let batch_get_route_bindings_0_port = callback;
        let callback: acme_platform::ports::Callback<
            edgegateway::GenerateGatewayPolicyReportRequest,
            edgegateway::GenerateGatewayPolicyReportResponse,
        > = {
            let client = GatewayPolicyReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::GenerateGatewayPolicyReportRequest| -> acme_platform::ports::RpcFuture<edgegateway::GenerateGatewayPolicyReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.generate_gateway_policy_report(request).await?.into_inner())
})
})
        };
        let watch_route_bindings_0_port = callback;
        let callback: acme_platform::ports::Callback<
            edgegateway::ListRouteBindingSubscriptionsRequest,
            edgegateway::ListRouteBindingSubscriptionsResponse,
        > = {
            let client = RouteBindingNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::ListRouteBindingSubscriptionsRequest| -> acme_platform::ports::RpcFuture<edgegateway::ListRouteBindingSubscriptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_route_binding_subscriptions(request).await?.into_inner())
})
})
        };
        let batch_create_route_bindings_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            notifications::ResolveNotificationsRequest,
            notifications::ResolveNotificationsResponse,
        > = {
            let client = NotificationQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: notifications::ResolveNotificationsRequest| -> acme_platform::ports::RpcFuture<notifications::ResolveNotificationsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_notifications(request).await?.into_inner())
})
})
        };
        let clone_route_binding_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            batch_create_route_bindings_0_port,
            batch_get_route_bindings_0_port,
            clone_route_binding_0_port,
            update_route_binding_1_port,
            update_route_binding_2_port,
            watch_route_bindings_0_port,
        }
    }
    /// Run CreateRouteBinding after validating the tenant and durably recording the command.
    async fn run_create_route_binding(
        &self,
        req: edgegateway::CreateRouteBindingRequest,
    ) -> Result<edgegateway::CreateRouteBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateRouteBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateRouteBinding", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().route_binding_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::IngestApiKeyEventsRequest {
            context: req.context.clone(),
            chunk: Some(edgegateway::ingest_api_key_events_request::Chunk::Record(
                edgegateway::ApiKey {
                    audit: Some(common::AuditInfo {
                        etag: req
                            .route_binding
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .etag
                            .clone(),
                        updated_by: req
                            .route_binding
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = ApiKeyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::IngestApiKeyEventsRequest> =
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
            let response = client.ingest_api_key_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = contracts::SearchContractsRequest {
            context: req.context.clone(),
            filter: Some(contracts::search_contracts_request::Filter {
                created_after: req
                    .route_binding
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
        let outgoing_1 = acme_platform::codec::through_any(&outgoing_1, "acme.contracts.v1.SearchContractsRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ContractServiceClient::new(channel);
            let input: tonic::Request<contracts::SearchContractsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_contracts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::SendApiKeyTestNoticeRequest {
            context: req.context.clone(),
            details: Some(edgegateway::send_api_key_test_notice_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
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
                variables: req.route_binding.clone().unwrap_or_default().parameters.clone(),
                ..Default::default()
            }),
            reason: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::route_binding::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<edgegateway::SendApiKeyTestNoticeRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ApiKeyNotificationServiceClient::new(channel);
                let input: tonic::Request<edgegateway::SendApiKeyTestNoticeRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.send_api_key_test_notice(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_2)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_2 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let outgoing_3 = audit::BatchCreateAuditEventsRequest {
            context: req.context.clone(),
            chunk: Some(audit::batch_create_audit_events_request::Chunk::Record(
                audit::AuditEvent {
                    correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                    measurements: vec![audit::AuditEventMeasurement {
                        observed_at: reply_2
                            .api_key
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        ..Default::default()
                    }],
                    subject: Some(audit::AuditTrailRef {
                        display_name: req
                            .route_binding
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::BatchCreateAuditEventsRequest> =
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
            let response = client.batch_create_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    participants: vec![featureflags::FlagRolloutParticipant {
                        contact: Some(common::ContactInfo {
                            email: reply_2
                                .api_key
                                .clone()
                                .unwrap_or_default()
                                .contact
                                .clone()
                                .unwrap_or_default()
                                .email
                                .clone(),
                            name: reply_2
                                .api_key
                                .clone()
                                .unwrap_or_default()
                                .contact
                                .clone()
                                .unwrap_or_default()
                                .name
                                .clone(),
                            preferred_channel: reply_2
                                .api_key
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
            let response = client.stream_flag_rollout_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagRolloutChangesResponse::default();
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
        let mut response = edgegateway::CreateRouteBindingResponse::default();
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
            .api_key
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.route_binding.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.route_binding.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.route_binding.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.route_binding.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run GetRouteBinding after validating the tenant and durably recording the command.
    async fn run_get_route_binding(
        &self,
        req: edgegateway::GetRouteBindingRequest,
    ) -> Result<edgegateway::GetRouteBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRouteBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRouteBinding", "running workflow");
        let outgoing_0 = fanportal::CountFanSessionsRequest {
            context: req.context.clone(),
            filter: Some(fanportal::count_fan_sessions_request::Filter {
                owner_id: req.route_binding_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FanSessionQueryServiceClient::new(channel);
            let input: tonic::Request<fanportal::CountFanSessionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_fan_sessions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::AnnotateRoleBindingVersionRequest {
            context: req.context.clone(),
            details: Some(authorization::annotate_role_binding_version_request::Details {
                author_email: req
                    .context
                    .clone()
                    .unwrap_or_default()
                    .actor
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            role_binding: Some(authorization::RoleBinding {
                due_at: req.context.clone().unwrap_or_default().deadline.clone(),
                lines: vec![authorization::RoleBindingLine {
                    attributes: req.context.clone().unwrap_or_default().baggage.clone(),
                    ..Default::default()
                }],
                metadata: reply_0.result.clone().unwrap_or_default().details.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RoleBindingHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::AnnotateRoleBindingVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.annotate_role_binding_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::GetRouteBindingResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .role_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.route_binding_id.clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .route_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run UpdateRouteBinding after validating the tenant and durably recording the command.
    async fn run_update_route_binding(
        &self,
        req: edgegateway::UpdateRouteBindingRequest,
    ) -> Result<edgegateway::UpdateRouteBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UpdateRouteBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UpdateRouteBinding", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::ReplayGatewayPolicyEventsRequest {
            context: req.context.clone(),
            details: Some(edgegateway::replay_gateway_policy_events_request::Details {
                message_id: req
                    .route_binding
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                received_at: req
                    .route_binding
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                ..Default::default()
            }),
            reason: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::route_binding::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = GatewayPolicyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ReplayGatewayPolicyEventsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.replay_gateway_policy_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::PeekRouteBindingRequest {
            context: req.context.clone(),
            route_binding_id: req.route_binding.clone().unwrap_or_default().route_binding_id.clone(),
            ..Default::default()
        };
        let reply_1 = (self.update_route_binding_1_port)(outgoing_1).await?;
        let outgoing_2 = fanportal::BrowseFanSessionsRequest {
            context: req.context.clone(),
            filter: Some(fanportal::browse_fan_sessions_request::Filter {
                created_after: reply_1
                    .route_binding
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: req
                    .route_binding
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
        let reply_2 =
            self.update_route_binding_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let mut response = edgegateway::UpdateRouteBindingResponse::default();
        let contribution: i64 = reply_0.gateway_policy.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.route_binding.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.route_binding.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .parameters
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.route_binding.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.route_binding.get_or_insert_with(Default::default).effective = contribution;
        }
        let contribution: String = req.route_binding.clone().unwrap_or_default().route_binding_id.clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .route_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run ListRouteBindings after validating the tenant and durably recording the command.
    async fn run_list_route_bindings(
        &self,
        req: edgegateway::ListRouteBindingsRequest,
    ) -> Result<edgegateway::ListRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListRouteBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRouteBindings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = edgegateway::ListRouteBindingsResponse::default();
        Ok(response)
    }
    /// Run BatchGetRouteBindings after validating the tenant and durably recording the command.
    async fn run_batch_get_route_bindings(
        &self,
        req: edgegateway::BatchGetRouteBindingsRequest,
    ) -> Result<edgegateway::BatchGetRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchGetRouteBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchGetRouteBindings", "running workflow");
        let outgoing_0 = edgegateway::AcknowledgeGatewayPolicyEventRequest {
            context: req.context.clone(),
            details: Some(edgegateway::acknowledge_gateway_policy_event_request::Details {
                message_id: req
                    .context
                    .clone()
                    .unwrap_or_default()
                    .actor
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                received_at: req.context.clone().unwrap_or_default().deadline.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.batch_get_route_bindings_0_port)(outgoing_0).await?;
        let mut response = edgegateway::BatchGetRouteBindingsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::TimeWindow> =
            reply_0.gateway_policy.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            if response.route_bindings.is_empty() {
                response.route_bindings.push(Default::default());
            }
            response.route_bindings[0].effective = contribution;
        }
        Ok(response)
    }
    /// Run SearchRouteBindings after validating the tenant and durably recording the command.
    async fn run_search_route_bindings(
        &self,
        req: edgegateway::SearchRouteBindingsRequest,
    ) -> Result<edgegateway::SearchRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SearchRouteBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SearchRouteBindings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = edgegateway::SearchRouteBindingsResponse::default();
        Ok(response)
    }
    /// Run WatchRouteBindings after validating the tenant and durably recording the command.
    async fn run_watch_route_bindings(
        &self,
        req: edgegateway::WatchRouteBindingsRequest,
    ) -> Result<edgegateway::WatchRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("WatchRouteBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "WatchRouteBindings", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::GenerateGatewayPolicyReportRequest {
            annotations: req.filter.clone().unwrap_or_default().labels.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::generate_gateway_policy_report_request::Details {
                filters: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.watch_route_bindings_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = edgegateway::SubscribeRouteBindingUpdatesRequest {
            context: req.context.clone(),
            details: Some(edgegateway::subscribe_route_binding_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    phone: req.subscriber.clone().unwrap_or_default().phone.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<edgegateway::SubscribeRouteBindingUpdatesRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SubscribeRouteBindingUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_route_binding_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = WatchRouteBindingsDispatch2::from(req.clone());
        command_2.reason = (match reply_1.route_binding.clone().unwrap_or_default().action.clone() {
            Some(edgegateway::route_binding::Action::Block(value)) => value,
            _ => Default::default(),
        })
        .reason
        .clone();
        let outgoing_2: edgegateway::SummarizeRouteBindingsRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RouteBindingQueryServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SummarizeRouteBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_route_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = edgegateway::TestApiKeyConnectionRequest {
            context: req.context.clone(),
            details: Some(edgegateway::test_api_key_connection_request::Details {
                message_id: reply_1.result.clone().unwrap_or_default().message.clone(),
                ..Default::default()
            }),
            reason: (match reply_1.route_binding.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::route_binding::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ApiKeyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::TestApiKeyConnectionRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.test_api_key_connection(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::WatchRouteBindingsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: String = reply_1
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
            .api_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        Ok(response)
    }
    /// Run BatchCreateRouteBindings after validating the tenant and durably recording the command.
    async fn run_batch_create_route_bindings(
        &self,
        req: edgegateway::BatchCreateRouteBindingsRequest,
    ) -> Result<edgegateway::BatchCreateRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("BatchCreateRouteBindings", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateRouteBindings", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(edgegateway::batch_create_route_bindings_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::ListRouteBindingSubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_route_binding_subscriptions_request::Filter {
                created_after: (match req.chunk.clone() {
                    Some(edgegateway::batch_create_route_bindings_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.batch_create_route_bindings_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = edgegateway::StreamApiKeyNoticesRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: (match (match req.chunk.clone() {
                    Some(edgegateway::batch_create_route_bindings_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .action
                .clone()
                {
                    Some(edgegateway::route_binding::Action::Block(value)) => value,
                    _ => Default::default(),
                })
                .notify_email
                .clone(),
                name: (match req.chunk.clone() {
                    Some(edgegateway::batch_create_route_bindings_request::Chunk::Header(value)) => value,
                    _ => Default::default(),
                })
                .submitted_by
                .clone()
                .unwrap_or_default()
                .name
                .clone(),
                phone: (match req.chunk.clone() {
                    Some(edgegateway::batch_create_route_bindings_request::Chunk::Header(value)) => value,
                    _ => Default::default(),
                })
                .submitted_by
                .clone()
                .unwrap_or_default()
                .phone
                .clone(),
                preferred_channel: (match req.chunk.clone() {
                    Some(edgegateway::batch_create_route_bindings_request::Chunk::Header(value)) => value,
                    _ => Default::default(),
                })
                .submitted_by
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
            let mut client = ApiKeyNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::StreamApiKeyNoticesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_api_key_notices(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::StreamApiKeyNoticesResponse::default();
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
        let mut response = edgegateway::BatchCreateRouteBindingsResponse::default();
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
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run CloneRouteBinding after validating the tenant and durably recording the command.
    async fn run_clone_route_binding(
        &self,
        req: edgegateway::CloneRouteBindingRequest,
    ) -> Result<edgegateway::CloneRouteBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CloneRouteBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CloneRouteBinding", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = notifications::ResolveNotificationsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.clone_route_binding_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = edgegateway::IngestGatewayPolicyEventsRequest {
            context: req.context.clone(),
            chunk: Some(edgegateway::ingest_gateway_policy_events_request::Chunk::Record(
                edgegateway::GatewayPolicy {
                    audit: Some(common::AuditInfo {
                        created_by: req
                            .route_binding
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        ..Default::default()
                    }),
                    action: Some(edgegateway::gateway_policy::Action::Route(
                        edgegateway::GatewayPolicyRouteAction {
                            target: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                                Some(edgegateway::route_binding::Action::Route(value)) => value,
                                _ => Default::default(),
                            })
                            .target
                            .clone(),
                            weight: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                                Some(edgegateway::route_binding::Action::Route(value)) => value,
                                _ => Default::default(),
                            })
                            .weight
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
        let _reply_1 = async move {
            let mut client = GatewayPolicyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::IngestGatewayPolicyEventsRequest> =
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
            let response = client.ingest_gateway_policy_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::TestGatewayPolicyConnectionRequest {
            annotations: reply_0.result.clone().unwrap_or_default().details.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::test_gateway_policy_connection_request::Details {
                message_id: reply_0.result.clone().unwrap_or_default().message.clone(),
                received_at: req
                    .route_binding
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                ..Default::default()
            }),
            reason: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::route_binding::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let outgoing_2 =
            acme_platform::ports::RequestBuilder::<edgegateway::TestGatewayPolicyConnectionRequest>::default()
                .map(|draft| *draft = outgoing_2)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::TestGatewayPolicyConnectionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.test_gateway_policy_connection(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::CloneRouteBindingResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<identity::UserRef> = reply_2.gateway_policy.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.route_binding.get_or_insert_with(Default::default).owner = contribution;
        }
        let contribution: i32 = req.route_binding.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.route_binding.get_or_insert_with(Default::default).priority = contribution;
        }
        let contribution: String = req.route_binding.clone().unwrap_or_default().route_binding_id.clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .route_binding_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by WatchRouteBindings for SummarizeRouteBindings.
struct WatchRouteBindingsDispatch2 {
    context: Option<common::RequestContext>,
    reason: String,
}
impl From<edgegateway::WatchRouteBindingsRequest> for WatchRouteBindingsDispatch2 {
    fn from(req: edgegateway::WatchRouteBindingsRequest) -> Self {
        Self {
            context: req.context.clone(),
            reason: Default::default(),
        }
    }
}
impl From<WatchRouteBindingsDispatch2> for edgegateway::SummarizeRouteBindingsRequest {
    fn from(value: WatchRouteBindingsDispatch2) -> Self {
        edgegateway::SummarizeRouteBindingsRequest {
            context: value.context,
            reason: value.reason,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl RouteBindingService for RouteBindingHandler {
    /// Creates a route binding.
    async fn create_route_binding(
        &self,
        request: tonic::Request<edgegateway::CreateRouteBindingRequest>,
    ) -> Result<tonic::Response<edgegateway::CreateRouteBindingResponse>, tonic::Status> {
        self.run_create_route_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a route binding.
    async fn get_route_binding(
        &self,
        request: tonic::Request<edgegateway::GetRouteBindingRequest>,
    ) -> Result<tonic::Response<edgegateway::GetRouteBindingResponse>, tonic::Status> {
        self.run_get_route_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Changes a route binding.
    async fn update_route_binding(
        &self,
        request: tonic::Request<edgegateway::UpdateRouteBindingRequest>,
    ) -> Result<tonic::Response<edgegateway::UpdateRouteBindingResponse>, tonic::Status> {
        self.run_update_route_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists route bindings a page at a time.
    async fn list_route_bindings(
        &self,
        request: tonic::Request<edgegateway::ListRouteBindingsRequest>,
    ) -> Result<tonic::Response<edgegateway::ListRouteBindingsResponse>, tonic::Status> {
        self.run_list_route_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several route bindings at once.
    async fn batch_get_route_bindings(
        &self,
        request: tonic::Request<edgegateway::BatchGetRouteBindingsRequest>,
    ) -> Result<tonic::Response<edgegateway::BatchGetRouteBindingsResponse>, tonic::Status> {
        self.run_batch_get_route_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists route bindings a page at a time.
    async fn search_route_bindings(
        &self,
        request: tonic::Request<edgegateway::SearchRouteBindingsRequest>,
    ) -> Result<tonic::Response<edgegateway::SearchRouteBindingsResponse>, tonic::Status> {
        self.run_search_route_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type WatchRouteBindingsStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::WatchRouteBindingsResponse, tonic::Status>>;
    /// Streams the changes of route bindings as they happen.
    async fn watch_route_bindings(
        &self,
        request: tonic::Request<edgegateway::WatchRouteBindingsRequest>,
    ) -> Result<tonic::Response<Self::WatchRouteBindingsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_watch_route_bindings(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Takes a stream of route bindings and answers once at the end.
    async fn batch_create_route_bindings(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::BatchCreateRouteBindingsRequest>>,
    ) -> Result<tonic::Response<edgegateway::BatchCreateRouteBindingsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = edgegateway::BatchCreateRouteBindingsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_batch_create_route_bindings(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Creates a route binding.
    async fn clone_route_binding(
        &self,
        request: tonic::Request<edgegateway::CloneRouteBindingRequest>,
    ) -> Result<tonic::Response<edgegateway::CloneRouteBindingResponse>, tonic::Status> {
        self.run_clone_route_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
