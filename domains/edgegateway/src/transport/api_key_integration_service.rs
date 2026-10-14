use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::audit::v1::audit_event_validation_service_client::AuditEventValidationServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_query_service_client::RoleBindingQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_client::CertificateApprovalServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_integration_service_server::ApiKeyIntegrationService;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_client::ApiKeyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_client::RouteBindingIntegrationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::sponsorships::v1::sponsor_service_client::SponsorServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Connects external systems that send api key events.
#[derive(Clone)]
pub struct ApiKeyIntegrationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    acknowledge_api_key_event_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            edgegateway::AdvanceGatewayPolicyWorkflowRequest,
            edgegateway::AdvanceGatewayPolicyWorkflowResponse,
        >,
    >,
    acknowledge_api_key_event_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                certificates::SubmitCertificateRequest,
                certificates::SubmitCertificateResponse,
            >,
    >,
    acknowledge_api_key_event_2_port: acme_platform::ports::Callback<
        edgegateway::DisconnectRouteBindingSourceRequest,
        edgegateway::DisconnectRouteBindingSourceResponse,
    >,
    ingest_api_key_events_2_port: acme_platform::ports::Callback<
        edgegateway::ReplayRouteBindingEventsRequest,
        edgegateway::ReplayRouteBindingEventsResponse,
    >,
    list_api_key_sources_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<audit::PreviewAuditEventRequest, audit::PreviewAuditEventResponse>,
    >,
}
impl ApiKeyIntegrationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            edgegateway::ReplayRouteBindingEventsRequest,
            edgegateway::ReplayRouteBindingEventsResponse,
        > = {
            let client = RouteBindingIntegrationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::ReplayRouteBindingEventsRequest| -> acme_platform::ports::RpcFuture<edgegateway::ReplayRouteBindingEventsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.replay_route_binding_events(request).await?.into_inner())
})
})
        };
        let ingest_api_key_events_2_port = callback;
        let callback: acme_platform::ports::Callback<
            edgegateway::AdvanceGatewayPolicyWorkflowRequest,
            edgegateway::AdvanceGatewayPolicyWorkflowResponse,
        > = {
            let client = GatewayPolicyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::AdvanceGatewayPolicyWorkflowRequest| -> acme_platform::ports::RpcFuture<edgegateway::AdvanceGatewayPolicyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.advance_gateway_policy_workflow(request).await?.into_inner())
})
})
        };
        let acknowledge_api_key_event_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::SubmitCertificateRequest,
            certificates::SubmitCertificateResponse,
        > = {
            let client = CertificateApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::SubmitCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::SubmitCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.submit_certificate(request).await?.into_inner())
})
})
        };
        let acknowledge_api_key_event_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            edgegateway::DisconnectRouteBindingSourceRequest,
            edgegateway::DisconnectRouteBindingSourceResponse,
        > = {
            let client = RouteBindingIntegrationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::DisconnectRouteBindingSourceRequest| -> acme_platform::ports::RpcFuture<edgegateway::DisconnectRouteBindingSourceResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.disconnect_route_binding_source(request).await?.into_inner())
})
})
        };
        let acknowledge_api_key_event_2_port = callback;
        let callback: acme_platform::ports::Callback<
            audit::PreviewAuditEventRequest,
            audit::PreviewAuditEventResponse,
        > = {
            let client = AuditEventValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: audit::PreviewAuditEventRequest| -> acme_platform::ports::RpcFuture<audit::PreviewAuditEventResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_audit_event(request).await?.into_inner())
})
})
        };
        let list_api_key_sources_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            acknowledge_api_key_event_0_port,
            acknowledge_api_key_event_1_port,
            acknowledge_api_key_event_2_port,
            ingest_api_key_events_2_port,
            list_api_key_sources_0_port,
        }
    }
    /// Run ConnectApiKeySource after validating the tenant and durably recording the command.
    async fn run_connect_api_key_source(
        &self,
        req: edgegateway::ConnectApiKeySourceRequest,
    ) -> Result<edgegateway::ConnectApiKeySourceResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ConnectApiKeySource", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ConnectApiKeySource", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = venues::SummarizeVenuesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = edgegateway::AcknowledgeRouteBindingEventRequest {
            context: req.context.clone(),
            details: Some(edgegateway::acknowledge_route_binding_event_request::Details {
                received_at: req.details.clone().unwrap_or_default().received_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = VenueQueryServiceClient::new(channel_0);
                let input: tonic::Request<venues::SummarizeVenuesRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.summarize_venues(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = RouteBindingIntegrationServiceClient::new(channel_1);
                let input: tonic::Request<edgegateway::AcknowledgeRouteBindingEventRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.acknowledge_route_binding_event(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = audit::BrowseAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::browse_audit_events_request::Filter {
                created_after: reply_1
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
        let outgoing_2 = acme_platform::ports::RequestBuilder::<audit::BrowseAuditEventsRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AuditEventQueryServiceClient::new(channel);
            let input: tonic::Request<audit::BrowseAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_3 = ConnectApiKeySourceDispatch3::from(req.clone());
        command_3.api_key_audit_etag = reply_1
            .route_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        command_3.api_key_holder_display_name = reply_1
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        let outgoing_3: edgegateway::AdvanceApiKeyWorkflowRequest = command_3.into();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AdvanceApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = sponsorships::BatchCreateSponsorsRequest {
            context: req.context.clone(),
            chunk: Some(sponsorships::batch_create_sponsors_request::Chunk::Record(
                sponsorships::Sponsor {
                    audit: Some(common::AuditInfo {
                        created_at: reply_3
                            .api_key
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        ..Default::default()
                    }),
                    contact: Some(common::ContactInfo {
                        name: reply_3
                            .api_key
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        preferred_channel: reply_3
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = SponsorServiceClient::new(channel);
            let input: tonic::Request<sponsorships::BatchCreateSponsorsRequest> =
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
            let response = client.batch_create_sponsors(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = ratelimiting::StreamRateLimitChangesRequest {
            frame: Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(
                ratelimiting::RateLimit {
                    audit: Some(common::AuditInfo {
                        created_at: reply_1
                            .route_binding
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        etag: reply_1
                            .route_binding
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .etag
                            .clone(),
                        updated_at: reply_3
                            .api_key
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
        let reply_5 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamRateLimitChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_5);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_rate_limit_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamRateLimitChangesResponse::default();
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
        let mut response = edgegateway::ConnectApiKeySourceResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1
            .route_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
    /// Run DisconnectApiKeySource after validating the tenant and durably recording the command.
    async fn run_disconnect_api_key_source(
        &self,
        req: edgegateway::DisconnectApiKeySourceRequest,
    ) -> Result<edgegateway::DisconnectApiKeySourceResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DisconnectApiKeySource", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DisconnectApiKeySource", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::StartKeyRotationWorkflowRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                participants: vec![keymanagement::KeyRotationParticipant {
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
                    ..Default::default()
                }],
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = KeyRotationWorkflowServiceClient::new(channel);
            let input: tonic::Request<keymanagement::StartKeyRotationWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.start_key_rotation_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = keymanagement::ListPendingCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::list_pending_crypto_keys_request::Filter {
                created_after: req.details.clone().unwrap_or_default().received_at.clone(),
                region: reply_0
                    .key_rotation
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyApprovalServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ListPendingCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::ResumeGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            gateway_policy: Some(edgegateway::GatewayPolicy {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .key_rotation
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
                    user_id: reply_0
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
        let reply_2 = async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ResumeGatewayPolicyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_gateway_policy_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::DisconnectApiKeySourceResponse::default();
        let contribution: String = reply_0
            .key_rotation
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
    /// Run IngestApiKeyEvents after validating the tenant and durably recording the command.
    async fn run_ingest_api_key_events(
        &self,
        req: edgegateway::IngestApiKeyEventsRequest,
    ) -> Result<edgegateway::IngestApiKeyEventsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("IngestApiKeyEvents", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "IngestApiKeyEvents", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(edgegateway::ingest_api_key_events_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::ResumeGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            details: Some(edgegateway::resume_gateway_policy_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    preferred_channel: (match req.chunk.clone() {
                        Some(edgegateway::ingest_api_key_events_request::Chunk::Header(value)) => value,
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
            }),
            gateway_policy: Some(edgegateway::GatewayPolicy {
                owner: Some(identity::UserRef {
                    display_name: (match req.chunk.clone() {
                        Some(edgegateway::ingest_api_key_events_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .display_name
                    .clone(),
                    email: (match req.chunk.clone() {
                        Some(edgegateway::ingest_api_key_events_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = audit::ResolveAuditEventsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = GatewayPolicyWorkflowServiceClient::new(channel_0);
                let input: tonic::Request<edgegateway::ResumeGatewayPolicyWorkflowRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resume_gateway_policy_workflow(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = AuditEventQueryServiceClient::new(channel_1);
                let input: tonic::Request<audit::ResolveAuditEventsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_audit_events(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = edgegateway::ReplayRouteBindingEventsRequest {
            annotations: reply_0.gateway_policy.clone().unwrap_or_default().parameters.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::replay_route_binding_events_request::Details {
                message_id: reply_0.result.clone().unwrap_or_default().message.clone(),
                received_at: (match req.chunk.clone() {
                    Some(edgegateway::ingest_api_key_events_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
                ..Default::default()
            }),
            reason: (match reply_0.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let reply_2 = (self.ingest_api_key_events_2_port)(outgoing_2).await?;
        let mut response = edgegateway::IngestApiKeyEventsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run AcknowledgeApiKeyEvent after validating the tenant and durably recording the command.
    async fn run_acknowledge_api_key_event(
        &self,
        req: edgegateway::AcknowledgeApiKeyEventRequest,
    ) -> Result<edgegateway::AcknowledgeApiKeyEventResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("AcknowledgeApiKeyEvent", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AcknowledgeApiKeyEvent", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::AdvanceGatewayPolicyWorkflowRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            gateway_policy: Some(edgegateway::GatewayPolicy {
                audit: Some(common::AuditInfo {
                    updated_at: req.details.clone().unwrap_or_default().received_at.clone(),
                    ..Default::default()
                }),
                action: Some(edgegateway::gateway_policy::Action::Block(
                    edgegateway::GatewayPolicyBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.acknowledge_api_key_event_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = certificates::SubmitCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    updated_at: reply_0
                        .gateway_policy
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
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.acknowledge_api_key_event_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = edgegateway::DisconnectRouteBindingSourceRequest {
            context: req.context.clone(),
            details: Some(edgegateway::disconnect_route_binding_source_request::Details {
                received_at: req.details.clone().unwrap_or_default().received_at.clone(),
                source_system: req.details.clone().unwrap_or_default().source_system.clone(),
                ..Default::default()
            }),
            reason: (match reply_0.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let reply_2 = (self.acknowledge_api_key_event_2_port)(outgoing_2).await?;
        let mut response = edgegateway::AcknowledgeApiKeyEventResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
    /// Run RelayApiKeyMessages after validating the tenant and durably recording the command.
    async fn run_relay_api_key_messages(
        &self,
        req: edgegateway::RelayApiKeyMessagesRequest,
    ) -> Result<edgegateway::RelayApiKeyMessagesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RelayApiKeyMessages", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RelayApiKeyMessages", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value, _ => Default::default() }).holder.clone().unwrap_or_default().display_name.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ApproveCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                funding: Some(keymanagement::crypto_key::Funding::Card(
                    keymanagement::CryptoKeyCardFunding {
                        network: (match (match req.frame.clone() {
                            Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value,
                            _ => Default::default(),
                        })
                        .funding
                        .clone()
                        {
                            Some(edgegateway::api_key::Funding::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .network
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyApprovalServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ApproveCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.approve_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = audit::NormalizeAuditEventRequest {
            context: req.context.clone(),
            details: Some(audit::normalize_audit_event_request::Details {
                contact: Some(common::ContactInfo {
                    name: (match req.frame.clone() {
                        Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                    phone: (match req.frame.clone() {
                        Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                    preferred_channel: reply_0
                        .crypto_key
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
        let outgoing_1 = acme_platform::ports::RequestBuilder::<audit::NormalizeAuditEventRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AuditEventValidationServiceClient::new(channel);
            let input: tonic::Request<audit::NormalizeAuditEventRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_audit_event(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = RelayApiKeyMessagesDispatch2::from(req.clone());
        command_2.details_notify_email = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        let outgoing_2: edgegateway::SignalGatewayPolicyWorkflowRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SignalGatewayPolicyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_gateway_policy_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = edgegateway::ListApiKeyWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_api_key_workflows_request::Filter {
                created_after: (match req.frame.clone() {
                    Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
                labels: reply_0.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_1.audit_event.clone().unwrap_or_default().audit_event_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListApiKeyWorkflowsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_api_key_workflows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::RelayApiKeyMessagesResponse::default();
        let contribution: String = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
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
        let contribution: Option<prost_types::Timestamp> = reply_2
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_3.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = (match req.frame.clone() {
            Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .api_key_id
        .clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: i64 = (match req.frame.clone() {
            Some(edgegateway::relay_api_key_messages_request::Frame::Ack(value)) => value,
            _ => Default::default(),
        })
        .sequence
        .clone();
        if contribution != <i64>::default() {
            response.sequence = contribution;
        }
        Ok(response)
    }
    /// Run ListApiKeySources after validating the tenant and durably recording the command.
    async fn run_list_api_key_sources(
        &self,
        req: edgegateway::ListApiKeySourcesRequest,
    ) -> Result<edgegateway::ListApiKeySourcesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListApiKeySources", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListApiKeySources", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = audit::PreviewAuditEventRequest {
            context: req.context.clone(),
            details: Some(audit::preview_audit_event_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.list_api_key_sources_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = edgegateway::ListApiKeySourcesResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run TestApiKeyConnection after validating the tenant and durably recording the command.
    async fn run_test_api_key_connection(
        &self,
        req: edgegateway::TestApiKeyConnectionRequest,
    ) -> Result<edgegateway::TestApiKeyConnectionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TestApiKeyConnection", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TestApiKeyConnection", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ListKeyRotationWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::list_key_rotation_workflows_request::Filter {
                created_after: req.details.clone().unwrap_or_default().received_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req.details.clone().unwrap_or_default().message_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = authorization::ResolveRoleBindingsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = KeyRotationWorkflowServiceClient::new(channel_0);
                let input: tonic::Request<keymanagement::ListKeyRotationWorkflowsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_key_rotation_workflows(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = RoleBindingQueryServiceClient::new(channel_1);
                let input: tonic::Request<authorization::ResolveRoleBindingsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_role_bindings(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = edgegateway::TestApiKeyConnectionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ConnectApiKeySource for AdvanceApiKeyWorkflow.
struct ConnectApiKeySourceDispatch3 {
    context: Option<common::RequestContext>,
    api_key_audit_etag: String,
    api_key_holder_display_name: String,
}
impl From<edgegateway::ConnectApiKeySourceRequest> for ConnectApiKeySourceDispatch3 {
    fn from(req: edgegateway::ConnectApiKeySourceRequest) -> Self {
        Self {
            context: req.context.clone(),
            api_key_audit_etag: Default::default(),
            api_key_holder_display_name: Default::default(),
        }
    }
}
impl From<ConnectApiKeySourceDispatch3> for edgegateway::AdvanceApiKeyWorkflowRequest {
    fn from(value: ConnectApiKeySourceDispatch3) -> Self {
        edgegateway::AdvanceApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                audit: Some(common::AuditInfo {
                    etag: value.api_key_audit_etag,
                    ..Default::default()
                }),
                holder: Some(edgegateway::GatewayPolicyRef {
                    display_name: value.api_key_holder_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
/// The domain command prepared by RelayApiKeyMessages for SignalGatewayPolicyWorkflow.
struct RelayApiKeyMessagesDispatch2 {
    context: Option<common::RequestContext>,
    details_notify_email: String,
    gateway_policy_audit_created_by: String,
    gateway_policy_gateway_policy_id: String,
    details_notify_name: Option<common::PersonName>,
}
impl From<edgegateway::RelayApiKeyMessagesRequest> for RelayApiKeyMessagesDispatch2 {
    fn from(req: edgegateway::RelayApiKeyMessagesRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_notify_email: Default::default(),
            gateway_policy_audit_created_by: (match req.frame.clone() {
                Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value,
                _ => Default::default(),
            })
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone(),
            gateway_policy_gateway_policy_id: (match req.frame.clone() {
                Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value,
                _ => Default::default(),
            })
            .holder
            .clone()
            .unwrap_or_default()
            .gateway_policy_id
            .clone(),
            details_notify_name: (match req.frame.clone() {
                Some(edgegateway::relay_api_key_messages_request::Frame::Change(value)) => value,
                _ => Default::default(),
            })
            .contact
            .clone()
            .unwrap_or_default()
            .name
            .clone(),
        }
    }
}
impl From<RelayApiKeyMessagesDispatch2> for edgegateway::SignalGatewayPolicyWorkflowRequest {
    fn from(value: RelayApiKeyMessagesDispatch2) -> Self {
        edgegateway::SignalGatewayPolicyWorkflowRequest {
            context: value.context,
            details: Some(edgegateway::signal_gateway_policy_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: value.details_notify_email,
                    name: value.details_notify_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            gateway_policy: Some(edgegateway::GatewayPolicy {
                audit: Some(common::AuditInfo {
                    created_by: value.gateway_policy_audit_created_by,
                    ..Default::default()
                }),
                gateway_policy_id: value.gateway_policy_gateway_policy_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ApiKeyIntegrationService for ApiKeyIntegrationApi {
    /// Connects an api key.
    async fn connect_api_key_source(
        &self,
        request: tonic::Request<edgegateway::ConnectApiKeySourceRequest>,
    ) -> Result<tonic::Response<edgegateway::ConnectApiKeySourceResponse>, tonic::Status> {
        self.run_connect_api_key_source(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Disconnects an api key.
    async fn disconnect_api_key_source(
        &self,
        request: tonic::Request<edgegateway::DisconnectApiKeySourceRequest>,
    ) -> Result<tonic::Response<edgegateway::DisconnectApiKeySourceResponse>, tonic::Status> {
        self.run_disconnect_api_key_source(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Takes a stream of api keys and answers once at the end.
    async fn ingest_api_key_events(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::IngestApiKeyEventsRequest>>,
    ) -> Result<tonic::Response<edgegateway::IngestApiKeyEventsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = edgegateway::IngestApiKeyEventsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_ingest_api_key_events(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Acknowledges an api key.
    async fn acknowledge_api_key_event(
        &self,
        request: tonic::Request<edgegateway::AcknowledgeApiKeyEventRequest>,
    ) -> Result<tonic::Response<edgegateway::AcknowledgeApiKeyEventResponse>, tonic::Status> {
        self.run_acknowledge_api_key_event(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type RelayApiKeyMessagesStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::RelayApiKeyMessagesResponse, tonic::Status>>;
    /// Exchanges changes of api keys with a peer.
    async fn relay_api_key_messages(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::RelayApiKeyMessagesRequest>>,
    ) -> Result<tonic::Response<Self::RelayApiKeyMessagesStream>, tonic::Status> {
        let handler = self.clone();
        let mut incoming = request.into_inner();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let received = tokio::select! { _ = sender.closed() => break, value = incoming.message() => value };
                let result = match received {
                    Ok(Some(request)) => handler.run_relay_api_key_messages(request).await,
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
    /// Lists api keys a page at a time.
    async fn list_api_key_sources(
        &self,
        request: tonic::Request<edgegateway::ListApiKeySourcesRequest>,
    ) -> Result<tonic::Response<edgegateway::ListApiKeySourcesResponse>, tonic::Status> {
        self.run_list_api_key_sources(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Tests an api key.
    async fn test_api_key_connection(
        &self,
        request: tonic::Request<edgegateway::TestApiKeyConnectionRequest>,
    ) -> Result<tonic::Response<edgegateway::TestApiKeyConnectionResponse>, tonic::Status> {
        self.run_test_api_key_connection(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
