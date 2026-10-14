use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_validation_service_client::AuditEventValidationServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_client::CertificateApprovalServiceClient;
use acme_proto::acme::certificates::v1::csr_approval_service_client::CsrApprovalServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_server::GatewayPolicyWorkflowService;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_client::RouteBindingIntegrationServiceClient;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::fanportal::v1::fan_session_service_client::FanSessionServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_client::KeyRingReportingServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::reviews::v1 as reviews;
use acme_proto::acme::reviews::v1::product_review_service_client::ProductReviewServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_validation_service_client::SearchIndexValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient;

/// Runs the long-lived workflow that takes a gateway policy from start to finish.
#[derive(Clone)]
pub struct GatewayPolicyWorkflowEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    advance_gateway_policy_workflow_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                featureflags::ExplainFlagEvaluationDecisionRequest,
                featureflags::ExplainFlagEvaluationDecisionResponse,
            >,
    >,
    cancel_gateway_policy_workflow_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            certificates::ApproveCertificateRequest,
            certificates::ApproveCertificateResponse,
        >,
    >,
    resume_gateway_policy_workflow_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            edgegateway::FetchGatewayPolicyReportRequest,
            edgegateway::FetchGatewayPolicyReportResponse,
        >,
    >,
    resume_gateway_policy_workflow_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::PauseRateLimitWorkflowRequest,
                ratelimiting::PauseRateLimitWorkflowResponse,
            >,
    >,
    resume_gateway_policy_workflow_2_port:
        acme_platform::ports::Callback<audit::ValidateAuditEventRequest, audit::ValidateAuditEventResponse>,
    resume_gateway_policy_workflow_3_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::RepairFlagEvaluationRequest,
            featureflags::RepairFlagEvaluationResponse,
        >,
    >,
}
impl GatewayPolicyWorkflowEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::ExplainFlagEvaluationDecisionRequest,
            featureflags::ExplainFlagEvaluationDecisionResponse,
        > = {
            let client = FlagEvaluationValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ExplainFlagEvaluationDecisionRequest| -> acme_platform::ports::RpcFuture<featureflags::ExplainFlagEvaluationDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_flag_evaluation_decision(request).await?.into_inner())
})
})
        };
        let advance_gateway_policy_workflow_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::ApproveCertificateRequest,
            certificates::ApproveCertificateResponse,
        > = {
            let client = CertificateApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::ApproveCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::ApproveCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.approve_certificate(request).await?.into_inner())
})
})
        };
        let cancel_gateway_policy_workflow_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            edgegateway::FetchGatewayPolicyReportRequest,
            edgegateway::FetchGatewayPolicyReportResponse,
        > = {
            let client = GatewayPolicyReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::FetchGatewayPolicyReportRequest| -> acme_platform::ports::RpcFuture<edgegateway::FetchGatewayPolicyReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.fetch_gateway_policy_report(request).await?.into_inner())
})
})
        };
        let resume_gateway_policy_workflow_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::PauseRateLimitWorkflowRequest,
            ratelimiting::PauseRateLimitWorkflowResponse,
        > = {
            let client = RateLimitWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::PauseRateLimitWorkflowRequest| -> acme_platform::ports::RpcFuture<ratelimiting::PauseRateLimitWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.pause_rate_limit_workflow(request).await?.into_inner())
})
})
        };
        let resume_gateway_policy_workflow_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            audit::ValidateAuditEventRequest,
            audit::ValidateAuditEventResponse,
        > = {
            let client = AuditEventValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: audit::ValidateAuditEventRequest| -> acme_platform::ports::RpcFuture<audit::ValidateAuditEventResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.validate_audit_event(request).await?.into_inner())
})
})
        };
        let resume_gateway_policy_workflow_2_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::RepairFlagEvaluationRequest,
            featureflags::RepairFlagEvaluationResponse,
        > = {
            let client = FlagEvaluationAdminServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::RepairFlagEvaluationRequest| -> acme_platform::ports::RpcFuture<featureflags::RepairFlagEvaluationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.repair_flag_evaluation(request).await?.into_inner())
})
})
        };
        let resume_gateway_policy_workflow_3_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            advance_gateway_policy_workflow_2_port,
            cancel_gateway_policy_workflow_0_port,
            resume_gateway_policy_workflow_0_port,
            resume_gateway_policy_workflow_1_port,
            resume_gateway_policy_workflow_2_port,
            resume_gateway_policy_workflow_3_port,
        }
    }
    /// Run StartGatewayPolicyWorkflow after validating the tenant and durably recording the command.
    async fn run_start_gateway_policy_workflow(
        &self,
        req: edgegateway::StartGatewayPolicyWorkflowRequest,
    ) -> Result<edgegateway::StartGatewayPolicyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartGatewayPolicyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartGatewayPolicyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy.clone().unwrap_or_default().gateway_policy_id.clone(), "recorded audit field");
        let outgoing_0 = search::ScoreSearchIndexRequest {
            context: req.context.clone(),
            search_index: Some(search::SearchIndex {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .gateway_policy
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: req
                        .gateway_policy
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    search::SearchIndexComponent {
                        description: req.gateway_policy.clone().unwrap_or_default().description.clone(),
                        ..Default::default()
                    },
                )]),
                owner: Some(identity::UserRef {
                    user_id: req
                        .gateway_policy
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
        let reply_0 = async move {
            let mut client = SearchIndexValidationServiceClient::new(channel);
            let input: tonic::Request<search::ScoreSearchIndexRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_search_index(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::CancelSecretVersionWorkflowRequest {
            context: req.context.clone(),
            details: Some(secrets::cancel_secret_version_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: reply_0
                        .search_index
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
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret_version: Some(secrets::SecretVersion {
                measurements: vec![secrets::SecretVersionMeasurement {
                    unit: reply_0
                        .search_index
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .unit
                        .clone(),
                    ..Default::default()
                }],
                subject: Some(secrets::AccessGrantRef {
                    version: req.gateway_policy.clone().unwrap_or_default().version.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SecretVersionWorkflowServiceClient::new(channel);
            let input: tonic::Request<secrets::CancelSecretVersionWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.cancel_secret_version_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::ResolveFeatureFlagConflictRequest {
            context: req.context.clone(),
            details: Some(featureflags::resolve_feature_flag_conflict_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .search_index
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: reply_0
                        .search_index
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: req
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ResolveFeatureFlagConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_feature_flag_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::BatchCreateRateLimitsRequest {
            context: req.context.clone(),
            chunk: Some(ratelimiting::batch_create_rate_limits_request::Chunk::Record(
                ratelimiting::RateLimit {
                    audit: Some(common::AuditInfo {
                        created_at: reply_2
                            .feature_flag
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = RateLimitServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::BatchCreateRateLimitsRequest> =
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
            let response = client.batch_create_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = ratelimiting::StreamThrottleChangesRequest {
            frame: Some(ratelimiting::stream_throttle_changes_request::Frame::Change(
                ratelimiting::Throttle {
                    subject: Some(ratelimiting::RateLimitRef {
                        version: reply_1
                            .secret_version
                            .clone()
                            .unwrap_or_default()
                            .subject
                            .clone()
                            .unwrap_or_default()
                            .version
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
        let mut response = edgegateway::StartGatewayPolicyWorkflowResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: i32 = req.gateway_policy.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceGatewayPolicyWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_gateway_policy_workflow(
        &self,
        req: edgegateway::AdvanceGatewayPolicyWorkflowRequest,
    ) -> Result<edgegateway::AdvanceGatewayPolicyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceGatewayPolicyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceGatewayPolicyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy.clone().unwrap_or_default().gateway_policy_id.clone(), "recorded audit field");
        let outgoing_0 = fanportal::CreateFanSessionRequest {
            context: req.context.clone(),
            fan_session: Some(fanportal::FanSession {
                assignee: Some(identity::UserRef {
                    display_name: req
                        .gateway_policy
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                requester: Some(common::ContactInfo {
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FanSessionServiceClient::new(channel);
            let input: tonic::Request<fanportal::CreateFanSessionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_fan_session(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::ListPendingCsrsRequest {
            context: req.context.clone(),
            filter: Some(certificates::list_pending_csrs_request::Filter {
                created_after: req
                    .gateway_policy
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: (req.gateway_policy.clone().unwrap_or_default().parameters.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: reply_0.fan_session.clone().unwrap_or_default().fan_session_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CsrApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::ListPendingCsrsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_csrs(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::ExplainFlagEvaluationDecisionRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_flag_evaluation_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .gateway_policy
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
        let reply_2 =
            acme_platform::ports::execute(self.advance_gateway_policy_workflow_2_port.as_ref(), outgoing_2).await?;
        let jobs_3 = req
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .conditions
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_3 = featureflags::PushFeatureFlagRequest {
                        context: req.context.clone(),
                        feature_flag: Some(featureflags::FeatureFlag {
                            conditions: vec![featureflags::FeatureFlagCondition {
                                attribute: item.attribute.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = FeatureFlagSyncServiceClient::new(channel);
                        let input: tonic::Request<featureflags::PushFeatureFlagRequest> =
                            tonic::IntoRequest::into_request(outgoing_3);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.push_feature_flag(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_3).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_3 = replies.into_iter().last().unwrap_or_default();
        let mut response = edgegateway::AdvanceGatewayPolicyWorkflowResponse::default();
        let contribution: i32 = reply_0.fan_session.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).priority = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_2.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.gateway_policy.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CancelGatewayPolicyWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_gateway_policy_workflow(
        &self,
        req: edgegateway::CancelGatewayPolicyWorkflowRequest,
    ) -> Result<edgegateway::CancelGatewayPolicyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelGatewayPolicyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelGatewayPolicyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = certificates::ApproveCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .gateway_policy
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_at: req
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
                counterparty: Some(certificates::CsrRef {
                    display_name: req
                        .gateway_policy
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                lines: vec![certificates::CertificateLine {
                    tax: Some(certificates::CertificateLineTax {
                        amount: (match req.gateway_policy.clone().unwrap_or_default().action.clone() {
                            Some(edgegateway::gateway_policy::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .amount
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
        let reply_0 =
            self.cancel_gateway_policy_workflow_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = edgegateway::AcknowledgeRouteBindingEventRequest {
            context: req.context.clone(),
            reason: (match req.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AcknowledgeRouteBindingEventRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.acknowledge_route_binding_event(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::DeleteGatewayPolicyReportRequest {
            context: req.context.clone(),
            etag: req
                .gateway_policy
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .etag
                .clone(),
            gateway_policy_id: req.gateway_policy.clone().unwrap_or_default().gateway_policy_id.clone(),
            reason: (match req.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let outgoing_2 =
            acme_platform::ports::RequestBuilder::<edgegateway::DeleteGatewayPolicyReportRequest>::default()
                .map(|draft| *draft = outgoing_2)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::DeleteGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.delete_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_3 = CancelGatewayPolicyWorkflowDispatch3::from(req.clone());
        command_3.details_contact_email = reply_1
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        command_3.flag_evaluation_subject_display_name = reply_1
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        let outgoing_3: featureflags::LintFlagEvaluationsRequest = command_3.into();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FlagEvaluationValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::LintFlagEvaluationsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_flag_evaluations(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::CancelGatewayPolicyWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_1.route_binding.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_3.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        let contribution: String = req.gateway_policy.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run RetryGatewayPolicyWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_gateway_policy_workflow(
        &self,
        req: edgegateway::RetryGatewayPolicyWorkflowRequest,
    ) -> Result<edgegateway::RetryGatewayPolicyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryGatewayPolicyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryGatewayPolicyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy.clone().unwrap_or_default().gateway_policy_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::StreamGatewayPolicyMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .gateway_policy
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                phone: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::StreamGatewayPolicyMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_gateway_policy_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::StreamGatewayPolicyMetricsResponse::default();
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
        let outgoing_1 = reviews::SearchProductReviewsRequest {
            context: req.context.clone(),
            filter: Some(reviews::search_product_reviews_request::Filter {
                created_after: req
                    .gateway_policy
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: req
                    .gateway_policy
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
        let outgoing_1 = acme_platform::ports::RequestBuilder::<reviews::SearchProductReviewsRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ProductReviewServiceClient::new(channel);
            let input: tonic::Request<reviews::SearchProductReviewsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_product_reviews(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_2 = RetryGatewayPolicyWorkflowDispatch2::from(req.clone());
        let outgoing_2: edgegateway::FetchGatewayPolicyReportRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::FetchGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.fetch_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = edgegateway::ListGatewayPolicyReportsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_gateway_policy_reports_request::Filter {
                created_after: req
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
            ..Default::default()
        };
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.edgegateway.v1.ListGatewayPolicyReportsRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListGatewayPolicyReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_gateway_policy_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::RetryGatewayPolicyWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
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
        let contribution: i32 = req.gateway_policy.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run PauseGatewayPolicyWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_gateway_policy_workflow(
        &self,
        req: edgegateway::PauseGatewayPolicyWorkflowRequest,
    ) -> Result<edgegateway::PauseGatewayPolicyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PauseGatewayPolicyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseGatewayPolicyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy.clone().unwrap_or_default().gateway_policy_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::FetchGatewayPolicyReportRequest {
            context: req.context.clone(),
            gateway_policy_id: req.gateway_policy_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::FetchGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.fetch_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::CountGatewayPolicyTotalsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::count_gateway_policy_totals_request::Filter {
                created_after: req
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::CountGatewayPolicyTotalsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_gateway_policy_totals(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::PauseGatewayPolicyWorkflowResponse::default();
        let contribution: String = reply_0.gateway_policy.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).description = contribution;
        }
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
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        let contribution: i32 = req.gateway_policy.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run ResumeGatewayPolicyWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_gateway_policy_workflow(
        &self,
        req: edgegateway::ResumeGatewayPolicyWorkflowRequest,
    ) -> Result<edgegateway::ResumeGatewayPolicyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeGatewayPolicyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeGatewayPolicyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::FetchGatewayPolicyReportRequest {
            context: req.context.clone(),
            gateway_policy_id: req.gateway_policy.clone().unwrap_or_default().gateway_policy_id.clone(),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.resume_gateway_policy_workflow_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = ratelimiting::PauseRateLimitWorkflowRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
                    user_id: reply_0
                        .gateway_policy
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
        let reply_1 =
            acme_platform::ports::execute(self.resume_gateway_policy_workflow_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = audit::ValidateAuditEventRequest {
            audit_event: Some(audit::AuditEvent {
                subject: Some(audit::AuditTrailRef {
                    display_name: reply_0
                        .gateway_policy
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
            context: req.context.clone(),
            details: Some(audit::validate_audit_event_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .gateway_policy
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
        let reply_2 = (self.resume_gateway_policy_workflow_2_port)(outgoing_2).await?;
        let outgoing_3 = featureflags::RepairFlagEvaluationRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                correlation_id: reply_2.result.clone().unwrap_or_default().correlation_id.clone(),
                occurred_at: reply_2.audit_event.clone().unwrap_or_default().occurred_at.clone(),
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: reply_0
                        .gateway_policy
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
            reason: (match req.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let reply_3 =
            self.resume_gateway_policy_workflow_3_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_3)
            .await?;
        let mut response = edgegateway::ResumeGatewayPolicyWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .subject
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
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.details.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run InspectGatewayPolicyWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_gateway_policy_workflow(
        &self,
        req: edgegateway::InspectGatewayPolicyWorkflowRequest,
    ) -> Result<edgegateway::InspectGatewayPolicyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectGatewayPolicyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectGatewayPolicyWorkflow", "running workflow");
        let outgoing_0 = edgegateway::ListGatewayPolicyReportsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_gateway_policy_reports_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.gateway_policy_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.edgegateway.v1.ListGatewayPolicyReportsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListGatewayPolicyReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_gateway_policy_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::FetchGatewayPolicyReportRequest {
            context: req.context.clone(),
            gateway_policy_id: req.gateway_policy_id.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<edgegateway::FetchGatewayPolicyReportRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = GatewayPolicyReportingServiceClient::new(channel);
                let input: tonic::Request<edgegateway::FetchGatewayPolicyReportRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.fetch_gateway_policy_report(input).await?;
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
        let mut response = edgegateway::InspectGatewayPolicyWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: i32 = reply_1.gateway_policy.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::TimeWindow> =
            reply_1.gateway_policy.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).effective = contribution;
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
    /// Run ListGatewayPolicyWorkflows after validating the tenant and durably recording the command.
    async fn run_list_gateway_policy_workflows(
        &self,
        req: edgegateway::ListGatewayPolicyWorkflowsRequest,
    ) -> Result<edgegateway::ListGatewayPolicyWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListGatewayPolicyWorkflows", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListGatewayPolicyWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::GenerateGatewayPolicyReportRequest {
            context: req.context.clone(),
            details: Some(edgegateway::generate_gateway_policy_report_request::Details {
                filters: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::GenerateGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = edgegateway::ListGatewayPolicyWorkflowsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run SignalGatewayPolicyWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_gateway_policy_workflow(
        &self,
        req: edgegateway::SignalGatewayPolicyWorkflowRequest,
    ) -> Result<edgegateway::SignalGatewayPolicyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalGatewayPolicyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalGatewayPolicyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy.clone().unwrap_or_default().gateway_policy_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ShareKeyRingReportRequest {
            context: req.context.clone(),
            details: Some(keymanagement::share_key_ring_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::ShareKeyRingReportRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = KeyRingReportingServiceClient::new(channel);
                let input: tonic::Request<keymanagement::ShareKeyRingReportRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.share_key_ring_report(input).await?;
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
        let outgoing_1 = featureflags::ExplainFlagEvaluationCalculationRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_flag_evaluation_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: reply_0
                        .key_ring
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
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = FlagEvaluationCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExplainFlagEvaluationCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_flag_evaluation_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = edgegateway::SignalGatewayPolicyWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0
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
                .gateway_policy
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.gateway_policy.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CancelGatewayPolicyWorkflow for LintFlagEvaluations.
struct CancelGatewayPolicyWorkflowDispatch3 {
    context: Option<common::RequestContext>,
    details_contact_email: String,
    flag_evaluation_subject_display_name: String,
}
impl From<edgegateway::CancelGatewayPolicyWorkflowRequest> for CancelGatewayPolicyWorkflowDispatch3 {
    fn from(req: edgegateway::CancelGatewayPolicyWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_contact_email: Default::default(),
            flag_evaluation_subject_display_name: Default::default(),
        }
    }
}
impl From<CancelGatewayPolicyWorkflowDispatch3> for featureflags::LintFlagEvaluationsRequest {
    fn from(value: CancelGatewayPolicyWorkflowDispatch3) -> Self {
        featureflags::LintFlagEvaluationsRequest {
            context: value.context,
            details: Some(featureflags::lint_flag_evaluations_request::Details {
                contact: Some(common::ContactInfo {
                    email: value.details_contact_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: value.flag_evaluation_subject_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by RetryGatewayPolicyWorkflow for FetchGatewayPolicyReport.
struct RetryGatewayPolicyWorkflowDispatch2 {
    context: Option<common::RequestContext>,
    gateway_policy_id: String,
}
impl From<edgegateway::RetryGatewayPolicyWorkflowRequest> for RetryGatewayPolicyWorkflowDispatch2 {
    fn from(req: edgegateway::RetryGatewayPolicyWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            gateway_policy_id: req.gateway_policy_id.clone(),
        }
    }
}
impl From<RetryGatewayPolicyWorkflowDispatch2> for edgegateway::FetchGatewayPolicyReportRequest {
    fn from(value: RetryGatewayPolicyWorkflowDispatch2) -> Self {
        edgegateway::FetchGatewayPolicyReportRequest {
            context: value.context,
            gateway_policy_id: value.gateway_policy_id,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl GatewayPolicyWorkflowService for GatewayPolicyWorkflowEndpoint {
    /// Starts a gateway policy.
    async fn start_gateway_policy_workflow(
        &self,
        request: tonic::Request<edgegateway::StartGatewayPolicyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::StartGatewayPolicyWorkflowResponse>, tonic::Status> {
        self.run_start_gateway_policy_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances a gateway policy.
    async fn advance_gateway_policy_workflow(
        &self,
        request: tonic::Request<edgegateway::AdvanceGatewayPolicyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::AdvanceGatewayPolicyWorkflowResponse>, tonic::Status> {
        self.run_advance_gateway_policy_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a gateway policy.
    async fn cancel_gateway_policy_workflow(
        &self,
        request: tonic::Request<edgegateway::CancelGatewayPolicyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::CancelGatewayPolicyWorkflowResponse>, tonic::Status> {
        self.run_cancel_gateway_policy_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a gateway policy.
    async fn retry_gateway_policy_workflow(
        &self,
        request: tonic::Request<edgegateway::RetryGatewayPolicyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::RetryGatewayPolicyWorkflowResponse>, tonic::Status> {
        self.run_retry_gateway_policy_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses a gateway policy.
    async fn pause_gateway_policy_workflow(
        &self,
        request: tonic::Request<edgegateway::PauseGatewayPolicyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::PauseGatewayPolicyWorkflowResponse>, tonic::Status> {
        self.run_pause_gateway_policy_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes a gateway policy.
    async fn resume_gateway_policy_workflow(
        &self,
        request: tonic::Request<edgegateway::ResumeGatewayPolicyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::ResumeGatewayPolicyWorkflowResponse>, tonic::Status> {
        self.run_resume_gateway_policy_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a gateway policy.
    async fn inspect_gateway_policy_workflow(
        &self,
        request: tonic::Request<edgegateway::InspectGatewayPolicyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::InspectGatewayPolicyWorkflowResponse>, tonic::Status> {
        self.run_inspect_gateway_policy_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists gateway policies a page at a time.
    async fn list_gateway_policy_workflows(
        &self,
        request: tonic::Request<edgegateway::ListGatewayPolicyWorkflowsRequest>,
    ) -> Result<tonic::Response<edgegateway::ListGatewayPolicyWorkflowsResponse>, tonic::Status> {
        self.run_list_gateway_policy_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals a gateway policy.
    async fn signal_gateway_policy_workflow(
        &self,
        request: tonic::Request<edgegateway::SignalGatewayPolicyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::SignalGatewayPolicyWorkflowResponse>, tonic::Status> {
        self.run_signal_gateway_policy_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
