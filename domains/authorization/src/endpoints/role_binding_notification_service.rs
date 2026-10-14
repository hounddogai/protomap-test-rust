use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_notification_service_server::RoleBindingNotificationService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;

/// Tells the people who follow a role binding what changed.
#[derive(Clone)]
pub struct RoleBindingNotificationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl RoleBindingNotificationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SubscribeRoleBindingUpdates after validating the tenant and durably recording the command.
    async fn run_subscribe_role_binding_updates(
        &self,
        req: authorization::SubscribeRoleBindingUpdatesRequest,
    ) -> Result<authorization::SubscribeRoleBindingUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SubscribeRoleBindingUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubscribeRoleBindingUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::SubscribeRoleBindingUpdatesResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run UnsubscribeRoleBindingUpdates after validating the tenant and durably recording the command.
    async fn run_unsubscribe_role_binding_updates(
        &self,
        req: authorization::UnsubscribeRoleBindingUpdatesRequest,
    ) -> Result<authorization::UnsubscribeRoleBindingUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UnsubscribeRoleBindingUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnsubscribeRoleBindingUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::UnsubscribeRoleBindingUpdatesResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run NotifyRoleBindingWatchers after validating the tenant and durably recording the command.
    async fn run_notify_role_binding_watchers(
        &self,
        req: authorization::NotifyRoleBindingWatchersRequest,
    ) -> Result<authorization::NotifyRoleBindingWatchersResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NotifyRoleBindingWatchers", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NotifyRoleBindingWatchers", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::NotifyRoleBindingWatchersResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run ListRoleBindingSubscriptions after validating the tenant and durably recording the command.
    async fn run_list_role_binding_subscriptions(
        &self,
        req: authorization::ListRoleBindingSubscriptionsRequest,
    ) -> Result<authorization::ListRoleBindingSubscriptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListRoleBindingSubscriptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRoleBindingSubscriptions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::EstimateFlagEvaluationRequest {
            context: req.context.clone(),
            details: Some(featureflags::estimate_flag_evaluation_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = FlagEvaluationCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::EstimateFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = authorization::ListRoleBindingSubscriptionsResponse::default();
        let contribution: String = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run SendRoleBindingTestNotice after validating the tenant and durably recording the command.
    async fn run_send_role_binding_test_notice(
        &self,
        req: authorization::SendRoleBindingTestNoticeRequest,
    ) -> Result<authorization::SendRoleBindingTestNoticeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SendRoleBindingTestNotice", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SendRoleBindingTestNotice", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::SendRoleBindingTestNoticeResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run GetRoleBindingPreferences after validating the tenant and durably recording the command.
    async fn run_get_role_binding_preferences(
        &self,
        req: authorization::GetRoleBindingPreferencesRequest,
    ) -> Result<authorization::GetRoleBindingPreferencesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetRoleBindingPreferences", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRoleBindingPreferences", "running workflow");
        let outgoing_0 = featureflags::SyncFlagEvaluationsRequest {
            context: req.context.clone(),
            details: Some(featureflags::sync_flag_evaluations_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                source: req.role_binding_id.clone(),
                subject: Some(featureflags::FlagRolloutRef {
                    starts_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::SyncFlagEvaluationsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.sync_flag_evaluations(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::GetRoleBindingPreferencesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamRoleBindingNotices after validating the tenant and durably recording the command.
    async fn run_stream_role_binding_notices(
        &self,
        req: authorization::StreamRoleBindingNoticesRequest,
    ) -> Result<authorization::StreamRoleBindingNoticesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamRoleBindingNotices", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamRoleBindingNotices", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ExplainFlagEvaluationDecisionRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_flag_evaluation_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    phone: req.subscriber.clone().unwrap_or_default().phone.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExplainFlagEvaluationDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_flag_evaluation_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::StreamRoleBindingNoticesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl RoleBindingNotificationService for RoleBindingNotificationController {
    /// Subscribes a role binding.
    async fn subscribe_role_binding_updates(
        &self,
        request: tonic::Request<authorization::SubscribeRoleBindingUpdatesRequest>,
    ) -> Result<tonic::Response<authorization::SubscribeRoleBindingUpdatesResponse>, tonic::Status> {
        self.run_subscribe_role_binding_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unsubscribes a role binding.
    async fn unsubscribe_role_binding_updates(
        &self,
        request: tonic::Request<authorization::UnsubscribeRoleBindingUpdatesRequest>,
    ) -> Result<tonic::Response<authorization::UnsubscribeRoleBindingUpdatesResponse>, tonic::Status> {
        self.run_unsubscribe_role_binding_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Notifys a role binding.
    async fn notify_role_binding_watchers(
        &self,
        request: tonic::Request<authorization::NotifyRoleBindingWatchersRequest>,
    ) -> Result<tonic::Response<authorization::NotifyRoleBindingWatchersResponse>, tonic::Status> {
        self.run_notify_role_binding_watchers(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists role bindings a page at a time.
    async fn list_role_binding_subscriptions(
        &self,
        request: tonic::Request<authorization::ListRoleBindingSubscriptionsRequest>,
    ) -> Result<tonic::Response<authorization::ListRoleBindingSubscriptionsResponse>, tonic::Status> {
        self.run_list_role_binding_subscriptions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Sends a role binding.
    async fn send_role_binding_test_notice(
        &self,
        request: tonic::Request<authorization::SendRoleBindingTestNoticeRequest>,
    ) -> Result<tonic::Response<authorization::SendRoleBindingTestNoticeResponse>, tonic::Status> {
        self.run_send_role_binding_test_notice(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a role binding.
    async fn get_role_binding_preferences(
        &self,
        request: tonic::Request<authorization::GetRoleBindingPreferencesRequest>,
    ) -> Result<tonic::Response<authorization::GetRoleBindingPreferencesResponse>, tonic::Status> {
        self.run_get_role_binding_preferences(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamRoleBindingNoticesStream =
        tokio_stream::wrappers::ReceiverStream<Result<authorization::StreamRoleBindingNoticesResponse, tonic::Status>>;
    /// Streams the changes of role bindings as they happen.
    async fn stream_role_binding_notices(
        &self,
        request: tonic::Request<authorization::StreamRoleBindingNoticesRequest>,
    ) -> Result<tonic::Response<Self::StreamRoleBindingNoticesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_role_binding_notices(request).await });
        Ok(tonic::Response::new(stream))
    }
}
