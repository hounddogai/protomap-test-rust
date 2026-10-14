use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_validation_service_client::AuditEventValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_query_service_client::ApiKeyQueryServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_client::ApiKeyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_gateway_service_server::RouteBindingGatewayService;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_client::RouteBindingIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_notification_service_client::RouteBindingNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_query_service_client::RouteBindingQueryServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_service_client::RouteBindingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_export_service_client::FeatureFlagExportServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_ring_workflow_service_client::KeyRingWorkflowServiceClient;
use acme_proto::acme::promotions::v1 as promotions;
use acme_proto::acme::promotions::v1::promotion_calculation_service_client::PromotionCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_approval_service_client::SecretApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_workflow_service_client::SecretWorkflowServiceClient;

/// The edge API that apps call for route bindings.
#[derive(Clone)]
pub struct RouteBindingGatewayHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    compose_route_binding_page_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            edgegateway::PauseApiKeyWorkflowRequest,
            edgegateway::PauseApiKeyWorkflowResponse,
        >,
    >,
    refresh_route_binding_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::ListFeatureFlagExportsRequest,
            featureflags::ListFeatureFlagExportsResponse,
        >,
    >,
    refresh_route_binding_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                keymanagement::PauseKeyRingWorkflowRequest,
                keymanagement::PauseKeyRingWorkflowResponse,
            >,
    >,
    start_route_binding_checkout_0_port:
        acme_platform::ports::Callback<edgegateway::PeekRouteBindingRequest, edgegateway::PeekRouteBindingResponse>,
    start_route_binding_checkout_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            promotions::RecalculatePromotionsRequest,
            promotions::RecalculatePromotionsResponse,
        >,
    >,
    stream_route_binding_updates_0_port:
        acme_platform::ports::Callback<edgegateway::CloneRouteBindingRequest, edgegateway::CloneRouteBindingResponse>,
    track_route_binding_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            audit::ExplainAuditEventDecisionRequest,
            audit::ExplainAuditEventDecisionResponse,
        >,
    >,
    track_route_binding_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::ShareRateLimitReportRequest,
                ratelimiting::ShareRateLimitReportResponse,
            >,
    >,
    upload_route_binding_attachments_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::RetryFeatureFlagExportRequest,
            featureflags::RetryFeatureFlagExportResponse,
        >,
    >,
}
impl RouteBindingGatewayHandler {
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
        let compose_route_binding_page_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            featureflags::ListFeatureFlagExportsRequest,
            featureflags::ListFeatureFlagExportsResponse,
        > = {
            let client = FeatureFlagExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ListFeatureFlagExportsRequest| -> acme_platform::ports::RpcFuture<featureflags::ListFeatureFlagExportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_feature_flag_exports(request).await?.into_inner())
})
})
        };
        let refresh_route_binding_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::PauseKeyRingWorkflowRequest,
            keymanagement::PauseKeyRingWorkflowResponse,
        > = {
            let client = KeyRingWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::PauseKeyRingWorkflowRequest| -> acme_platform::ports::RpcFuture<keymanagement::PauseKeyRingWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.pause_key_ring_workflow(request).await?.into_inner())
})
})
        };
        let refresh_route_binding_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            audit::ExplainAuditEventDecisionRequest,
            audit::ExplainAuditEventDecisionResponse,
        > = {
            let client = AuditEventValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: audit::ExplainAuditEventDecisionRequest| -> acme_platform::ports::RpcFuture<audit::ExplainAuditEventDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_audit_event_decision(request).await?.into_inner())
})
})
        };
        let track_route_binding_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::ShareRateLimitReportRequest,
            ratelimiting::ShareRateLimitReportResponse,
        > = {
            let client = RateLimitReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ShareRateLimitReportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ShareRateLimitReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_rate_limit_report(request).await?.into_inner())
})
})
        };
        let track_route_binding_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            edgegateway::CloneRouteBindingRequest,
            edgegateway::CloneRouteBindingResponse,
        > = {
            let client = RouteBindingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::CloneRouteBindingRequest| -> acme_platform::ports::RpcFuture<edgegateway::CloneRouteBindingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_route_binding(request).await?.into_inner())
})
})
        };
        let stream_route_binding_updates_0_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::RetryFeatureFlagExportRequest,
            featureflags::RetryFeatureFlagExportResponse,
        > = {
            let client = FeatureFlagExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::RetryFeatureFlagExportRequest| -> acme_platform::ports::RpcFuture<featureflags::RetryFeatureFlagExportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.retry_feature_flag_export(request).await?.into_inner())
})
})
        };
        let upload_route_binding_attachments_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
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
        let start_route_binding_checkout_0_port = callback;
        let callback: acme_platform::ports::Callback<
            promotions::RecalculatePromotionsRequest,
            promotions::RecalculatePromotionsResponse,
        > = {
            let client = PromotionCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: promotions::RecalculatePromotionsRequest| -> acme_platform::ports::RpcFuture<promotions::RecalculatePromotionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_promotions(request).await?.into_inner())
})
})
        };
        let start_route_binding_checkout_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            compose_route_binding_page_0_port,
            refresh_route_binding_0_port,
            refresh_route_binding_1_port,
            start_route_binding_checkout_0_port,
            start_route_binding_checkout_1_port,
            stream_route_binding_updates_0_port,
            track_route_binding_1_port,
            track_route_binding_2_port,
            upload_route_binding_attachments_1_port,
        }
    }
    /// Run FetchRouteBindingView after validating the tenant and durably recording the command.
    async fn run_fetch_route_binding_view(
        &self,
        req: edgegateway::FetchRouteBindingViewRequest,
    ) -> Result<edgegateway::FetchRouteBindingViewResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FetchRouteBindingView", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchRouteBindingView", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().route_binding_id.clone(), "recorded audit field");
        let command_0 = FetchRouteBindingViewDispatch0::from(req.clone());
        let outgoing_0: secrets::RejectSecretRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretApprovalServiceClient::new(channel);
            let input: tonic::Request<secrets::RejectSecretRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reject_secret(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::ListSecretWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(secrets::list_secret_workflows_request::Filter {
                created_after: reply_0
                    .secret
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_0.secret.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_0.outcome.clone().unwrap_or_default().reference_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::codec::through_any(&outgoing_1, "acme.secrets.v1.ListSecretWorkflowsRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SecretWorkflowServiceClient::new(channel);
            let input: tonic::Request<secrets::ListSecretWorkflowsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_secret_workflows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::IngestRouteBindingEventsRequest {
            context: req.context.clone(),
            chunk: Some(edgegateway::ingest_route_binding_events_request::Chunk::Header(
                edgegateway::ingest_route_binding_events_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
                        email: reply_0
                            .secret
                            .clone()
                            .unwrap_or_default()
                            .holder
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        phone: reply_0
                            .secret
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::IngestRouteBindingEventsRequest> =
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
            let response = client.ingest_route_binding_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::StreamRateLimitChangesRequest {
            frame: Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(
                ratelimiting::RateLimit {
                    action: Some(ratelimiting::rate_limit::Action::Discount(
                        ratelimiting::RateLimitDiscountAction {
                            percent: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                                Some(edgegateway::route_binding::Action::Discount(value)) => value,
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
        let mut response = edgegateway::FetchRouteBindingViewResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .secret
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: Option<identity::UserRef> = req.route_binding.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.route_binding.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run ComposeRouteBindingPage after validating the tenant and durably recording the command.
    async fn run_compose_route_binding_page(
        &self,
        req: edgegateway::ComposeRouteBindingPageRequest,
    ) -> Result<edgegateway::ComposeRouteBindingPageResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ComposeRouteBindingPage", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ComposeRouteBindingPage", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().route_binding_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::PauseApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                audit: Some(common::AuditInfo {
                    created_at: req
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
            }),
            context: req.context.clone(),
            details: Some(edgegateway::pause_api_key_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.compose_route_binding_page_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = ratelimiting::ResolveThrottleConflictRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::resolve_throttle_conflict_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ResolveThrottleConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_throttle_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::AdvanceQuotaBucketWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::advance_quota_bucket_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                funding: Some(ratelimiting::quota_bucket::Funding::Card(
                    ratelimiting::QuotaBucketCardFunding {
                        network: (match reply_0.api_key.clone().unwrap_or_default().funding.clone() {
                            Some(edgegateway::api_key::Funding::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .network
                        .clone(),
                        ..Default::default()
                    },
                )),
                holder: Some(ratelimiting::RateLimitRef {
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
            }),
            ..Default::default()
        };
        let outgoing_2 =
            acme_platform::ports::RequestBuilder::<ratelimiting::AdvanceQuotaBucketWorkflowRequest>::default()
                .map(|draft| *draft = outgoing_2)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::AdvanceQuotaBucketWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_quota_bucket_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::ComposeRouteBindingPageResponse::default();
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
        let contribution: String = reply_2
            .quota_bucket
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
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.route_binding_id.clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .route_binding_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.route_binding.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.route_binding.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run RefreshRouteBinding after validating the tenant and durably recording the command.
    async fn run_refresh_route_binding(
        &self,
        req: edgegateway::RefreshRouteBindingRequest,
    ) -> Result<edgegateway::RefreshRouteBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RefreshRouteBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RefreshRouteBinding", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().route_binding_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFeatureFlagExportsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_exports_request::Filter {
                created_after: req
                    .route_binding
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
        };
        let reply_0 = acme_platform::ports::execute(self.refresh_route_binding_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = keymanagement::PauseKeyRingWorkflowRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            updated_at: req
                                .route_binding
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.refresh_route_binding_1_port.as_ref(), outgoing_1).await?;
        let mut response = edgegateway::RefreshRouteBindingResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1
            .key_ring
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
        let contribution: String = req.route_binding.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.route_binding.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run SubmitRouteBindingForm after validating the tenant and durably recording the command.
    async fn run_submit_route_binding_form(
        &self,
        req: edgegateway::SubmitRouteBindingFormRequest,
    ) -> Result<edgegateway::SubmitRouteBindingFormResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitRouteBindingForm", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitRouteBindingForm", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().route_binding_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::StreamEnvironmentChangesRequest {
            frame: Some(configuration::stream_environment_changes_request::Frame::Change(
                configuration::Environment {
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
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamEnvironmentChangesRequest> =
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
            let response = client.stream_environment_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::StreamEnvironmentChangesResponse::default();
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
        let outgoing_1 = edgegateway::AcknowledgeRouteBindingEventRequest {
            context: req.context.clone(),
            reason: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::route_binding::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AcknowledgeRouteBindingEventRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.acknowledge_route_binding_event(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = edgegateway::AdvanceApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
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
                    ..Default::default()
                }),
                limits: vec![edgegateway::ApiKeyLimit {
                    amount: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                        Some(edgegateway::route_binding::Action::Discount(value)) => value,
                        _ => Default::default(),
                    })
                    .amount
                    .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(edgegateway::advance_api_key_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AdvanceApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::SubmitRouteBindingFormResponse::default();
        let contribution: i64 = reply_1.route_binding.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.route_binding.get_or_insert_with(Default::default).version = contribution;
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.route_binding_id.clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .route_binding_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.route_binding.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.route_binding.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run TrackRouteBinding after validating the tenant and durably recording the command.
    async fn run_track_route_binding(
        &self,
        req: edgegateway::TrackRouteBindingRequest,
    ) -> Result<edgegateway::TrackRouteBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TrackRouteBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TrackRouteBinding", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().route_binding_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::AdvanceApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .route_binding
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
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
                holder: Some(edgegateway::GatewayPolicyRef {
                    display_name: req
                        .route_binding
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: req.route_binding.clone().unwrap_or_default().version.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AdvanceApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = audit::ExplainAuditEventDecisionRequest {
            audit_event: Some(audit::AuditEvent {
                subject: Some(audit::AuditTrailRef {
                    display_name: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(audit::explain_audit_event_decision_request::Details {
                locale: req.details.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.track_route_binding_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = ratelimiting::ShareRateLimitReportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::share_rate_limit_report_request::Details {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.track_route_binding_2_port.as_ref(), outgoing_2).await?;
        let mut response = edgegateway::TrackRouteBindingResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.audit_event.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: String = req.route_binding_id.clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .route_binding_id = contribution;
        }
        let contribution: String = req.route_binding.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.route_binding.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run StreamRouteBindingUpdates after validating the tenant and durably recording the command.
    async fn run_stream_route_binding_updates(
        &self,
        req: edgegateway::StreamRouteBindingUpdatesRequest,
    ) -> Result<edgegateway::StreamRouteBindingUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamRouteBindingUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamRouteBindingUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::CloneRouteBindingRequest {
            context: req.context.clone(),
            route_binding: Some(edgegateway::RouteBinding {
                owner: Some(identity::UserRef {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.stream_route_binding_updates_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = edgegateway::UnsubscribeRouteBindingUpdatesRequest {
            context: req.context.clone(),
            details: Some(edgegateway::unsubscribe_route_binding_updates_request::Details {
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
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    phone: req.subscriber.clone().unwrap_or_default().phone.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: (match reply_0.route_binding.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::route_binding::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<edgegateway::UnsubscribeRouteBindingUpdatesRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::UnsubscribeRouteBindingUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.unsubscribe_route_binding_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ReconcileThrottlesRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::reconcile_throttles_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: reply_1
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ReconcileThrottlesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_throttles(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = edgegateway::SubscribeRouteBindingUpdatesRequest {
            context: req.context.clone(),
            details: Some(edgegateway::subscribe_route_binding_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    phone: req.subscriber.clone().unwrap_or_default().phone.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.edgegateway.v1.SubscribeRouteBindingUpdatesRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SubscribeRouteBindingUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_route_binding_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::StreamRouteBindingUpdatesResponse::default();
        let contribution: String = reply_0.route_binding.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.route_binding.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: i64 = reply_0.route_binding.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.route_binding.get_or_insert_with(Default::default).version = contribution;
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
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
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
        let contribution: i64 = reply_3.route_binding.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.route_binding.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run UploadRouteBindingAttachments after validating the tenant and durably recording the command.
    async fn run_upload_route_binding_attachments(
        &self,
        req: edgegateway::UploadRouteBindingAttachmentsRequest,
    ) -> Result<edgegateway::UploadRouteBindingAttachmentsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UploadRouteBindingAttachments", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UploadRouteBindingAttachments", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(edgegateway::upload_route_binding_attachments_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = configuration::ListEnvironmentReportsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_environment_reports_request::Filter {
                created_after: (match req.chunk.clone() {
                    Some(edgegateway::upload_route_binding_attachments_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
                labels: ((match req.chunk.clone() {
                    Some(edgegateway::upload_route_binding_attachments_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .parameters
                .clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
                owner_id: (match req.chunk.clone() {
                    Some(edgegateway::upload_route_binding_attachments_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ListEnvironmentReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_environment_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::RetryFeatureFlagExportRequest {
            context: req.context.clone(),
            details: Some(featureflags::retry_feature_flag_export_request::Details {
                notify: Some(common::ContactInfo {
                    phone: (match req.chunk.clone() {
                        Some(edgegateway::upload_route_binding_attachments_request::Chunk::Header(value)) => value,
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
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    updated_by: (match req.chunk.clone() {
                        Some(edgegateway::upload_route_binding_attachments_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_by
                    .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: (match req.chunk.clone() {
                        Some(edgegateway::upload_route_binding_attachments_request::Chunk::Header(value)) => value,
                        _ => Default::default(),
                    })
                    .submitted_by
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
        let reply_1 =
            self.upload_route_binding_attachments_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = edgegateway::TestRouteBindingConnectionRequest {
            context: req.context.clone(),
            details: Some(edgegateway::test_route_binding_connection_request::Details {
                message_id: reply_1.result.clone().unwrap_or_default().message.clone(),
                received_at: (match req.chunk.clone() {
                    Some(edgegateway::upload_route_binding_attachments_request::Chunk::Record(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
                ..Default::default()
            }),
            reason: (match (match req.chunk.clone() {
                Some(edgegateway::upload_route_binding_attachments_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .action
            .clone()
            {
                Some(edgegateway::route_binding::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            route_binding_id: (match req.chunk.clone() {
                Some(edgegateway::upload_route_binding_attachments_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .route_binding_id
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::TestRouteBindingConnectionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.test_route_binding_connection(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::UploadRouteBindingAttachmentsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        Ok(response)
    }
    /// Run GetRouteBindingSummary after validating the tenant and durably recording the command.
    async fn run_get_route_binding_summary(
        &self,
        req: edgegateway::GetRouteBindingSummaryRequest,
    ) -> Result<edgegateway::GetRouteBindingSummaryResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRouteBindingSummary", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRouteBindingSummary", "running workflow");
        let outgoing_0 = configuration::ResolveEnvironmentConflictRequest {
            context: req.context.clone(),
            details: Some(configuration::resolve_environment_conflict_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            environment: Some(configuration::Environment {
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    configuration::EnvironmentComponent {
                        component_id: req.route_binding_id.clone(),
                        ..Default::default()
                    },
                )]),
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
                specs: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::ResolveEnvironmentConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_environment_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::ResolveApiKeysRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ApiKeyQueryServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ResolveApiKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_api_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::GetRouteBindingSummaryResponse::default();
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .components
            .clone()
            .get("primary")
            .cloned()
            .unwrap_or_default()
            .description
            .clone();
        if contribution != <String>::default() {
            response.route_binding.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
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
    /// Run StartRouteBindingCheckout after validating the tenant and durably recording the command.
    async fn run_start_route_binding_checkout(
        &self,
        req: edgegateway::StartRouteBindingCheckoutRequest,
    ) -> Result<edgegateway::StartRouteBindingCheckoutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartRouteBindingCheckout", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartRouteBindingCheckout", "running workflow");
        tracing::debug!(audit_value = ?req.route_binding.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::PeekRouteBindingRequest {
            context: req.context.clone(),
            route_binding_id: req.route_binding.clone().unwrap_or_default().route_binding_id.clone(),
            ..Default::default()
        };
        let reply_0 = (self.start_route_binding_checkout_0_port)(outgoing_0).await?;
        let outgoing_1 = promotions::RecalculatePromotionsRequest {
            context: req.context.clone(),
            promotion: Some(promotions::Promotion {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .route_binding
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                action: Some(promotions::promotion::Action::Block(promotions::PromotionBlockAction {
                    notify_email: (match req.route_binding.clone().unwrap_or_default().action.clone() {
                        Some(edgegateway::route_binding::Action::Block(value)) => value,
                        _ => Default::default(),
                    })
                    .notify_email
                    .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            self.start_route_binding_checkout_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = edgegateway::StartRouteBindingCheckoutResponse::default();
        let contribution: Option<common::TimeWindow> =
            reply_0.route_binding.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.route_binding.get_or_insert_with(Default::default).effective = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .promotion
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
        let contribution: std::collections::BTreeMap<String, String> =
            req.route_binding.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
}
/// The domain command prepared by FetchRouteBindingView for RejectSecret.
struct FetchRouteBindingViewDispatch0 {
    context: Option<common::RequestContext>,
    secret_audit_updated_by: String,
    details_approver_display_name: String,
}
impl From<edgegateway::FetchRouteBindingViewRequest> for FetchRouteBindingViewDispatch0 {
    fn from(req: edgegateway::FetchRouteBindingViewRequest) -> Self {
        Self {
            context: req.context.clone(),
            secret_audit_updated_by: req
                .route_binding
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .updated_by
                .clone(),
            details_approver_display_name: req
                .route_binding
                .clone()
                .unwrap_or_default()
                .owner
                .clone()
                .unwrap_or_default()
                .display_name
                .clone(),
        }
    }
}
impl From<FetchRouteBindingViewDispatch0> for secrets::RejectSecretRequest {
    fn from(value: FetchRouteBindingViewDispatch0) -> Self {
        secrets::RejectSecretRequest {
            context: value.context,
            details: Some(secrets::reject_secret_request::Details {
                approver: Some(identity::UserRef {
                    display_name: value.details_approver_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret: Some(secrets::Secret {
                audit: Some(common::AuditInfo {
                    updated_by: value.secret_audit_updated_by,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl RouteBindingGatewayService for RouteBindingGatewayHandler {
    /// Fetchs a route binding.
    async fn fetch_route_binding_view(
        &self,
        request: tonic::Request<edgegateway::FetchRouteBindingViewRequest>,
    ) -> Result<tonic::Response<edgegateway::FetchRouteBindingViewResponse>, tonic::Status> {
        self.run_fetch_route_binding_view(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Composes a route binding.
    async fn compose_route_binding_page(
        &self,
        request: tonic::Request<edgegateway::ComposeRouteBindingPageRequest>,
    ) -> Result<tonic::Response<edgegateway::ComposeRouteBindingPageResponse>, tonic::Status> {
        self.run_compose_route_binding_page(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Refreshs a route binding.
    async fn refresh_route_binding(
        &self,
        request: tonic::Request<edgegateway::RefreshRouteBindingRequest>,
    ) -> Result<tonic::Response<edgegateway::RefreshRouteBindingResponse>, tonic::Status> {
        self.run_refresh_route_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Submits a route binding.
    async fn submit_route_binding_form(
        &self,
        request: tonic::Request<edgegateway::SubmitRouteBindingFormRequest>,
    ) -> Result<tonic::Response<edgegateway::SubmitRouteBindingFormResponse>, tonic::Status> {
        self.run_submit_route_binding_form(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Tracks a route binding.
    async fn track_route_binding(
        &self,
        request: tonic::Request<edgegateway::TrackRouteBindingRequest>,
    ) -> Result<tonic::Response<edgegateway::TrackRouteBindingResponse>, tonic::Status> {
        self.run_track_route_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamRouteBindingUpdatesStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::StreamRouteBindingUpdatesResponse, tonic::Status>>;
    /// Streams the changes of route bindings as they happen.
    async fn stream_route_binding_updates(
        &self,
        request: tonic::Request<edgegateway::StreamRouteBindingUpdatesRequest>,
    ) -> Result<tonic::Response<Self::StreamRouteBindingUpdatesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_route_binding_updates(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Takes a stream of route bindings and answers once at the end.
    async fn upload_route_binding_attachments(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::UploadRouteBindingAttachmentsRequest>>,
    ) -> Result<tonic::Response<edgegateway::UploadRouteBindingAttachmentsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = edgegateway::UploadRouteBindingAttachmentsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_upload_route_binding_attachments(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Returns a route binding.
    async fn get_route_binding_summary(
        &self,
        request: tonic::Request<edgegateway::GetRouteBindingSummaryRequest>,
    ) -> Result<tonic::Response<edgegateway::GetRouteBindingSummaryResponse>, tonic::Status> {
        self.run_get_route_binding_summary(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Starts a route binding.
    async fn start_route_binding_checkout(
        &self,
        request: tonic::Request<edgegateway::StartRouteBindingCheckoutRequest>,
    ) -> Result<tonic::Response<edgegateway::StartRouteBindingCheckoutResponse>, tonic::Status> {
        self.run_start_route_binding_checkout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
