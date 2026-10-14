use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::appointments::v1::appointment_query_service_client::AppointmentQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_service_client::CertificateServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_client::ApiKeyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_integration_service_server::GatewayPolicyIntegrationService;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_client::RouteBindingIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_notification_service_client::RouteBindingNotificationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_query_service_client::UserQueryServiceClient;
use acme_proto::acme::inventory::v1 as inventory;
use acme_proto::acme::inventory::v1::stock_item_query_service_client::StockItemQueryServiceClient;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_query_service_client::LedgerAccountQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_service_client::SearchIndexServiceClient;

/// Connects external systems that send gateway policy events.
#[derive(Clone)]
pub struct GatewayPolicyIntegrationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    acknowledge_gateway_policy_event_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            edgegateway::DisconnectRouteBindingSourceRequest,
            edgegateway::DisconnectRouteBindingSourceResponse,
        >,
    >,
    connect_gateway_policy_source_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                edgegateway::UnsubscribeRouteBindingUpdatesRequest,
                edgegateway::UnsubscribeRouteBindingUpdatesResponse,
            >,
    >,
    connect_gateway_policy_source_1_port: acme_platform::ports::Callback<
        edgegateway::CancelGatewayPolicyWorkflowRequest,
        edgegateway::CancelGatewayPolicyWorkflowResponse,
    >,
    list_gateway_policy_sources_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                edgegateway::ListApiKeyWorkflowsRequest,
                edgegateway::ListApiKeyWorkflowsResponse,
            >,
    >,
    replay_gateway_policy_events_0_port: acme_platform::ports::Callback<
        edgegateway::TestRouteBindingConnectionRequest,
        edgegateway::TestRouteBindingConnectionResponse,
    >,
}
impl GatewayPolicyIntegrationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            edgegateway::UnsubscribeRouteBindingUpdatesRequest,
            edgegateway::UnsubscribeRouteBindingUpdatesResponse,
        > = {
            let client = RouteBindingNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::UnsubscribeRouteBindingUpdatesRequest| -> acme_platform::ports::RpcFuture<edgegateway::UnsubscribeRouteBindingUpdatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.unsubscribe_route_binding_updates(request).await?.into_inner())
})
})
        };
        let connect_gateway_policy_source_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            edgegateway::CancelGatewayPolicyWorkflowRequest,
            edgegateway::CancelGatewayPolicyWorkflowResponse,
        > = {
            let client = GatewayPolicyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::CancelGatewayPolicyWorkflowRequest| -> acme_platform::ports::RpcFuture<edgegateway::CancelGatewayPolicyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.cancel_gateway_policy_workflow(request).await?.into_inner())
})
})
        };
        let connect_gateway_policy_source_1_port = callback;
        let callback: acme_platform::ports::Callback<
            edgegateway::TestRouteBindingConnectionRequest,
            edgegateway::TestRouteBindingConnectionResponse,
        > = {
            let client = RouteBindingIntegrationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::TestRouteBindingConnectionRequest| -> acme_platform::ports::RpcFuture<edgegateway::TestRouteBindingConnectionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.test_route_binding_connection(request).await?.into_inner())
})
})
        };
        let replay_gateway_policy_events_0_port = callback;
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
        let acknowledge_gateway_policy_event_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            edgegateway::ListApiKeyWorkflowsRequest,
            edgegateway::ListApiKeyWorkflowsResponse,
        > = {
            let client = ApiKeyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::ListApiKeyWorkflowsRequest| -> acme_platform::ports::RpcFuture<edgegateway::ListApiKeyWorkflowsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_api_key_workflows(request).await?.into_inner())
})
})
        };
        let list_gateway_policy_sources_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            acknowledge_gateway_policy_event_2_port,
            connect_gateway_policy_source_0_port,
            connect_gateway_policy_source_1_port,
            list_gateway_policy_sources_1_port,
            replay_gateway_policy_events_0_port,
        }
    }
    /// Run ConnectGatewayPolicySource after validating the tenant and durably recording the command.
    async fn run_connect_gateway_policy_source(
        &self,
        req: edgegateway::ConnectGatewayPolicySourceRequest,
    ) -> Result<edgegateway::ConnectGatewayPolicySourceResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ConnectGatewayPolicySource", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ConnectGatewayPolicySource", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::UnsubscribeRouteBindingUpdatesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.connect_gateway_policy_source_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = edgegateway::CancelGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            gateway_policy: Some(edgegateway::GatewayPolicy {
                owner: Some(identity::UserRef {
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
        let reply_1 = (self.connect_gateway_policy_source_1_port)(outgoing_1).await?;
        let outgoing_2 = certificates::BatchCreateCertificatesRequest {
            context: req.context.clone(),
            chunk: Some(certificates::batch_create_certificates_request::Chunk::Header(
                certificates::batch_create_certificates_request::StreamHeader {
                    source: reply_1
                        .gateway_policy
                        .clone()
                        .unwrap_or_default()
                        .gateway_policy_id
                        .clone(),
                    submitted_by: Some(common::ContactInfo {
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateServiceClient::new(channel);
            let input: tonic::Request<certificates::BatchCreateCertificatesRequest> =
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
            let response = client.batch_create_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::StreamRateLimitChangesRequest {
            frame: Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(
                ratelimiting::RateLimit {
                    audit: Some(common::AuditInfo {
                        created_at: reply_1
                            .gateway_policy
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        ..Default::default()
                    }),
                    action: Some(ratelimiting::rate_limit::Action::Block(
                        ratelimiting::RateLimitBlockAction {
                            notify_email: (match reply_1.gateway_policy.clone().unwrap_or_default().action.clone() {
                                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                                _ => Default::default(),
                            })
                            .notify_email
                            .clone(),
                            ..Default::default()
                        },
                    )),
                    owner: Some(identity::UserRef {
                        email: reply_0
                            .route_binding
                            .clone()
                            .unwrap_or_default()
                            .owner
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        user_id: reply_0
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamRateLimitChangesRequest> =
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
        let mut response = edgegateway::ConnectGatewayPolicySourceResponse::default();
        let contribution: String = reply_0
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = reply_0
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: i64 = reply_1.gateway_policy.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
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
    /// Run DisconnectGatewayPolicySource after validating the tenant and durably recording the command.
    async fn run_disconnect_gateway_policy_source(
        &self,
        req: edgegateway::DisconnectGatewayPolicySourceRequest,
    ) -> Result<edgegateway::DisconnectGatewayPolicySourceResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DisconnectGatewayPolicySource", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DisconnectGatewayPolicySource", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::UnsubscribeRouteBindingUpdatesRequest {
            context: req.context.clone(),
            details: Some(edgegateway::unsubscribe_route_binding_updates_request::Details {
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<edgegateway::UnsubscribeRouteBindingUpdatesRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::UnsubscribeRouteBindingUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.unsubscribe_route_binding_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = DisconnectGatewayPolicySourceDispatch1::from(req.clone());
        command_1.api_key_contact_email = reply_0
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        command_1.api_key_audit_created_at = reply_0
            .route_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        let outgoing_1: edgegateway::StartApiKeyWorkflowRequest = command_1.into();
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
        let outgoing_2 = edgegateway::ListGatewayPolicyWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_gateway_policy_workflows_request::Filter {
                created_after: reply_0
                    .route_binding
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListGatewayPolicyWorkflowsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_gateway_policy_workflows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::DisconnectGatewayPolicySourceResponse::default();
        let contribution: String = reply_0
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
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
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
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run IngestGatewayPolicyEvents after validating the tenant and durably recording the command.
    async fn run_ingest_gateway_policy_events(
        &self,
        req: edgegateway::IngestGatewayPolicyEventsRequest,
    ) -> Result<edgegateway::IngestGatewayPolicyEventsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("IngestGatewayPolicyEvents", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "IngestGatewayPolicyEvents", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(edgegateway::ingest_gateway_policy_events_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CheckFlagRolloutEligibilityRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: (match req.chunk.clone() {
                            Some(edgegateway::ingest_gateway_policy_events_request::Chunk::Record(value)) => value,
                            _ => Default::default(),
                        })
                        .owner
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<featureflags::CheckFlagRolloutEligibilityRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = FlagRolloutValidationServiceClient::new(channel);
                let input: tonic::Request<featureflags::CheckFlagRolloutEligibilityRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.check_flag_rollout_eligibility(input).await?;
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
        let outgoing_1 = appointments::SuggestAppointmentsRequest {
            context: req.context.clone(),
            filter: Some(appointments::suggest_appointments_request::Filter {
                created_after: reply_0
                    .flag_rollout
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                region: reply_0
                    .flag_rollout
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
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = AppointmentQueryServiceClient::new(channel);
            let input: tonic::Request<appointments::SuggestAppointmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_appointments(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = edgegateway::DisconnectRouteBindingSourceRequest {
            annotations: reply_1.result.clone().unwrap_or_default().details.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::disconnect_route_binding_source_request::Details {
                message_id: reply_1.result.clone().unwrap_or_default().message.clone(),
                received_at: reply_0
                    .flag_rollout
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                ..Default::default()
            }),
            reason: (match (match req.chunk.clone() {
                Some(edgegateway::ingest_gateway_policy_events_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .action
            .clone()
            {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::DisconnectRouteBindingSourceRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.disconnect_route_binding_source(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = search::CreateSearchIndexRequest {
            context: req.context.clone(),
            search_index: Some(search::SearchIndex {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: reply_2
                        .route_binding
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
                    email: (match req.chunk.clone() {
                        Some(edgegateway::ingest_gateway_policy_events_request::Chunk::Header(value)) => value,
                        _ => Default::default(),
                    })
                    .submitted_by
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                    user_id: reply_2
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = SearchIndexServiceClient::new(channel);
            let input: tonic::Request<search::CreateSearchIndexRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_search_index(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::IngestGatewayPolicyEventsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = (match reply_2.route_binding.clone().unwrap_or_default().action.clone() {
            Some(edgegateway::route_binding::Action::Block(value)) => value,
            _ => Default::default(),
        })
        .reason
        .clone();
        if contribution != <String>::default() {
            if response.rejections.is_empty() {
                response.rejections.push(Default::default());
            }
            response.rejections[0].reason = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run ReplayGatewayPolicyEvents after validating the tenant and durably recording the command.
    async fn run_replay_gateway_policy_events(
        &self,
        req: edgegateway::ReplayGatewayPolicyEventsRequest,
    ) -> Result<edgegateway::ReplayGatewayPolicyEventsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ReplayGatewayPolicyEvents", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReplayGatewayPolicyEvents", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().message_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::TestRouteBindingConnectionRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::test_route_binding_connection_request::Details {
                endpoint: req.details.clone().unwrap_or_default().endpoint.clone(),
                received_at: req.details.clone().unwrap_or_default().received_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.replay_gateway_policy_events_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = inventory::BrowseStockItemsRequest {
            context: req.context.clone(),
            filter: Some(inventory::browse_stock_items_request::Filter {
                created_after: reply_0.outcome.clone().unwrap_or_default().effective_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<inventory::BrowseStockItemsRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = StockItemQueryServiceClient::new(channel);
            let input: tonic::Request<inventory::BrowseStockItemsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_stock_items(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::ReplayGatewayPolicyEventsResponse::default();
        let contribution: Option<common::TimeWindow> =
            reply_0.route_binding.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).effective = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
    /// Run AcknowledgeGatewayPolicyEvent after validating the tenant and durably recording the command.
    async fn run_acknowledge_gateway_policy_event(
        &self,
        req: edgegateway::AcknowledgeGatewayPolicyEventRequest,
    ) -> Result<edgegateway::AcknowledgeGatewayPolicyEventResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AcknowledgeGatewayPolicyEvent", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AcknowledgeGatewayPolicyEvent", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let outgoing_0 = identity::LookupUserRequest {
            context: req.context.clone(),
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
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = UserQueryServiceClient::new(channel);
            let input: tonic::Request<identity::LookupUserRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lookup_user(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::ListRouteBindingSourcesRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_route_binding_sources_request::Filter {
                created_after: reply_0
                    .user
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: reply_0
                    .user
                    .clone()
                    .unwrap_or_default()
                    .relationship_manager
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: reply_0
                    .user
                    .clone()
                    .unwrap_or_default()
                    .primary_address
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
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListRouteBindingSourcesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_route_binding_sources(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::DisconnectRouteBindingSourceRequest {
            context: req.context.clone(),
            details: Some(edgegateway::disconnect_route_binding_source_request::Details {
                message_id: req.details.clone().unwrap_or_default().message_id.clone(),
                received_at: req.details.clone().unwrap_or_default().received_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 =
            acme_platform::ports::execute(self.acknowledge_gateway_policy_event_2_port.as_ref(), outgoing_2).await?;
        let mut response = edgegateway::AcknowledgeGatewayPolicyEventResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0
            .user
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
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
    /// Run RelayGatewayPolicyMessages after validating the tenant and durably recording the command.
    async fn run_relay_gateway_policy_messages(
        &self,
        req: edgegateway::RelayGatewayPolicyMessagesRequest,
    ) -> Result<edgegateway::RelayGatewayPolicyMessagesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RelayGatewayPolicyMessages", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RelayGatewayPolicyMessages", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(value)) => value, _ => Default::default() }).name.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamQuotaBucketMetricsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::stream_quota_bucket_metrics_request::Filter {
                created_after: (match req.frame.clone() {
                    Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
                owner_id: (match req.frame.clone() {
                    Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
                email: (match req.frame.clone() {
                    Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .owner
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
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_quota_bucket_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamQuotaBucketMetricsResponse::default();
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
        let outgoing_1 = ledger::ExportLedgerAccountsRequest {
            context: req.context.clone(),
            filter: Some(ledger::export_ledger_accounts_request::Filter {
                created_after: (match req.frame.clone() {
                    Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
                labels: (match req.frame.clone() {
                    Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .parameters
                .clone(),
                owner_id: (match req.frame.clone() {
                    Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .gateway_policy_id
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = LedgerAccountQueryServiceClient::new(channel);
            let input: tonic::Request<ledger::ExportLedgerAccountsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_ledger_accounts(input).await?;
            let mut frames = response.into_inner();
            let mut last = ledger::ExportLedgerAccountsResponse::default();
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
        let mut response = edgegateway::RelayGatewayPolicyMessagesResponse::default();
        let contribution: Option<common::TimeWindow> = (match req.frame.clone() {
            Some(edgegateway::relay_gateway_policy_messages_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .effective
        .clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).effective = contribution;
        }
        let contribution: String = (match req.frame.clone() {
            Some(edgegateway::relay_gateway_policy_messages_request::Frame::Ack(value)) => value,
            _ => Default::default(),
        })
        .status
        .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ListGatewayPolicySources after validating the tenant and durably recording the command.
    async fn run_list_gateway_policy_sources(
        &self,
        req: edgegateway::ListGatewayPolicySourcesRequest,
    ) -> Result<edgegateway::ListGatewayPolicySourcesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListGatewayPolicySources", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListGatewayPolicySources", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::SubscribeRouteBindingUpdatesRequest {
            annotations: (req.filter.clone().unwrap_or_default().labels.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(edgegateway::subscribe_route_binding_updates_request::Details {
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
                variables: (req.context.clone().unwrap_or_default().baggage.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
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
        let outgoing_1 = edgegateway::ListApiKeyWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_api_key_workflows_request::Filter {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_size: req.page.clone().unwrap_or_default().page_size.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            acme_platform::ports::execute(self.list_gateway_policy_sources_1_port.as_ref(), outgoing_1).await?;
        let mut response = edgegateway::ListGatewayPolicySourcesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_0.route_binding.clone().unwrap_or_default().scopes.clone();
        if contribution != <Vec<String>>::default() {
            if response.gateway_policies.is_empty() {
                response.gateway_policies.push(Default::default());
            }
            response.gateway_policies[0].scopes.extend(contribution);
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
        Ok(response)
    }
    /// Run TestGatewayPolicyConnection after validating the tenant and durably recording the command.
    async fn run_test_gateway_policy_connection(
        &self,
        req: edgegateway::TestGatewayPolicyConnectionRequest,
    ) -> Result<edgegateway::TestGatewayPolicyConnectionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("TestGatewayPolicyConnection", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TestGatewayPolicyConnection", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().endpoint.clone(), "recorded audit field");
        let command_0 = TestGatewayPolicyConnectionDispatch0::from(req.clone());
        let outgoing_0: edgegateway::AcknowledgeRouteBindingEventRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AcknowledgeRouteBindingEventRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.acknowledge_route_binding_event(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::InspectGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            gateway_policy_id: req.gateway_policy_id.clone(),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.edgegateway.v1.InspectGatewayPolicyWorkflowRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::InspectGatewayPolicyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.inspect_gateway_policy_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::TestGatewayPolicyConnectionResponse::default();
        let contribution: i64 = reply_0.route_binding.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = reply_1
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .gateway_policy_id
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        let contribution: String = reply_1.gateway_policy.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by DisconnectGatewayPolicySource for StartApiKeyWorkflow.
struct DisconnectGatewayPolicySourceDispatch1 {
    context: Option<common::RequestContext>,
    api_key_contact_email: String,
    api_key_audit_created_at: Option<prost_types::Timestamp>,
}
impl From<edgegateway::DisconnectGatewayPolicySourceRequest> for DisconnectGatewayPolicySourceDispatch1 {
    fn from(req: edgegateway::DisconnectGatewayPolicySourceRequest) -> Self {
        Self {
            context: req.context.clone(),
            api_key_contact_email: Default::default(),
            api_key_audit_created_at: Default::default(),
        }
    }
}
impl From<DisconnectGatewayPolicySourceDispatch1> for edgegateway::StartApiKeyWorkflowRequest {
    fn from(value: DisconnectGatewayPolicySourceDispatch1) -> Self {
        edgegateway::StartApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                audit: Some(common::AuditInfo {
                    created_at: value.api_key_audit_created_at,
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    email: value.api_key_contact_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
/// The domain command prepared by TestGatewayPolicyConnection for AcknowledgeRouteBindingEvent.
struct TestGatewayPolicyConnectionDispatch0 {
    context: Option<common::RequestContext>,
    annotations: std::collections::BTreeMap<String, String>,
}
impl From<edgegateway::TestGatewayPolicyConnectionRequest> for TestGatewayPolicyConnectionDispatch0 {
    fn from(req: edgegateway::TestGatewayPolicyConnectionRequest) -> Self {
        Self {
            context: req.context.clone(),
            annotations: req.annotations.clone(),
        }
    }
}
impl From<TestGatewayPolicyConnectionDispatch0> for edgegateway::AcknowledgeRouteBindingEventRequest {
    fn from(value: TestGatewayPolicyConnectionDispatch0) -> Self {
        edgegateway::AcknowledgeRouteBindingEventRequest {
            annotations: value.annotations,
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl GatewayPolicyIntegrationService for GatewayPolicyIntegrationEndpoint {
    /// Connects a gateway policy.
    async fn connect_gateway_policy_source(
        &self,
        request: tonic::Request<edgegateway::ConnectGatewayPolicySourceRequest>,
    ) -> Result<tonic::Response<edgegateway::ConnectGatewayPolicySourceResponse>, tonic::Status> {
        self.run_connect_gateway_policy_source(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Disconnects a gateway policy.
    async fn disconnect_gateway_policy_source(
        &self,
        request: tonic::Request<edgegateway::DisconnectGatewayPolicySourceRequest>,
    ) -> Result<tonic::Response<edgegateway::DisconnectGatewayPolicySourceResponse>, tonic::Status> {
        self.run_disconnect_gateway_policy_source(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Takes a stream of gateway policies and answers once at the end.
    async fn ingest_gateway_policy_events(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::IngestGatewayPolicyEventsRequest>>,
    ) -> Result<tonic::Response<edgegateway::IngestGatewayPolicyEventsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = edgegateway::IngestGatewayPolicyEventsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_ingest_gateway_policy_events(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Replays a gateway policy.
    async fn replay_gateway_policy_events(
        &self,
        request: tonic::Request<edgegateway::ReplayGatewayPolicyEventsRequest>,
    ) -> Result<tonic::Response<edgegateway::ReplayGatewayPolicyEventsResponse>, tonic::Status> {
        self.run_replay_gateway_policy_events(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Acknowledges a gateway policy.
    async fn acknowledge_gateway_policy_event(
        &self,
        request: tonic::Request<edgegateway::AcknowledgeGatewayPolicyEventRequest>,
    ) -> Result<tonic::Response<edgegateway::AcknowledgeGatewayPolicyEventResponse>, tonic::Status> {
        self.run_acknowledge_gateway_policy_event(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type RelayGatewayPolicyMessagesStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::RelayGatewayPolicyMessagesResponse, tonic::Status>>;
    /// Exchanges changes of gateway policies with a peer.
    async fn relay_gateway_policy_messages(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::RelayGatewayPolicyMessagesRequest>>,
    ) -> Result<tonic::Response<Self::RelayGatewayPolicyMessagesStream>, tonic::Status> {
        let handler = self.clone();
        let mut incoming = request.into_inner();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let received = tokio::select! { _ = sender.closed() => break, value = incoming.message() => value };
                let result = match received {
                    Ok(Some(request)) => handler.run_relay_gateway_policy_messages(request).await,
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
    /// Lists gateway policies a page at a time.
    async fn list_gateway_policy_sources(
        &self,
        request: tonic::Request<edgegateway::ListGatewayPolicySourcesRequest>,
    ) -> Result<tonic::Response<edgegateway::ListGatewayPolicySourcesResponse>, tonic::Status> {
        self.run_list_gateway_policy_sources(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Tests a gateway policy.
    async fn test_gateway_policy_connection(
        &self,
        request: tonic::Request<edgegateway::TestGatewayPolicyConnectionRequest>,
    ) -> Result<tonic::Response<edgegateway::TestGatewayPolicyConnectionResponse>, tonic::Status> {
        self.run_test_gateway_policy_connection(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
