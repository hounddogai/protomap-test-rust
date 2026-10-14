use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_export_service_client::RoleBindingExportServiceClient;
use acme_proto::acme::authorization::v1::role_binding_notification_service_client::RoleBindingNotificationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_client::ApiKeyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_server::RouteBindingIntegrationService;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::fanportal::v1::fan_session_query_service_client::FanSessionQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_export_service_client::FlagRolloutExportServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_client::KeyRingReportingServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_workflow_service_client::AccessGrantWorkflowServiceClient;
use acme_proto::acme::secrets::v1::secret_approval_service_client::SecretApprovalServiceClient;
use acme_proto::acme::timekeeping::v1 as timekeeping;
use acme_proto::acme::timekeeping::v1::timesheet_query_service_client::TimesheetQueryServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Connects external systems that send route binding events.
#[derive(Clone)]
pub struct RouteBindingIntegrationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    disconnect_route_binding_source_2_port: acme_platform::ports::Callback<
        edgegateway::PauseApiKeyWorkflowRequest,
        edgegateway::PauseApiKeyWorkflowResponse,
    >,
    ingest_route_binding_events_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            edgegateway::StartApiKeyWorkflowRequest,
            edgegateway::StartApiKeyWorkflowResponse,
        >,
    >,
    replay_route_binding_events_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<timekeeping::FindTimesheetsRequest, timekeeping::FindTimesheetsResponse>,
    >,
    test_route_binding_connection_0_port:
        std::sync::Arc<dyn acme_platform::ports::RpcPort<secrets::RecallSecretRequest, secrets::RecallSecretResponse>>,
    test_route_binding_connection_1_port: acme_platform::ports::Callback<
        edgegateway::StartApiKeyWorkflowRequest,
        edgegateway::StartApiKeyWorkflowResponse,
    >,
}
impl RouteBindingIntegrationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            edgegateway::PauseApiKeyWorkflowRequest,
            edgegateway::PauseApiKeyWorkflowResponse,
        > = {
            let client = ApiKeyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::PauseApiKeyWorkflowRequest| -> acme_platform::ports::RpcFuture<edgegateway::PauseApiKeyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.pause_api_key_workflow(request).await?.into_inner())
})
})
        };
        let disconnect_route_binding_source_2_port = callback;
        let callback: acme_platform::ports::Callback<
            edgegateway::StartApiKeyWorkflowRequest,
            edgegateway::StartApiKeyWorkflowResponse,
        > = {
            let client = ApiKeyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::StartApiKeyWorkflowRequest| -> acme_platform::ports::RpcFuture<edgegateway::StartApiKeyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.start_api_key_workflow(request).await?.into_inner())
})
})
        };
        let ingest_route_binding_events_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            timekeeping::FindTimesheetsRequest,
            timekeeping::FindTimesheetsResponse,
        > = {
            let client = TimesheetQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: timekeeping::FindTimesheetsRequest| -> acme_platform::ports::RpcFuture<timekeeping::FindTimesheetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.find_timesheets(request).await?.into_inner())
})
})
        };
        let replay_route_binding_events_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<secrets::RecallSecretRequest, secrets::RecallSecretResponse> = {
            let client = SecretApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::RecallSecretRequest| -> acme_platform::ports::RpcFuture<secrets::RecallSecretResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recall_secret(request).await?.into_inner())
})
})
        };
        let test_route_binding_connection_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            edgegateway::StartApiKeyWorkflowRequest,
            edgegateway::StartApiKeyWorkflowResponse,
        > = {
            let client = ApiKeyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::StartApiKeyWorkflowRequest| -> acme_platform::ports::RpcFuture<edgegateway::StartApiKeyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.start_api_key_workflow(request).await?.into_inner())
})
})
        };
        let test_route_binding_connection_1_port = callback;
        Self {
            channel,
            repository,
            disconnect_route_binding_source_2_port,
            ingest_route_binding_events_0_port,
            replay_route_binding_events_2_port,
            test_route_binding_connection_0_port,
            test_route_binding_connection_1_port,
        }
    }
    /// Run ConnectRouteBindingSource after validating the tenant and durably recording the command.
    async fn run_connect_route_binding_source(
        &self,
        req: edgegateway::ConnectRouteBindingSourceRequest,
    ) -> Result<edgegateway::ConnectRouteBindingSourceResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ConnectRouteBindingSource", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ConnectRouteBindingSource", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = venues::CloneVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                audit: Some(common::AuditInfo {
                    updated_at: req.details.clone().unwrap_or_default().received_at.clone(),
                    ..Default::default()
                }),
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<venues::CloneVenueRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = VenueServiceClient::new(channel);
                let input: tonic::Request<venues::CloneVenueRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.clone_venue(input).await?;
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
        let outgoing_1 = featureflags::ScheduleFlagRolloutExportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(featureflags::schedule_flag_rollout_export_request::Details {
                filters: reply_0.venue.clone().unwrap_or_default().specs.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = FlagRolloutExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::ScheduleFlagRolloutExportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_flag_rollout_export(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = audit::BatchCreateAuditEventsRequest {
            context: req.context.clone(),
            chunk: Some(audit::batch_create_audit_events_request::Chunk::Record(
                audit::AuditEvent {
                    correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::BatchCreateAuditEventsRequest> =
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
            let response = client.batch_create_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        created_at: reply_0
                            .venue
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        created_by: reply_0
                            .venue
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        updated_at: reply_0
                            .venue
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
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
        let mut response = edgegateway::ConnectRouteBindingSourceResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run DisconnectRouteBindingSource after validating the tenant and durably recording the command.
    async fn run_disconnect_route_binding_source(
        &self,
        req: edgegateway::DisconnectRouteBindingSourceRequest,
    ) -> Result<edgegateway::DisconnectRouteBindingSourceResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DisconnectRouteBindingSource", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DisconnectRouteBindingSource", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::SignalApiKeyWorkflowRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SignalApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::StartApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                funding: Some(edgegateway::api_key::Funding::Bank(edgegateway::ApiKeyBankFunding {
                    bic: (match reply_0.api_key.clone().unwrap_or_default().funding.clone() {
                        Some(edgegateway::api_key::Funding::Bank(value)) => value,
                        _ => Default::default(),
                    })
                    .bic
                    .clone(),
                    ..Default::default()
                })),
                contact: Some(common::ContactInfo {
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
                holder: Some(edgegateway::GatewayPolicyRef {
                    gateway_policy_id: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .gateway_policy_id
                        .clone(),
                    version: reply_0
                        .api_key
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::StartApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.start_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::PauseApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                holder: Some(edgegateway::GatewayPolicyRef {
                    gateway_policy_id: reply_1
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .gateway_policy_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(edgegateway::pause_api_key_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    phone: reply_0
                        .api_key
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
        let reply_2 = (self.disconnect_route_binding_source_2_port)(outgoing_2).await?;
        let mut response = edgegateway::DisconnectRouteBindingSourceResponse::default();
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: i64 = reply_1
            .api_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response.route_binding.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2
            .api_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run IngestRouteBindingEvents after validating the tenant and durably recording the command.
    async fn run_ingest_route_binding_events(
        &self,
        req: edgegateway::IngestRouteBindingEventsRequest,
    ) -> Result<edgegateway::IngestRouteBindingEventsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("IngestRouteBindingEvents", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "IngestRouteBindingEvents", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(edgegateway::ingest_route_binding_events_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::StartApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                audit: Some(common::AuditInfo {
                    updated_by: (match req.chunk.clone() {
                        Some(edgegateway::ingest_route_binding_events_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_by
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(edgegateway::start_api_key_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: (match req.chunk.clone() {
                        Some(edgegateway::ingest_route_binding_events_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                    phone: (match req.chunk.clone() {
                        Some(edgegateway::ingest_route_binding_events_request::Chunk::Header(value)) => value,
                        _ => Default::default(),
                    })
                    .submitted_by
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
        let reply_0 =
            self.ingest_route_binding_events_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = edgegateway::PauseApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                funding: Some(edgegateway::api_key::Funding::Card(edgegateway::ApiKeyCardFunding {
                    last4: (match reply_0.api_key.clone().unwrap_or_default().funding.clone() {
                        Some(edgegateway::api_key::Funding::Card(value)) => value,
                        _ => Default::default(),
                    })
                    .last4
                    .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::PauseApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pause_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::ListApiKeyWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_api_key_workflows_request::Filter {
                created_after: reply_1
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: (reply_1.api_key.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: reply_1
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .gateway_policy_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListApiKeyWorkflowsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_api_key_workflows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::IngestRouteBindingEventsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ReplayRouteBindingEvents after validating the tenant and durably recording the command.
    async fn run_replay_route_binding_events(
        &self,
        req: edgegateway::ReplayRouteBindingEventsRequest,
    ) -> Result<edgegateway::ReplayRouteBindingEventsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ReplayRouteBindingEvents", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReplayRouteBindingEvents", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = authorization::SendRoleBindingTestNoticeRequest {
            context: req.context.clone(),
            details: Some(authorization::send_role_binding_test_notice_request::Details {
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
            let mut client = RoleBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::SendRoleBindingTestNoticeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.send_role_binding_test_notice(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::ResumeCryptoKeyWorkflowRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                funding: Some(keymanagement::crypto_key::Funding::Card(
                    keymanagement::CryptoKeyCardFunding {
                        holder_name: (match reply_0.role_binding.clone().unwrap_or_default().settlement.clone() {
                            Some(authorization::role_binding::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .holder_name
                        .clone(),
                        network: (match reply_0.role_binding.clone().unwrap_or_default().settlement.clone() {
                            Some(authorization::role_binding::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .network
                        .clone(),
                        token: (match reply_0.role_binding.clone().unwrap_or_default().settlement.clone() {
                            Some(authorization::role_binding::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .token
                        .clone(),
                        ..Default::default()
                    },
                )),
                holder: Some(keymanagement::KeyRingRef {
                    display_name: reply_0
                        .role_binding
                        .clone()
                        .unwrap_or_default()
                        .counterparty
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
        let reply_1 = async move {
            let mut client = CryptoKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ResumeCryptoKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_crypto_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = timekeeping::FindTimesheetsRequest {
            context: req.context.clone(),
            filter: Some(timekeeping::find_timesheets_request::Filter {
                created_after: reply_0.role_binding.clone().unwrap_or_default().issued_at.clone(),
                labels: reply_1.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_0.outcome.clone().unwrap_or_default().reference_id.clone(),
                region: reply_0
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
        let reply_2 =
            acme_platform::ports::execute(self.replay_route_binding_events_2_port.as_ref(), outgoing_2).await?;
        let mut response = edgegateway::ReplayRouteBindingEventsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_0
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
        let contribution: String = reply_1
            .crypto_key
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
    /// Run AcknowledgeRouteBindingEvent after validating the tenant and durably recording the command.
    async fn run_acknowledge_route_binding_event(
        &self,
        req: edgegateway::AcknowledgeRouteBindingEventRequest,
    ) -> Result<edgegateway::AcknowledgeRouteBindingEventResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AcknowledgeRouteBindingEvent", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AcknowledgeRouteBindingEvent", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = secrets::RetryAccessGrantWorkflowRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(secrets::retry_access_grant_workflow_request::Details {
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
        let outgoing_0 = acme_platform::ports::RequestBuilder::<secrets::RetryAccessGrantWorkflowRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantWorkflowServiceClient::new(channel);
            let input: tonic::Request<secrets::RetryAccessGrantWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.retry_access_grant_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = AcknowledgeRouteBindingEventDispatch1::from(req.clone());
        command_1.filter_created_after = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        command_1.filter_owner_id = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        let outgoing_1: fanportal::CountFanSessionsRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FanSessionQueryServiceClient::new(channel);
            let input: tonic::Request<fanportal::CountFanSessionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_fan_sessions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = venues::UpdateVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: reply_0
                        .access_grant
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
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: reply_0
                        .access_grant
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
        let outgoing_2 = acme_platform::codec::through_any(&outgoing_2, "acme.venues.v1.UpdateVenueRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::UpdateVenueRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::AcknowledgeRouteBindingEventResponse::default();
        let contribution: String = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<common::TimeWindow> =
            reply_0.access_grant.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.route_binding.get_or_insert_with(Default::default).effective = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2
            .venue
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .venue
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
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
    /// Run RelayRouteBindingMessages after validating the tenant and durably recording the command.
    async fn run_relay_route_binding_messages(
        &self,
        req: edgegateway::RelayRouteBindingMessagesRequest,
    ) -> Result<edgegateway::RelayRouteBindingMessagesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RelayRouteBindingMessages", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RelayRouteBindingMessages", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(edgegateway::relay_route_binding_messages_request::Frame::Change(value)) => value, _ => Default::default() }).description.clone(), "recorded audit field");
        let outgoing_0 = authorization::ListRoleBindingExportsRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_role_binding_exports_request::Filter {
                created_after: (match req.frame.clone() {
                    Some(edgegateway::relay_route_binding_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
                labels: (match req.frame.clone() {
                    Some(edgegateway::relay_route_binding_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .parameters
                .clone(),
                owner_id: (match req.frame.clone() {
                    Some(edgegateway::relay_route_binding_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .route_binding_id
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<authorization::ListRoleBindingExportsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = RoleBindingExportServiceClient::new(channel);
                let input: tonic::Request<authorization::ListRoleBindingExportsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_role_binding_exports(input).await?;
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
        let outgoing_1 = edgegateway::ListApiKeyWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_api_key_workflows_request::Filter {
                created_after: (match req.frame.clone() {
                    Some(edgegateway::relay_route_binding_messages_request::Frame::Change(value)) => value,
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
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListApiKeyWorkflowsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_api_key_workflows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = edgegateway::RelayRouteBindingMessagesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<identity::UserRef> = (match req.frame.clone() {
            Some(edgegateway::relay_route_binding_messages_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .owner
        .clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.route_binding.get_or_insert_with(Default::default).owner = contribution;
        }
        let contribution: String = (match req.frame.clone() {
            Some(edgegateway::relay_route_binding_messages_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .name
        .clone();
        if contribution != <String>::default() {
            response.route_binding.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run ListRouteBindingSources after validating the tenant and durably recording the command.
    async fn run_list_route_binding_sources(
        &self,
        req: edgegateway::ListRouteBindingSourcesRequest,
    ) -> Result<edgegateway::ListRouteBindingSourcesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListRouteBindingSources", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRouteBindingSources", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let command_0 = ListRouteBindingSourcesDispatch0::from(req.clone());
        let outgoing_0: keymanagement::ScheduleKeyRingReportRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ScheduleKeyRingReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_key_ring_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::AdvanceApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
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
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.edgegateway.v1.AdvanceApiKeyWorkflowRequest")?;
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
        let mut response = edgegateway::ListRouteBindingSourcesResponse::default();
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
        let contribution: String = reply_1
            .api_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            if response.route_bindings.is_empty() {
                response.route_bindings.push(Default::default());
            }
            response.route_bindings[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_1
            .api_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response.route_bindings.is_empty() {
                response.route_bindings.push(Default::default());
            }
            response.route_bindings[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        Ok(response)
    }
    /// Run TestRouteBindingConnection after validating the tenant and durably recording the command.
    async fn run_test_route_binding_connection(
        &self,
        req: edgegateway::TestRouteBindingConnectionRequest,
    ) -> Result<edgegateway::TestRouteBindingConnectionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("TestRouteBindingConnection", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TestRouteBindingConnection", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = secrets::RecallSecretRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            secret: Some(secrets::Secret {
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
                holder: Some(identity::UserRef {
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
            acme_platform::ports::execute(self.test_route_binding_connection_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = edgegateway::StartApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_by: reply_0
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                funding: Some(edgegateway::api_key::Funding::Bank(edgegateway::ApiKeyBankFunding {
                    iban: (match reply_0.secret.clone().unwrap_or_default().funding.clone() {
                        Some(secrets::secret::Funding::Bank(value)) => value,
                        _ => Default::default(),
                    })
                    .iban
                    .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = (self.test_route_binding_connection_1_port)(outgoing_1).await?;
        let mut response = edgegateway::TestRouteBindingConnectionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0
            .secret
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1
            .api_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
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
}
/// The domain command prepared by AcknowledgeRouteBindingEvent for CountFanSessions.
struct AcknowledgeRouteBindingEventDispatch1 {
    context: Option<common::RequestContext>,
    filter_created_after: Option<prost_types::Timestamp>,
    filter_owner_id: String,
}
impl From<edgegateway::AcknowledgeRouteBindingEventRequest> for AcknowledgeRouteBindingEventDispatch1 {
    fn from(req: edgegateway::AcknowledgeRouteBindingEventRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_created_after: Default::default(),
            filter_owner_id: Default::default(),
        }
    }
}
impl From<AcknowledgeRouteBindingEventDispatch1> for fanportal::CountFanSessionsRequest {
    fn from(value: AcknowledgeRouteBindingEventDispatch1) -> Self {
        fanportal::CountFanSessionsRequest {
            context: value.context,
            filter: Some(fanportal::count_fan_sessions_request::Filter {
                created_after: value.filter_created_after,
                owner_id: value.filter_owner_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by ListRouteBindingSources for ScheduleKeyRingReport.
struct ListRouteBindingSourcesDispatch0 {
    context: Option<common::RequestContext>,
    details_filters: std::collections::BTreeMap<String, String>,
    annotations: std::collections::BTreeMap<String, String>,
}
impl From<edgegateway::ListRouteBindingSourcesRequest> for ListRouteBindingSourcesDispatch0 {
    fn from(req: edgegateway::ListRouteBindingSourcesRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_filters: req.filter.clone().unwrap_or_default().labels.clone(),
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
        }
    }
}
impl From<ListRouteBindingSourcesDispatch0> for keymanagement::ScheduleKeyRingReportRequest {
    fn from(value: ListRouteBindingSourcesDispatch0) -> Self {
        keymanagement::ScheduleKeyRingReportRequest {
            annotations: value.annotations,
            context: value.context,
            details: Some(keymanagement::schedule_key_ring_report_request::Details {
                filters: value.details_filters,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl RouteBindingIntegrationService for RouteBindingIntegrationController {
    /// Connects a route binding.
    async fn connect_route_binding_source(
        &self,
        request: tonic::Request<edgegateway::ConnectRouteBindingSourceRequest>,
    ) -> Result<tonic::Response<edgegateway::ConnectRouteBindingSourceResponse>, tonic::Status> {
        self.run_connect_route_binding_source(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Disconnects a route binding.
    async fn disconnect_route_binding_source(
        &self,
        request: tonic::Request<edgegateway::DisconnectRouteBindingSourceRequest>,
    ) -> Result<tonic::Response<edgegateway::DisconnectRouteBindingSourceResponse>, tonic::Status> {
        self.run_disconnect_route_binding_source(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Takes a stream of route bindings and answers once at the end.
    async fn ingest_route_binding_events(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::IngestRouteBindingEventsRequest>>,
    ) -> Result<tonic::Response<edgegateway::IngestRouteBindingEventsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = edgegateway::IngestRouteBindingEventsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_ingest_route_binding_events(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Replays a route binding.
    async fn replay_route_binding_events(
        &self,
        request: tonic::Request<edgegateway::ReplayRouteBindingEventsRequest>,
    ) -> Result<tonic::Response<edgegateway::ReplayRouteBindingEventsResponse>, tonic::Status> {
        self.run_replay_route_binding_events(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Acknowledges a route binding.
    async fn acknowledge_route_binding_event(
        &self,
        request: tonic::Request<edgegateway::AcknowledgeRouteBindingEventRequest>,
    ) -> Result<tonic::Response<edgegateway::AcknowledgeRouteBindingEventResponse>, tonic::Status> {
        self.run_acknowledge_route_binding_event(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type RelayRouteBindingMessagesStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::RelayRouteBindingMessagesResponse, tonic::Status>>;
    /// Exchanges changes of route bindings with a peer.
    async fn relay_route_binding_messages(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::RelayRouteBindingMessagesRequest>>,
    ) -> Result<tonic::Response<Self::RelayRouteBindingMessagesStream>, tonic::Status> {
        let handler = self.clone();
        let mut incoming = request.into_inner();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let received = tokio::select! { _ = sender.closed() => break, value = incoming.message() => value };
                let result = match received {
                    Ok(Some(request)) => handler.run_relay_route_binding_messages(request).await,
                    Ok(None) => break,
                    Err(error) => Err(error),
                };
                let failed = result.is_err();
                if sender.send(result).await.is_err() || failed {
                    break;
                }
            }
        });
        Ok(tonic::Response::new(tokio_stream::wrappers::ReceiverStream::new(
            receiver,
        )))
    }
    /// Lists route bindings a page at a time.
    async fn list_route_binding_sources(
        &self,
        request: tonic::Request<edgegateway::ListRouteBindingSourcesRequest>,
    ) -> Result<tonic::Response<edgegateway::ListRouteBindingSourcesResponse>, tonic::Status> {
        self.run_list_route_binding_sources(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Tests a route binding.
    async fn test_route_binding_connection(
        &self,
        request: tonic::Request<edgegateway::TestRouteBindingConnectionRequest>,
    ) -> Result<tonic::Response<edgegateway::TestRouteBindingConnectionResponse>, tonic::Status> {
        self.run_test_route_binding_connection(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
