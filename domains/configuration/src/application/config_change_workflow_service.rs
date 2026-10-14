use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::config_change_workflow_service_server::ConfigChangeWorkflowService;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::configuration::v1::environment_approval_service_client::EnvironmentApprovalServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::fleet::v1::vehicle_query_service_client::VehicleQueryServiceClient;
use acme_proto::acme::fleet::v1::vehicle_service_client::VehicleServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_ring_query_service_client::KeyRingQueryServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_service_client::SecretServiceClient;

/// Runs the long-lived workflow that takes a config change from start to finish.
#[derive(Clone)]
pub struct ConfigChangeWorkflowApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    retry_config_change_workflow_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ShareConfigChangeReportRequest,
            configuration::ShareConfigChangeReportResponse,
        >,
    >,
    retry_config_change_workflow_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::InspectConfigSetWorkflowRequest,
                configuration::InspectConfigSetWorkflowResponse,
            >,
    >,
    retry_config_change_workflow_2_port: acme_platform::ports::Callback<
        certificates::SuggestCertificatesRequest,
        certificates::SuggestCertificatesResponse,
    >,
    signal_config_change_workflow_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::GenerateConfigSetReportRequest,
            configuration::GenerateConfigSetReportResponse,
        >,
    >,
    signal_config_change_workflow_1_port: acme_platform::ports::Callback<
        configuration::RejectEnvironmentRequest,
        configuration::RejectEnvironmentResponse,
    >,
}
impl ConfigChangeWorkflowApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::ShareConfigChangeReportRequest,
            configuration::ShareConfigChangeReportResponse,
        > = {
            let client = ConfigChangeReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ShareConfigChangeReportRequest| -> acme_platform::ports::RpcFuture<configuration::ShareConfigChangeReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_config_change_report(request).await?.into_inner())
})
})
        };
        let retry_config_change_workflow_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::InspectConfigSetWorkflowRequest,
            configuration::InspectConfigSetWorkflowResponse,
        > = {
            let client = ConfigSetWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::InspectConfigSetWorkflowRequest| -> acme_platform::ports::RpcFuture<configuration::InspectConfigSetWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.inspect_config_set_workflow(request).await?.into_inner())
})
})
        };
        let retry_config_change_workflow_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::SuggestCertificatesRequest,
            certificates::SuggestCertificatesResponse,
        > = {
            let client = CertificateQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::SuggestCertificatesRequest| -> acme_platform::ports::RpcFuture<certificates::SuggestCertificatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.suggest_certificates(request).await?.into_inner())
})
})
        };
        let retry_config_change_workflow_2_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::GenerateConfigSetReportRequest,
            configuration::GenerateConfigSetReportResponse,
        > = {
            let client = ConfigSetReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::GenerateConfigSetReportRequest| -> acme_platform::ports::RpcFuture<configuration::GenerateConfigSetReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.generate_config_set_report(request).await?.into_inner())
})
})
        };
        let signal_config_change_workflow_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::RejectEnvironmentRequest,
            configuration::RejectEnvironmentResponse,
        > = {
            let client = EnvironmentApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::RejectEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::RejectEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.reject_environment(request).await?.into_inner())
})
})
        };
        let signal_config_change_workflow_1_port = callback;
        Self {
            channel,
            repository,
            retry_config_change_workflow_0_port,
            retry_config_change_workflow_1_port,
            retry_config_change_workflow_2_port,
            signal_config_change_workflow_0_port,
            signal_config_change_workflow_1_port,
        }
    }
    /// Run StartConfigChangeWorkflow after validating the tenant and durably recording the command.
    async fn run_start_config_change_workflow(
        &self,
        req: configuration::StartConfigChangeWorkflowRequest,
    ) -> Result<configuration::StartConfigChangeWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartConfigChangeWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartConfigChangeWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let command_0 = StartConfigChangeWorkflowDispatch0::from(req.clone());
        let outgoing_0: secrets::UpdateSecretRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretServiceClient::new(channel);
            let input: tonic::Request<secrets::UpdateSecretRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_secret(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = fleet::BatchCreateVehiclesRequest {
            context: req.context.clone(),
            chunk: Some(fleet::batch_create_vehicles_request::Chunk::Record(fleet::Vehicle {
                audit: Some(common::AuditInfo {
                    updated_at: reply_0
                        .secret
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
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: reply_0
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: reply_0
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
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
        let reply_1 = async move {
            let mut client = VehicleServiceClient::new(channel);
            let input: tonic::Request<fleet::BatchCreateVehiclesRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_vehicles(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    participants: vec![featureflags::FlagRolloutParticipant {
                        contact: Some(common::ContactInfo {
                            name: reply_0
                                .secret
                                .clone()
                                .unwrap_or_default()
                                .contact
                                .clone()
                                .unwrap_or_default()
                                .name
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
        let reply_2 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
        let mut response = configuration::StartConfigChangeWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
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
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: i64 = req.config_change.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.config_change.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceConfigChangeWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_config_change_workflow(
        &self,
        req: configuration::AdvanceConfigChangeWorkflowRequest,
    ) -> Result<configuration::AdvanceConfigChangeWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceConfigChangeWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceConfigChangeWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::SuggestKeyRingsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::suggest_key_rings_request::Filter {
                labels: req.details.clone().unwrap_or_default().parameters.clone(),
                owner_id: req
                    .config_change
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .config_set_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.keymanagement.v1.SuggestKeyRingsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SuggestKeyRingsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_key_rings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::AdvanceConfigChangeWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: String = req.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run CancelConfigChangeWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_config_change_workflow(
        &self,
        req: configuration::CancelConfigChangeWorkflowRequest,
    ) -> Result<configuration::CancelConfigChangeWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelConfigChangeWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelConfigChangeWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::RevertAccessPolicyRequest {
            access_policy: Some(authorization::AccessPolicy {
                owner: Some(identity::UserRef {
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
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(authorization::revert_access_policy_request::Details {
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
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
        let mut response = configuration::CancelConfigChangeWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<configuration::ConfigSetRef> =
            req.config_change.clone().unwrap_or_default().subject.clone();
        if contribution != <Option<configuration::ConfigSetRef>>::default() {
            response.config_change.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run RetryConfigChangeWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_config_change_workflow(
        &self,
        req: configuration::RetryConfigChangeWorkflowRequest,
    ) -> Result<configuration::RetryConfigChangeWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryConfigChangeWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryConfigChangeWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ShareConfigChangeReportRequest {
            context: req.context.clone(),
            details: Some(configuration::share_config_change_report_request::Details {
                recipient: Some(common::ContactInfo {
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.retry_config_change_workflow_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::InspectConfigSetWorkflowRequest {
            config_set_id: req
                .config_change
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .config_set_id
                .clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            acme_platform::ports::execute(self.retry_config_change_workflow_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = certificates::SuggestCertificatesRequest {
            context: req.context.clone(),
            filter: Some(certificates::suggest_certificates_request::Filter {
                created_after: req.config_change.clone().unwrap_or_default().occurred_at.clone(),
                labels: req.details.clone().unwrap_or_default().parameters.clone(),
                owner_id: reply_0
                    .config_change
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .config_set_id
                    .clone(),
                region: reply_1
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
        let reply_2 = (self.retry_config_change_workflow_2_port)(outgoing_2).await?;
        let mut response = configuration::RetryConfigChangeWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: String = req.config_change.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run ResumeConfigChangeWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_config_change_workflow(
        &self,
        req: configuration::ResumeConfigChangeWorkflowRequest,
    ) -> Result<configuration::ResumeConfigChangeWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeConfigChangeWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeConfigChangeWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = fleet::FindVehiclesRequest {
            context: req.context.clone(),
            filter: Some(fleet::find_vehicles_request::Filter {
                created_after: req.config_change.clone().unwrap_or_default().occurred_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VehicleQueryServiceClient::new(channel);
            let input: tonic::Request<fleet::FindVehiclesRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_vehicles(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ResumeConfigChangeWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.config_change.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.config_change.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run InspectConfigChangeWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_config_change_workflow(
        &self,
        req: configuration::InspectConfigChangeWorkflowRequest,
    ) -> Result<configuration::InspectConfigChangeWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectConfigChangeWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectConfigChangeWorkflow", "running workflow");
        let outgoing_0 = configuration::DelegateEnvironmentApprovalRequest {
            context: req.context.clone(),
            details: Some(configuration::delegate_environment_approval_request::Details {
                approver: Some(identity::UserRef {
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
            environment: Some(configuration::Environment {
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    configuration::EnvironmentComponent {
                        component_id: req.config_change_id.clone(),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = EnvironmentApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::DelegateEnvironmentApprovalRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.delegate_environment_approval(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = configuration::ListConfigChangeReportsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_config_change_reports_request::Filter {
                created_after: reply_0
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                region: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ListConfigChangeReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_config_change_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::InspectConfigChangeWorkflowResponse::default();
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
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        Ok(response)
    }
    /// Run SignalConfigChangeWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_config_change_workflow(
        &self,
        req: configuration::SignalConfigChangeWorkflowRequest,
    ) -> Result<configuration::SignalConfigChangeWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalConfigChangeWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalConfigChangeWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::GenerateConfigSetReportRequest {
            annotations: req.annotations.clone(),
            config_set_id: req
                .config_change
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .config_set_id
                .clone(),
            context: req.context.clone(),
            details: Some(configuration::generate_config_set_report_request::Details {
                filters: req.details.clone().unwrap_or_default().parameters.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.signal_config_change_workflow_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = configuration::RejectEnvironmentRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(configuration::reject_environment_request::Details {
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
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    display_name: req
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.signal_config_change_workflow_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let jobs_2 = req
            .config_change
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_1 = reply_1.clone();
                async move {
                    let outgoing_2 = certificates::ExportCertificateAuthoritiesRequest {
                        context: req.context.clone(),
                        filter: Some(certificates::export_certificate_authorities_request::Filter {
                            created_after: item.observed_at.clone(),
                            region: (match reply_1.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .region
                            .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CertificateAuthorityQueryServiceClient::new(channel);
                        let input: tonic::Request<certificates::ExportCertificateAuthoritiesRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.export_certificate_authorities(input).await?;
                        let mut frames = response.into_inner();
                        let mut last = certificates::ExportCertificateAuthoritiesResponse::default();
                        // The caller asks for a bounded snapshot, then drops its subscription.
                        for _ in 0..8 {
                            match frames.message().await? {
                                Some(frame) => last = frame,
                                None => break,
                            }
                        }
                        Ok::<_, tonic::Status>(last)
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let mut response = configuration::SignalConfigChangeWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: i64 = req.config_change.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.config_change.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by StartConfigChangeWorkflow for UpdateSecret.
struct StartConfigChangeWorkflowDispatch0 {
    context: Option<common::RequestContext>,
    secret_contact_name: Option<common::PersonName>,
    secret_holder_email: String,
}
impl From<configuration::StartConfigChangeWorkflowRequest> for StartConfigChangeWorkflowDispatch0 {
    fn from(req: configuration::StartConfigChangeWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            secret_contact_name: req
                .details
                .clone()
                .unwrap_or_default()
                .notify
                .clone()
                .unwrap_or_default()
                .name
                .clone(),
            secret_holder_email: req
                .details
                .clone()
                .unwrap_or_default()
                .notify
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
        }
    }
}
impl From<StartConfigChangeWorkflowDispatch0> for secrets::UpdateSecretRequest {
    fn from(value: StartConfigChangeWorkflowDispatch0) -> Self {
        secrets::UpdateSecretRequest {
            context: value.context,
            secret: Some(secrets::Secret {
                contact: Some(common::ContactInfo {
                    name: value.secret_contact_name,
                    ..Default::default()
                }),
                holder: Some(identity::UserRef {
                    email: value.secret_holder_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigChangeWorkflowService for ConfigChangeWorkflowApi {
    /// Starts a config change.
    async fn start_config_change_workflow(
        &self,
        request: tonic::Request<configuration::StartConfigChangeWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::StartConfigChangeWorkflowResponse>, tonic::Status> {
        self.run_start_config_change_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances a config change.
    async fn advance_config_change_workflow(
        &self,
        request: tonic::Request<configuration::AdvanceConfigChangeWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::AdvanceConfigChangeWorkflowResponse>, tonic::Status> {
        self.run_advance_config_change_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a config change.
    async fn cancel_config_change_workflow(
        &self,
        request: tonic::Request<configuration::CancelConfigChangeWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::CancelConfigChangeWorkflowResponse>, tonic::Status> {
        self.run_cancel_config_change_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a config change.
    async fn retry_config_change_workflow(
        &self,
        request: tonic::Request<configuration::RetryConfigChangeWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::RetryConfigChangeWorkflowResponse>, tonic::Status> {
        self.run_retry_config_change_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes a config change.
    async fn resume_config_change_workflow(
        &self,
        request: tonic::Request<configuration::ResumeConfigChangeWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::ResumeConfigChangeWorkflowResponse>, tonic::Status> {
        self.run_resume_config_change_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config change.
    async fn inspect_config_change_workflow(
        &self,
        request: tonic::Request<configuration::InspectConfigChangeWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::InspectConfigChangeWorkflowResponse>, tonic::Status> {
        self.run_inspect_config_change_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals a config change.
    async fn signal_config_change_workflow(
        &self,
        request: tonic::Request<configuration::SignalConfigChangeWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::SignalConfigChangeWorkflowResponse>, tonic::Status> {
        self.run_signal_config_change_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
