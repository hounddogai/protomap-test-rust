use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_ring_workflow_service_client::KeyRingWorkflowServiceClient;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_validation_service_client::MetricSeriesValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_validation_service_client::AccessGrantValidationServiceClient;
use acme_proto::acme::secrets::v1::secret_query_service_client::SecretQueryServiceClient;
use acme_proto::acme::secrets::v1::secret_version_approval_service_client::SecretVersionApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_version_validation_service_client::SecretVersionValidationServiceClient;
use acme_proto::acme::secrets::v1::secret_workflow_service_server::SecretWorkflowService;

/// Runs the long-lived workflow that takes a secret from start to finish.
#[derive(Clone)]
pub struct SecretWorkflowHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    advance_secret_workflow_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<secrets::NormalizeAccessGrantRequest, secrets::NormalizeAccessGrantResponse>,
    >,
    inspect_secret_workflow_0_port: acme_platform::ports::Callback<
        keymanagement::CancelKeyRingWorkflowRequest,
        keymanagement::CancelKeyRingWorkflowResponse,
    >,
}
impl SecretWorkflowHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            secrets::NormalizeAccessGrantRequest,
            secrets::NormalizeAccessGrantResponse,
        > = {
            let client = AccessGrantValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::NormalizeAccessGrantRequest| -> acme_platform::ports::RpcFuture<secrets::NormalizeAccessGrantResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_access_grant(request).await?.into_inner())
})
})
        };
        let advance_secret_workflow_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            keymanagement::CancelKeyRingWorkflowRequest,
            keymanagement::CancelKeyRingWorkflowResponse,
        > = {
            let client = KeyRingWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::CancelKeyRingWorkflowRequest| -> acme_platform::ports::RpcFuture<keymanagement::CancelKeyRingWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.cancel_key_ring_workflow(request).await?.into_inner())
})
})
        };
        let inspect_secret_workflow_0_port = callback;
        Self {
            channel,
            repository,
            advance_secret_workflow_0_port,
            inspect_secret_workflow_0_port,
        }
    }
    /// Run StartSecretWorkflow after validating the tenant and durably recording the command.
    async fn run_start_secret_workflow(
        &self,
        req: secrets::StartSecretWorkflowRequest,
    ) -> Result<secrets::StartSecretWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StartSecretWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartSecretWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::ApproveSecretVersionRequest {
            context: req.context.clone(),
            details: Some(secrets::approve_secret_version_request::Details {
                approver: Some(identity::UserRef {
                    display_name: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
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
                    user_id: req
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretVersionApprovalServiceClient::new(channel);
            let input: tonic::Request<secrets::ApproveSecretVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.approve_secret_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::StartSecretWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: String = req.secret.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceSecretWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_secret_workflow(
        &self,
        req: secrets::AdvanceSecretWorkflowRequest,
    ) -> Result<secrets::AdvanceSecretWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("AdvanceSecretWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceSecretWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::NormalizeAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                owner: Some(identity::UserRef {
                    email: req
                        .secret
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
            context: req.context.clone(),
            details: Some(secrets::normalize_access_grant_request::Details {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.advance_secret_workflow_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = secrets::AdvanceSecretWorkflowResponse::default();
        let contribution: String = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.secret.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.secret.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
    /// Run RetrySecretWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_secret_workflow(
        &self,
        req: secrets::RetrySecretWorkflowRequest,
    ) -> Result<secrets::RetrySecretWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetrySecretWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetrySecretWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().holder.clone().unwrap_or_default().user_id.clone(), "recorded audit field");
        let command_0 = RetrySecretWorkflowDispatch0::from(req.clone());
        let outgoing_0: secrets::LookupSecretRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretQueryServiceClient::new(channel);
            let input: tonic::Request<secrets::LookupSecretRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lookup_secret(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::RetrySecretWorkflowResponse::default();
        let contribution: Option<common::PersonName> = reply_0
            .secret
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .name
            .clone();
        if contribution != <Option<common::PersonName>>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0.secret.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.secret.get_or_insert_with(Default::default).opened_at = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.secret.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ResumeSecretWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_secret_workflow(
        &self,
        req: secrets::ResumeSecretWorkflowRequest,
    ) -> Result<secrets::ResumeSecretWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResumeSecretWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeSecretWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().holder.clone().unwrap_or_default().user_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::BrowseSecretsRequest {
            context: req.context.clone(),
            filter: Some(secrets::browse_secrets_request::Filter {
                created_after: req
                    .secret
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.secret.clone().unwrap_or_default().labels.clone(),
                owner_id: req.secret.clone().unwrap_or_default().secret_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<secrets::BrowseSecretsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretQueryServiceClient::new(channel);
            let input: tonic::Request<secrets::BrowseSecretsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_secrets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ResumeSecretWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: String = req.secret.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run InspectSecretWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_secret_workflow(
        &self,
        req: secrets::InspectSecretWorkflowRequest,
    ) -> Result<secrets::InspectSecretWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectSecretWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectSecretWorkflow", "running workflow");
        let outgoing_0 = keymanagement::CancelKeyRingWorkflowRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        children: vec![keymanagement::KeyRing {
                            parent_id: req.secret_id.clone(),
                            ..Default::default()
                        }],
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.inspect_secret_workflow_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = secrets::InspectSecretWorkflowResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        Ok(response)
    }
    /// Run ListSecretWorkflows after validating the tenant and durably recording the command.
    async fn run_list_secret_workflows(
        &self,
        req: secrets::ListSecretWorkflowsRequest,
    ) -> Result<secrets::ListSecretWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListSecretWorkflows", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListSecretWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = secrets::ListSecretWorkflowsResponse::default();
        Ok(response)
    }
    /// Run SignalSecretWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_secret_workflow(
        &self,
        req: secrets::SignalSecretWorkflowRequest,
    ) -> Result<secrets::SignalSecretWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SignalSecretWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalSecretWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::NormalizeAccessGrantRequest {
            context: req.context.clone(),
            details: Some(secrets::normalize_access_grant_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    phone: req
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
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = metrics::PreviewMetricSeriesRequest {
            context: req.context.clone(),
            metric_series: Some(metrics::MetricSeries {
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = AccessGrantValidationServiceClient::new(channel_0);
                let input: tonic::Request<secrets::NormalizeAccessGrantRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.normalize_access_grant(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = MetricSeriesValidationServiceClient::new(channel_1);
                let input: tonic::Request<metrics::PreviewMetricSeriesRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.preview_metric_series(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = secrets::PreviewSecretVersionRequest {
            context: req.context.clone(),
            secret_version: Some(secrets::SecretVersion {
                subject: Some(secrets::AccessGrantRef {
                    display_name: reply_1
                        .metric_series
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
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = SecretVersionValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::PreviewSecretVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_secret_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = secrets::SignalSecretWorkflowResponse::default();
        let contribution: Option<common::Money> =
            (match reply_0.access_grant.clone().unwrap_or_default().action.clone() {
                Some(secrets::access_grant::Action::Discount(value)) => value,
                _ => Default::default(),
            })
            .amount
            .clone();
        if contribution != <Option<common::Money>>::default() {
            if response.secret.get_or_insert_with(Default::default).limits.is_empty() {
                response
                    .secret
                    .get_or_insert_with(Default::default)
                    .limits
                    .push(Default::default());
            }
            response.secret.get_or_insert_with(Default::default).limits[0].amount = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.secret.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.secret.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by RetrySecretWorkflow for LookupSecret.
struct RetrySecretWorkflowDispatch0 {
    context: Option<common::RequestContext>,
    secret_id: String,
}
impl From<secrets::RetrySecretWorkflowRequest> for RetrySecretWorkflowDispatch0 {
    fn from(req: secrets::RetrySecretWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            secret_id: req.secret.clone().unwrap_or_default().secret_id.clone(),
        }
    }
}
impl From<RetrySecretWorkflowDispatch0> for secrets::LookupSecretRequest {
    fn from(value: RetrySecretWorkflowDispatch0) -> Self {
        secrets::LookupSecretRequest {
            context: value.context,
            secret_id: value.secret_id,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl SecretWorkflowService for SecretWorkflowHandler {
    /// Starts a secret.
    async fn start_secret_workflow(
        &self,
        request: tonic::Request<secrets::StartSecretWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::StartSecretWorkflowResponse>, tonic::Status> {
        self.run_start_secret_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances a secret.
    async fn advance_secret_workflow(
        &self,
        request: tonic::Request<secrets::AdvanceSecretWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::AdvanceSecretWorkflowResponse>, tonic::Status> {
        self.run_advance_secret_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a secret.
    async fn retry_secret_workflow(
        &self,
        request: tonic::Request<secrets::RetrySecretWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::RetrySecretWorkflowResponse>, tonic::Status> {
        self.run_retry_secret_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes a secret.
    async fn resume_secret_workflow(
        &self,
        request: tonic::Request<secrets::ResumeSecretWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::ResumeSecretWorkflowResponse>, tonic::Status> {
        self.run_resume_secret_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a secret.
    async fn inspect_secret_workflow(
        &self,
        request: tonic::Request<secrets::InspectSecretWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::InspectSecretWorkflowResponse>, tonic::Status> {
        self.run_inspect_secret_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secrets a page at a time.
    async fn list_secret_workflows(
        &self,
        request: tonic::Request<secrets::ListSecretWorkflowsRequest>,
    ) -> Result<tonic::Response<secrets::ListSecretWorkflowsResponse>, tonic::Status> {
        self.run_list_secret_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals a secret.
    async fn signal_secret_workflow(
        &self,
        request: tonic::Request<secrets::SignalSecretWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::SignalSecretWorkflowResponse>, tonic::Status> {
        self.run_signal_secret_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
