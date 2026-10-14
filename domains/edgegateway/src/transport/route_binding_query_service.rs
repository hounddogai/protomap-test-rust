use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::admissions::v1::admission_service_client::AdmissionServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::csr_approval_service_client::CsrApprovalServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_notification_service_client::RouteBindingNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_query_service_server::RouteBindingQueryService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_query_service_client::LedgerAccountQueryServiceClient;
use acme_proto::acme::patientapp::v1 as patientapp;
use acme_proto::acme::patientapp::v1::patient_home_query_service_client::PatientHomeQueryServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_approval_service_client::AccessGrantApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_service_client::SecretServiceClient;

/// Answers read-only questions about route bindings, for screens and reports.
#[derive(Clone)]
pub struct RouteBindingQueryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    export_route_bindings_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            edgegateway::SignalGatewayPolicyWorkflowRequest,
            edgegateway::SignalGatewayPolicyWorkflowResponse,
        >,
    >,
    export_route_bindings_3_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<patientapp::FindPatientHomesRequest, patientapp::FindPatientHomesResponse>,
    >,
    lookup_route_binding_0_port: acme_platform::ports::Callback<
        edgegateway::PauseGatewayPolicyWorkflowRequest,
        edgegateway::PauseGatewayPolicyWorkflowResponse,
    >,
}
impl RouteBindingQueryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            edgegateway::PauseGatewayPolicyWorkflowRequest,
            edgegateway::PauseGatewayPolicyWorkflowResponse,
        > = {
            let client = GatewayPolicyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::PauseGatewayPolicyWorkflowRequest| -> acme_platform::ports::RpcFuture<edgegateway::PauseGatewayPolicyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.pause_gateway_policy_workflow(request).await?.into_inner())
})
})
        };
        let lookup_route_binding_0_port = callback;
        let callback: acme_platform::ports::Callback<
            edgegateway::SignalGatewayPolicyWorkflowRequest,
            edgegateway::SignalGatewayPolicyWorkflowResponse,
        > = {
            let client = GatewayPolicyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::SignalGatewayPolicyWorkflowRequest| -> acme_platform::ports::RpcFuture<edgegateway::SignalGatewayPolicyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.signal_gateway_policy_workflow(request).await?.into_inner())
})
})
        };
        let export_route_bindings_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            patientapp::FindPatientHomesRequest,
            patientapp::FindPatientHomesResponse,
        > = {
            let client = PatientHomeQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: patientapp::FindPatientHomesRequest| -> acme_platform::ports::RpcFuture<patientapp::FindPatientHomesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.find_patient_homes(request).await?.into_inner())
})
})
        };
        let export_route_bindings_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            export_route_bindings_2_port,
            export_route_bindings_3_port,
            lookup_route_binding_0_port,
        }
    }
    /// Run LookupRouteBinding after validating the tenant and durably recording the command.
    async fn run_lookup_route_binding(
        &self,
        req: edgegateway::LookupRouteBindingRequest,
    ) -> Result<edgegateway::LookupRouteBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupRouteBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupRouteBinding", "running workflow");
        let outgoing_0 = edgegateway::PauseGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            details: Some(edgegateway::pause_gateway_policy_workflow_request::Details {
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
            gateway_policy: Some(edgegateway::GatewayPolicy {
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
        let reply_0 = (self.lookup_route_binding_0_port)(outgoing_0).await?;
        let outgoing_1 = edgegateway::StreamRouteBindingNoticesRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::stream_route_binding_notices_request::Filter {
                created_after: reply_0
                    .gateway_policy
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: reply_0
                    .gateway_policy
                    .clone()
                    .unwrap_or_default()
                    .gateway_policy_id
                    .clone(),
                ..Default::default()
            }),
            since: reply_0
                .gateway_policy
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
            subscriber: Some(common::ContactInfo {
                email: (match reply_0.gateway_policy.clone().unwrap_or_default().action.clone() {
                    Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                    _ => Default::default(),
                })
                .notify_email
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::StreamRouteBindingNoticesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_route_binding_notices(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::StreamRouteBindingNoticesResponse::default();
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
        let outgoing_2 = secrets::BatchCreateSecretsRequest {
            context: req.context.clone(),
            chunk: Some(secrets::batch_create_secrets_request::Chunk::Record(secrets::Secret {
                holder: Some(identity::UserRef {
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
            })),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = SecretServiceClient::new(channel);
            let input: tonic::Request<secrets::BatchCreateSecretsRequest> =
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
            let response = client.batch_create_secrets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
            let response = client.stream_flag_evaluation_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagEvaluationChangesResponse::default();
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
        let mut response = edgegateway::LookupRouteBindingResponse::default();
        let contribution: String = reply_0.gateway_policy.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.route_binding.get_or_insert_with(Default::default).name = contribution;
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
    /// Run BrowseRouteBindings after validating the tenant and durably recording the command.
    async fn run_browse_route_bindings(
        &self,
        req: edgegateway::BrowseRouteBindingsRequest,
    ) -> Result<edgegateway::BrowseRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseRouteBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseRouteBindings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = secrets::RejectAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(secrets::reject_access_grant_request::Details {
                approver: Some(identity::UserRef {
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
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = AccessGrantApprovalServiceClient::new(channel);
            let input: tonic::Request<secrets::RejectAccessGrantRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reject_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = edgegateway::BrowseRouteBindingsResponse::default();
        let contribution: String = reply_0.access_grant.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            if response.route_bindings.is_empty() {
                response.route_bindings.push(Default::default());
            }
            response.route_bindings[0].description = contribution;
        }
        Ok(response)
    }
    /// Run ResolveRouteBindings after validating the tenant and durably recording the command.
    async fn run_resolve_route_bindings(
        &self,
        req: edgegateway::ResolveRouteBindingsRequest,
    ) -> Result<edgegateway::ResolveRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveRouteBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveRouteBindings", "running workflow");
        let response = edgegateway::ResolveRouteBindingsResponse::default();
        Ok(response)
    }
    /// Run ExportRouteBindings after validating the tenant and durably recording the command.
    async fn run_export_route_bindings(
        &self,
        req: edgegateway::ExportRouteBindingsRequest,
    ) -> Result<edgegateway::ExportRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportRouteBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportRouteBindings", "running workflow");
        tracing::debug!(audit_value = ?req.format.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::ListRouteBindingSubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_route_binding_subscriptions_request::Filter {
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListRouteBindingSubscriptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_route_binding_subscriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = admissions::BatchGetAdmissionsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AdmissionServiceClient::new(channel);
            let input: tonic::Request<admissions::BatchGetAdmissionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_admissions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::SignalGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            gateway_policy: Some(edgegateway::GatewayPolicy {
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
        let reply_2 = acme_platform::ports::execute(self.export_route_bindings_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = patientapp::FindPatientHomesRequest {
            context: req.context.clone(),
            filter: Some(patientapp::find_patient_homes_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_3 = acme_platform::ports::execute(self.export_route_bindings_3_port.as_ref(), outgoing_3).await?;
        let mut response = edgegateway::ExportRouteBindingsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2
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
                .route_binding
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<common::TimeWindow> =
            reply_2.gateway_policy.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.route_binding.get_or_insert_with(Default::default).effective = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeRouteBindings after validating the tenant and durably recording the command.
    async fn run_summarize_route_bindings(
        &self,
        req: edgegateway::SummarizeRouteBindingsRequest,
    ) -> Result<edgegateway::SummarizeRouteBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeRouteBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeRouteBindings", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::StartGatewayPolicyWorkflowRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            gateway_policy: Some(edgegateway::GatewayPolicy {
                action: Some(edgegateway::gateway_policy::Action::Block(
                    edgegateway::GatewayPolicyBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<edgegateway::StartGatewayPolicyWorkflowRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
                let input: tonic::Request<edgegateway::StartGatewayPolicyWorkflowRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.start_gateway_policy_workflow(input).await?;
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
        let outgoing_1 = configuration::StreamConfigChangeMetricsRequest {
            context: req.context.clone(),
            filter: Some(configuration::stream_config_change_metrics_request::Filter {
                owner_id: reply_0
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
            subscriber: Some(common::ContactInfo {
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
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamConfigChangeMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_config_change_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::StreamConfigChangeMetricsResponse::default();
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
        let outgoing_2 = edgegateway::SubscribeRouteBindingUpdatesRequest {
            context: req.context.clone(),
            details: Some(edgegateway::subscribe_route_binding_updates_request::Details {
                recipient: Some(common::ContactInfo {
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
            reason: (match reply_0.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            route_binding_id: req.route_binding_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SubscribeRouteBindingUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_route_binding_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = certificates::RecallCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
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
                audit: Some(common::AuditInfo {
                    created_by: reply_2
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
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
                    email: reply_2
                        .route_binding
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = CsrApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::RecallCsrRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recall_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::SummarizeRouteBindingsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run PeekRouteBinding after validating the tenant and durably recording the command.
    async fn run_peek_route_binding(
        &self,
        req: edgegateway::PeekRouteBindingRequest,
    ) -> Result<edgegateway::PeekRouteBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekRouteBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekRouteBinding", "running workflow");
        let command_0 = PeekRouteBindingDispatch0::from(req.clone());
        let outgoing_0: ledger::SummarizeLedgerAccountsRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = LedgerAccountQueryServiceClient::new(channel);
            let input: tonic::Request<ledger::SummarizeLedgerAccountsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_ledger_accounts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::ResumeGatewayPolicyWorkflowRequest {
            annotations: reply_0.result.clone().unwrap_or_default().details.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::resume_gateway_policy_workflow_request::Details {
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
            gateway_policy: Some(edgegateway::GatewayPolicy {
                effective: reply_0.period.clone(),
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.edgegateway.v1.ResumeGatewayPolicyWorkflowRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ResumeGatewayPolicyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_gateway_policy_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::PeekRouteBindingResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .route_binding
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
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
/// The domain command prepared by PeekRouteBinding for SummarizeLedgerAccounts.
struct PeekRouteBindingDispatch0 {
    context: Option<common::RequestContext>,
    annotations: std::collections::BTreeMap<String, String>,
}
impl From<edgegateway::PeekRouteBindingRequest> for PeekRouteBindingDispatch0 {
    fn from(req: edgegateway::PeekRouteBindingRequest) -> Self {
        Self {
            context: req.context.clone(),
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
        }
    }
}
impl From<PeekRouteBindingDispatch0> for ledger::SummarizeLedgerAccountsRequest {
    fn from(value: PeekRouteBindingDispatch0) -> Self {
        ledger::SummarizeLedgerAccountsRequest {
            annotations: value.annotations,
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl RouteBindingQueryService for RouteBindingQueryEndpoint {
    /// Returns a route binding.
    async fn lookup_route_binding(
        &self,
        request: tonic::Request<edgegateway::LookupRouteBindingRequest>,
    ) -> Result<tonic::Response<edgegateway::LookupRouteBindingResponse>, tonic::Status> {
        self.run_lookup_route_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists route bindings a page at a time.
    async fn browse_route_bindings(
        &self,
        request: tonic::Request<edgegateway::BrowseRouteBindingsRequest>,
    ) -> Result<tonic::Response<edgegateway::BrowseRouteBindingsResponse>, tonic::Status> {
        self.run_browse_route_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several route bindings at once.
    async fn resolve_route_bindings(
        &self,
        request: tonic::Request<edgegateway::ResolveRouteBindingsRequest>,
    ) -> Result<tonic::Response<edgegateway::ResolveRouteBindingsResponse>, tonic::Status> {
        self.run_resolve_route_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportRouteBindingsStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::ExportRouteBindingsResponse, tonic::Status>>;
    /// Streams every matching route binding.
    async fn export_route_bindings(
        &self,
        request: tonic::Request<edgegateway::ExportRouteBindingsRequest>,
    ) -> Result<tonic::Response<Self::ExportRouteBindingsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_export_route_bindings(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a route binding.
    async fn summarize_route_bindings(
        &self,
        request: tonic::Request<edgegateway::SummarizeRouteBindingsRequest>,
    ) -> Result<tonic::Response<edgegateway::SummarizeRouteBindingsResponse>, tonic::Status> {
        self.run_summarize_route_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a route binding.
    async fn peek_route_binding(
        &self,
        request: tonic::Request<edgegateway::PeekRouteBindingRequest>,
    ) -> Result<tonic::Response<edgegateway::PeekRouteBindingResponse>, tonic::Status> {
        self.run_peek_route_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
