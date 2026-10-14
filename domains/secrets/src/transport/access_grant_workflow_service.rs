use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_history_service_client::PermissionCheckHistoryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_query_service_client::DocumentQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_validation_service_client::MetricSeriesValidationServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_validation_service_client::NotificationValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_workflow_service_server::AccessGrantWorkflowService;
use acme_proto::acme::secrets::v1::secret_approval_service_client::SecretApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_service_client::SecretServiceClient;
use acme_proto::acme::secrets::v1::secret_version_approval_service_client::SecretVersionApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_version_query_service_client::SecretVersionQueryServiceClient;
use acme_proto::acme::secrets::v1::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient;
use acme_proto::acme::secrets::v1::secret_workflow_service_client::SecretWorkflowServiceClient;

/// Runs the long-lived workflow that takes an access grant from start to finish.
#[derive(Clone)]
pub struct AccessGrantWorkflowHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    inspect_access_grant_workflow_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                notifications::LintNotificationsRequest,
                notifications::LintNotificationsResponse,
            >,
    >,
    pause_access_grant_workflow_0_port:
        acme_platform::ports::Callback<secrets::RejectSecretVersionRequest, secrets::RejectSecretVersionResponse>,
    pause_access_grant_workflow_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::SubscribeFeatureFlagUpdatesRequest,
            featureflags::SubscribeFeatureFlagUpdatesResponse,
        >,
    >,
}
impl AccessGrantWorkflowHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            secrets::RejectSecretVersionRequest,
            secrets::RejectSecretVersionResponse,
        > = {
            let client = SecretVersionApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::RejectSecretVersionRequest| -> acme_platform::ports::RpcFuture<secrets::RejectSecretVersionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.reject_secret_version(request).await?.into_inner())
})
})
        };
        let pause_access_grant_workflow_0_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::SubscribeFeatureFlagUpdatesRequest,
            featureflags::SubscribeFeatureFlagUpdatesResponse,
        > = {
            let client = FeatureFlagNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::SubscribeFeatureFlagUpdatesRequest| -> acme_platform::ports::RpcFuture<featureflags::SubscribeFeatureFlagUpdatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.subscribe_feature_flag_updates(request).await?.into_inner())
})
})
        };
        let pause_access_grant_workflow_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            notifications::LintNotificationsRequest,
            notifications::LintNotificationsResponse,
        > = {
            let client = NotificationValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: notifications::LintNotificationsRequest| -> acme_platform::ports::RpcFuture<notifications::LintNotificationsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_notifications(request).await?.into_inner())
})
})
        };
        let inspect_access_grant_workflow_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            inspect_access_grant_workflow_0_port,
            pause_access_grant_workflow_0_port,
            pause_access_grant_workflow_1_port,
        }
    }
    /// Run StartAccessGrantWorkflow after validating the tenant and durably recording the command.
    async fn run_start_access_grant_workflow(
        &self,
        req: secrets::StartAccessGrantWorkflowRequest,
    ) -> Result<secrets::StartAccessGrantWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartAccessGrantWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartAccessGrantWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = documents::ExportDocumentsRequest {
            context: req.context.clone(),
            filter: Some(documents::export_documents_request::Filter {
                created_after: req
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.access_grant.clone().unwrap_or_default().parameters.clone(),
                owner_id: req
                    .access_grant
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
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = DocumentQueryServiceClient::new(channel);
            let input: tonic::Request<documents::ExportDocumentsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_documents(input).await?;
            let mut frames = response.into_inner();
            let mut last = documents::ExportDocumentsResponse::default();
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
        let mut response = secrets::StartAccessGrantWorkflowResponse::default();
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_grant.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_grant.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceAccessGrantWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_access_grant_workflow(
        &self,
        req: secrets::AdvanceAccessGrantWorkflowRequest,
    ) -> Result<secrets::AdvanceAccessGrantWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceAccessGrantWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceAccessGrantWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::RejectSecretRequest {
            context: req.context.clone(),
            details: Some(secrets::reject_secret_request::Details {
                approver: Some(identity::UserRef {
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
        let outgoing_1 = secrets::CancelSecretVersionWorkflowRequest {
            context: req.context.clone(),
            details: Some(secrets::cancel_secret_version_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
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
        let mut response = secrets::AdvanceAccessGrantWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
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
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.access_grant.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.access_grant.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run CancelAccessGrantWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_access_grant_workflow(
        &self,
        req: secrets::CancelAccessGrantWorkflowRequest,
    ) -> Result<secrets::CancelAccessGrantWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelAccessGrantWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelAccessGrantWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::WatchSecretsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .access_grant
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
            let mut client = SecretServiceClient::new(channel);
            let input: tonic::Request<secrets::WatchSecretsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_secrets(input).await?;
            let mut frames = response.into_inner();
            let mut last = secrets::WatchSecretsResponse::default();
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
        let mut response = secrets::CancelAccessGrantWorkflowResponse::default();
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_grant.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_grant.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run RetryAccessGrantWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_access_grant_workflow(
        &self,
        req: secrets::RetryAccessGrantWorkflowRequest,
    ) -> Result<secrets::RetryAccessGrantWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryAccessGrantWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryAccessGrantWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = secrets::ExportSecretVersionsRequest {
            context: req.context.clone(),
            filter: Some(secrets::export_secret_versions_request::Filter {
                created_after: req
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: req.access_grant.clone().unwrap_or_default().access_grant_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = SecretVersionQueryServiceClient::new(channel);
            let input: tonic::Request<secrets::ExportSecretVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_secret_versions(input).await?;
            let mut frames = response.into_inner();
            let mut last = secrets::ExportSecretVersionsResponse::default();
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
        let outgoing_1 = authorization::AnnotatePermissionCheckVersionRequest {
            context: req.context.clone(),
            details: Some(authorization::annotate_permission_check_version_request::Details {
                author_email: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                    Some(secrets::access_grant::Action::Block(value)) => value,
                    _ => Default::default(),
                })
                .notify_email
                .clone(),
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            permission_check: Some(authorization::PermissionCheck {
                subject: Some(authorization::AccessPolicyRef {
                    display_name: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: req.access_grant.clone().unwrap_or_default().version.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                Some(secrets::access_grant::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = PermissionCheckHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::AnnotatePermissionCheckVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.annotate_permission_check_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = metrics::ValidateMetricSeriesRequest {
            context: req.context.clone(),
            details: Some(metrics::validate_metric_series_request::Details {
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
            metric_series: Some(metrics::MetricSeries {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    display_name: reply_1
                        .permission_check
                        .clone()
                        .unwrap_or_default()
                        .subject
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
        let outgoing_2 = acme_platform::codec::through_any(&outgoing_2, "acme.metrics.v1.ValidateMetricSeriesRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = MetricSeriesValidationServiceClient::new(channel);
            let input: tonic::Request<metrics::ValidateMetricSeriesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_metric_series(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::ListQuotaBucketConflictsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_quota_bucket_conflicts_request::Filter {
                created_after: reply_1.outcome.clone().unwrap_or_default().effective_at.clone(),
                labels: reply_1.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_1.outcome.clone().unwrap_or_default().reference_id.clone(),
                region: (match reply_2.metric_series.clone().unwrap_or_default().locator.clone() {
                    Some(metrics::metric_series::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ratelimiting::ListQuotaBucketConflictsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = QuotaBucketSyncServiceClient::new(channel);
                let input: tonic::Request<ratelimiting::ListQuotaBucketConflictsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_quota_bucket_conflicts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_3)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_3 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = secrets::RetryAccessGrantWorkflowResponse::default();
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<identity::UserRef> = reply_2.metric_series.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.access_grant.get_or_insert_with(Default::default).owner = contribution;
        }
        let contribution: String = reply_2
            .metric_series
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: i32 = req.access_grant.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run PauseAccessGrantWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_access_grant_workflow(
        &self,
        req: secrets::PauseAccessGrantWorkflowRequest,
    ) -> Result<secrets::PauseAccessGrantWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PauseAccessGrantWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseAccessGrantWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::RejectSecretVersionRequest {
            context: req.context.clone(),
            details: Some(secrets::reject_secret_version_request::Details {
                approver: Some(identity::UserRef {
                    email: req
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
        let reply_0 = (self.pause_access_grant_workflow_0_port)(outgoing_0).await?;
        let outgoing_1 = featureflags::SubscribeFeatureFlagUpdatesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(featureflags::subscribe_feature_flag_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .access_grant
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
        let reply_1 =
            self.pause_access_grant_workflow_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = secrets::PauseAccessGrantWorkflowResponse::default();
        let contribution: String = reply_0
            .secret_version
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .access_grant_id
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i32 = req.access_grant.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).priority = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.details.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ResumeAccessGrantWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_access_grant_workflow(
        &self,
        req: secrets::ResumeAccessGrantWorkflowRequest,
    ) -> Result<secrets::ResumeAccessGrantWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeAccessGrantWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeAccessGrantWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::ResumeSecretWorkflowRequest {
            context: req.context.clone(),
            secret: Some(secrets::Secret {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .access_grant
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
                    email: req
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
                holder: Some(identity::UserRef {
                    user_id: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                limits: vec![secrets::SecretLimit {
                    amount: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Discount(value)) => value,
                        _ => Default::default(),
                    })
                    .amount
                    .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretWorkflowServiceClient::new(channel);
            let input: tonic::Request<secrets::ResumeSecretWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_secret_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ResumeAccessGrantWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.access_grant.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run InspectAccessGrantWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_access_grant_workflow(
        &self,
        req: secrets::InspectAccessGrantWorkflowRequest,
    ) -> Result<secrets::InspectAccessGrantWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectAccessGrantWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectAccessGrantWorkflow", "running workflow");
        let outgoing_0 = notifications::LintNotificationsRequest {
            context: req.context.clone(),
            details: Some(notifications::lint_notifications_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            notification: Some(notifications::Notification {
                assignee: Some(identity::UserRef {
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
                comments: vec![notifications::NotificationComment {
                    created_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }],
                requester: Some(common::ContactInfo {
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
        let reply_0 =
            acme_platform::ports::execute(self.inspect_access_grant_workflow_0_port.as_ref(), outgoing_0).await?;
        let mut response = secrets::InspectAccessGrantWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        Ok(response)
    }
    /// Run ListAccessGrantWorkflows after validating the tenant and durably recording the command.
    async fn run_list_access_grant_workflows(
        &self,
        req: secrets::ListAccessGrantWorkflowsRequest,
    ) -> Result<secrets::ListAccessGrantWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListAccessGrantWorkflows", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListAccessGrantWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = secrets::ListAccessGrantWorkflowsResponse::default();
        Ok(response)
    }
}
#[tonic::async_trait]
impl AccessGrantWorkflowService for AccessGrantWorkflowHandler {
    /// Starts an access grant.
    async fn start_access_grant_workflow(
        &self,
        request: tonic::Request<secrets::StartAccessGrantWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::StartAccessGrantWorkflowResponse>, tonic::Status> {
        self.run_start_access_grant_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances an access grant.
    async fn advance_access_grant_workflow(
        &self,
        request: tonic::Request<secrets::AdvanceAccessGrantWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::AdvanceAccessGrantWorkflowResponse>, tonic::Status> {
        self.run_advance_access_grant_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels an access grant.
    async fn cancel_access_grant_workflow(
        &self,
        request: tonic::Request<secrets::CancelAccessGrantWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::CancelAccessGrantWorkflowResponse>, tonic::Status> {
        self.run_cancel_access_grant_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys an access grant.
    async fn retry_access_grant_workflow(
        &self,
        request: tonic::Request<secrets::RetryAccessGrantWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::RetryAccessGrantWorkflowResponse>, tonic::Status> {
        self.run_retry_access_grant_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses an access grant.
    async fn pause_access_grant_workflow(
        &self,
        request: tonic::Request<secrets::PauseAccessGrantWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::PauseAccessGrantWorkflowResponse>, tonic::Status> {
        self.run_pause_access_grant_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes an access grant.
    async fn resume_access_grant_workflow(
        &self,
        request: tonic::Request<secrets::ResumeAccessGrantWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::ResumeAccessGrantWorkflowResponse>, tonic::Status> {
        self.run_resume_access_grant_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an access grant.
    async fn inspect_access_grant_workflow(
        &self,
        request: tonic::Request<secrets::InspectAccessGrantWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::InspectAccessGrantWorkflowResponse>, tonic::Status> {
        self.run_inspect_access_grant_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists access grants a page at a time.
    async fn list_access_grant_workflows(
        &self,
        request: tonic::Request<secrets::ListAccessGrantWorkflowsRequest>,
    ) -> Result<tonic::Response<secrets::ListAccessGrantWorkflowsResponse>, tonic::Status> {
        self.run_list_access_grant_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
