use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_export_service_client::PermissionCheckExportServiceClient;
use acme_proto::acme::authorization::v1::permission_check_notification_service_server::PermissionCheckNotificationService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient;

/// Tells the people who follow a permission check what changed.
#[derive(Clone)]
pub struct PermissionCheckNotificationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    notify_permission_check_watchers_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                authorization::RetryPermissionCheckExportRequest,
                authorization::RetryPermissionCheckExportResponse,
            >,
    >,
}
impl PermissionCheckNotificationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            authorization::RetryPermissionCheckExportRequest,
            authorization::RetryPermissionCheckExportResponse,
        > = {
            let client = PermissionCheckExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::RetryPermissionCheckExportRequest| -> acme_platform::ports::RpcFuture<authorization::RetryPermissionCheckExportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.retry_permission_check_export(request).await?.into_inner())
})
})
        };
        let notify_permission_check_watchers_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            notify_permission_check_watchers_0_port,
        }
    }
    /// Run SubscribePermissionCheckUpdates after validating the tenant and durably recording the command.
    async fn run_subscribe_permission_check_updates(
        &self,
        req: authorization::SubscribePermissionCheckUpdatesRequest,
    ) -> Result<authorization::SubscribePermissionCheckUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SubscribePermissionCheckUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubscribePermissionCheckUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::SubscribePermissionCheckUpdatesResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
    /// Run UnsubscribePermissionCheckUpdates after validating the tenant and durably recording the command.
    async fn run_unsubscribe_permission_check_updates(
        &self,
        req: authorization::UnsubscribePermissionCheckUpdatesRequest,
    ) -> Result<authorization::UnsubscribePermissionCheckUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UnsubscribePermissionCheckUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnsubscribePermissionCheckUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::UnsubscribePermissionCheckUpdatesResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
    /// Run NotifyPermissionCheckWatchers after validating the tenant and durably recording the command.
    async fn run_notify_permission_check_watchers(
        &self,
        req: authorization::NotifyPermissionCheckWatchersRequest,
    ) -> Result<authorization::NotifyPermissionCheckWatchersResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NotifyPermissionCheckWatchers", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NotifyPermissionCheckWatchers", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().preferred_channel.clone(), "recorded audit field");
        let outgoing_0 = authorization::RetryPermissionCheckExportRequest {
            context: req.context.clone(),
            details: Some(authorization::retry_permission_check_export_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
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
            acme_platform::ports::execute(self.notify_permission_check_watchers_0_port.as_ref(), outgoing_0).await?;
        let mut response = authorization::NotifyPermissionCheckWatchersResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
    /// Run ListPermissionCheckSubscriptions after validating the tenant and durably recording the command.
    async fn run_list_permission_check_subscriptions(
        &self,
        req: authorization::ListPermissionCheckSubscriptionsRequest,
    ) -> Result<authorization::ListPermissionCheckSubscriptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPermissionCheckSubscriptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPermissionCheckSubscriptions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::ListPermissionCheckSubscriptionsResponse::default();
        Ok(response)
    }
    /// Run SendPermissionCheckTestNotice after validating the tenant and durably recording the command.
    async fn run_send_permission_check_test_notice(
        &self,
        req: authorization::SendPermissionCheckTestNoticeRequest,
    ) -> Result<authorization::SendPermissionCheckTestNoticeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SendPermissionCheckTestNotice", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SendPermissionCheckTestNotice", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::SendPermissionCheckTestNoticeResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
    /// Run GetPermissionCheckPreferences after validating the tenant and durably recording the command.
    async fn run_get_permission_check_preferences(
        &self,
        req: authorization::GetPermissionCheckPreferencesRequest,
    ) -> Result<authorization::GetPermissionCheckPreferencesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetPermissionCheckPreferences", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetPermissionCheckPreferences", "running workflow");
        let outgoing_0 = featureflags::RepairFlagEvaluationRequest {
            context: req.context.clone(),
            details: Some(featureflags::repair_flag_evaluation_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                correlation_id: req
                    .context
                    .clone()
                    .unwrap_or_default()
                    .actor
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                measurements: vec![featureflags::FlagEvaluationMeasurement {
                    observed_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }],
                source: req.permission_check_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationAdminServiceClient::new(channel);
            let input: tonic::Request<featureflags::RepairFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.repair_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::GetPermissionCheckPreferencesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamPermissionCheckNotices after validating the tenant and durably recording the command.
    async fn run_stream_permission_check_notices(
        &self,
        req: authorization::StreamPermissionCheckNoticesRequest,
    ) -> Result<authorization::StreamPermissionCheckNoticesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamPermissionCheckNotices", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamPermissionCheckNotices", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let response = authorization::StreamPermissionCheckNoticesResponse::default();
        Ok(response)
    }
}
#[tonic::async_trait]
impl PermissionCheckNotificationService for PermissionCheckNotificationHandler {
    /// Subscribes a permission check.
    async fn subscribe_permission_check_updates(
        &self,
        request: tonic::Request<authorization::SubscribePermissionCheckUpdatesRequest>,
    ) -> Result<tonic::Response<authorization::SubscribePermissionCheckUpdatesResponse>, tonic::Status> {
        self.run_subscribe_permission_check_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unsubscribes a permission check.
    async fn unsubscribe_permission_check_updates(
        &self,
        request: tonic::Request<authorization::UnsubscribePermissionCheckUpdatesRequest>,
    ) -> Result<tonic::Response<authorization::UnsubscribePermissionCheckUpdatesResponse>, tonic::Status> {
        self.run_unsubscribe_permission_check_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Notifys a permission check.
    async fn notify_permission_check_watchers(
        &self,
        request: tonic::Request<authorization::NotifyPermissionCheckWatchersRequest>,
    ) -> Result<tonic::Response<authorization::NotifyPermissionCheckWatchersResponse>, tonic::Status> {
        self.run_notify_permission_check_watchers(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists permission checks a page at a time.
    async fn list_permission_check_subscriptions(
        &self,
        request: tonic::Request<authorization::ListPermissionCheckSubscriptionsRequest>,
    ) -> Result<tonic::Response<authorization::ListPermissionCheckSubscriptionsResponse>, tonic::Status> {
        self.run_list_permission_check_subscriptions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Sends a permission check.
    async fn send_permission_check_test_notice(
        &self,
        request: tonic::Request<authorization::SendPermissionCheckTestNoticeRequest>,
    ) -> Result<tonic::Response<authorization::SendPermissionCheckTestNoticeResponse>, tonic::Status> {
        self.run_send_permission_check_test_notice(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a permission check.
    async fn get_permission_check_preferences(
        &self,
        request: tonic::Request<authorization::GetPermissionCheckPreferencesRequest>,
    ) -> Result<tonic::Response<authorization::GetPermissionCheckPreferencesResponse>, tonic::Status> {
        self.run_get_permission_check_preferences(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamPermissionCheckNoticesStream = tokio_stream::wrappers::ReceiverStream<
        Result<authorization::StreamPermissionCheckNoticesResponse, tonic::Status>,
    >;
    /// Streams the changes of permission checks as they happen.
    async fn stream_permission_check_notices(
        &self,
        request: tonic::Request<authorization::StreamPermissionCheckNoticesRequest>,
    ) -> Result<tonic::Response<Self::StreamPermissionCheckNoticesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(
                async move { handler.run_stream_permission_check_notices(request).await },
            );
        Ok(tonic::Response::new(stream))
    }
}
