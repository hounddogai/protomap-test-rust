use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_admin_service_client::FeatureFlagAdminServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_server::FeatureFlagNotificationService;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Tells the people who follow a feature flag what changed.
#[derive(Clone)]
pub struct FeatureFlagNotificationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    get_feature_flag_preferences_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                featureflags::CalculateFlagEvaluationRequest,
                featureflags::CalculateFlagEvaluationResponse,
            >,
    >,
    send_feature_flag_test_notice_0_port:
        acme_platform::ports::Callback<venues::UpdateVenueRequest, venues::UpdateVenueResponse>,
    unsubscribe_feature_flag_updates_0_port:
        acme_platform::ports::Callback<featureflags::UnlockFeatureFlagRequest, featureflags::UnlockFeatureFlagResponse>,
}
impl FeatureFlagNotificationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::UnlockFeatureFlagRequest,
            featureflags::UnlockFeatureFlagResponse,
        > = {
            let client = FeatureFlagAdminServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::UnlockFeatureFlagRequest| -> acme_platform::ports::RpcFuture<featureflags::UnlockFeatureFlagResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.unlock_feature_flag(request).await?.into_inner())
})
})
        };
        let unsubscribe_feature_flag_updates_0_port = callback;
        let callback: acme_platform::ports::Callback<venues::UpdateVenueRequest, venues::UpdateVenueResponse> = {
            let client = VenueServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: venues::UpdateVenueRequest| -> acme_platform::ports::RpcFuture<venues::UpdateVenueResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.update_venue(request).await?.into_inner())
})
})
        };
        let send_feature_flag_test_notice_0_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::CalculateFlagEvaluationRequest,
            featureflags::CalculateFlagEvaluationResponse,
        > = {
            let client = FlagEvaluationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::CalculateFlagEvaluationRequest| -> acme_platform::ports::RpcFuture<featureflags::CalculateFlagEvaluationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_flag_evaluation(request).await?.into_inner())
})
})
        };
        let get_feature_flag_preferences_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            get_feature_flag_preferences_0_port,
            send_feature_flag_test_notice_0_port,
            unsubscribe_feature_flag_updates_0_port,
        }
    }
    /// Run SubscribeFeatureFlagUpdates after validating the tenant and durably recording the command.
    async fn run_subscribe_feature_flag_updates(
        &self,
        req: featureflags::SubscribeFeatureFlagUpdatesRequest,
    ) -> Result<featureflags::SubscribeFeatureFlagUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SubscribeFeatureFlagUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubscribeFeatureFlagUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = featureflags::SubscribeFeatureFlagUpdatesResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run UnsubscribeFeatureFlagUpdates after validating the tenant and durably recording the command.
    async fn run_unsubscribe_feature_flag_updates(
        &self,
        req: featureflags::UnsubscribeFeatureFlagUpdatesRequest,
    ) -> Result<featureflags::UnsubscribeFeatureFlagUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UnsubscribeFeatureFlagUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnsubscribeFeatureFlagUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = featureflags::UnlockFeatureFlagRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(featureflags::unlock_feature_flag_request::Details {
                notify_contact: Some(common::ContactInfo {
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
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            feature_flag_id: req.feature_flag_id.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.unsubscribe_feature_flag_updates_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = featureflags::UnsubscribeFeatureFlagUpdatesResponse::default();
        let contribution: Option<identity::UserRef> = reply_0.feature_flag.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.feature_flag.get_or_insert_with(Default::default).owner = contribution;
        }
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        Ok(response)
    }
    /// Run NotifyFeatureFlagWatchers after validating the tenant and durably recording the command.
    async fn run_notify_feature_flag_watchers(
        &self,
        req: featureflags::NotifyFeatureFlagWatchersRequest,
    ) -> Result<featureflags::NotifyFeatureFlagWatchersResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NotifyFeatureFlagWatchers", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NotifyFeatureFlagWatchers", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = featureflags::NotifyFeatureFlagWatchersResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListFeatureFlagSubscriptions after validating the tenant and durably recording the command.
    async fn run_list_feature_flag_subscriptions(
        &self,
        req: featureflags::ListFeatureFlagSubscriptionsRequest,
    ) -> Result<featureflags::ListFeatureFlagSubscriptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFeatureFlagSubscriptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFeatureFlagSubscriptions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFeatureFlagSubscriptionsResponse::default();
        Ok(response)
    }
    /// Run SendFeatureFlagTestNotice after validating the tenant and durably recording the command.
    async fn run_send_feature_flag_test_notice(
        &self,
        req: featureflags::SendFeatureFlagTestNoticeRequest,
    ) -> Result<featureflags::SendFeatureFlagTestNoticeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SendFeatureFlagTestNotice", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SendFeatureFlagTestNotice", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = venues::UpdateVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
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
            ..Default::default()
        };
        let reply_0 = (self.send_feature_flag_test_notice_0_port)(outgoing_0).await?;
        let mut response = featureflags::SendFeatureFlagTestNoticeResponse::default();
        let contribution: String = reply_0
            .venue
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run GetFeatureFlagPreferences after validating the tenant and durably recording the command.
    async fn run_get_feature_flag_preferences(
        &self,
        req: featureflags::GetFeatureFlagPreferencesRequest,
    ) -> Result<featureflags::GetFeatureFlagPreferencesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetFeatureFlagPreferences", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFeatureFlagPreferences", "running workflow");
        let outgoing_0 = featureflags::CalculateFlagEvaluationRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                source: req.feature_flag_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.get_feature_flag_preferences_0_port.as_ref(), outgoing_0).await?;
        let mut response = featureflags::GetFeatureFlagPreferencesResponse::default();
        let contribution: String = reply_0
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
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamFeatureFlagNotices after validating the tenant and durably recording the command.
    async fn run_stream_feature_flag_notices(
        &self,
        req: featureflags::StreamFeatureFlagNoticesRequest,
    ) -> Result<featureflags::StreamFeatureFlagNoticesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFeatureFlagNotices", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFeatureFlagNotices", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = featureflags::PushFlagEvaluationRequest {
            annotations: (req.filter.clone().unwrap_or_default().labels.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                correlation_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                measurements: vec![featureflags::FlagEvaluationMeasurement {
                    observed_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }],
                subject: Some(featureflags::FlagRolloutRef {
                    starts_at: req.since.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::PushFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::StreamFeatureFlagNoticesResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FeatureFlagNotificationService for FeatureFlagNotificationHandler {
    /// Subscribes a feature flag.
    async fn subscribe_feature_flag_updates(
        &self,
        request: tonic::Request<featureflags::SubscribeFeatureFlagUpdatesRequest>,
    ) -> Result<tonic::Response<featureflags::SubscribeFeatureFlagUpdatesResponse>, tonic::Status> {
        self.run_subscribe_feature_flag_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unsubscribes a feature flag.
    async fn unsubscribe_feature_flag_updates(
        &self,
        request: tonic::Request<featureflags::UnsubscribeFeatureFlagUpdatesRequest>,
    ) -> Result<tonic::Response<featureflags::UnsubscribeFeatureFlagUpdatesResponse>, tonic::Status> {
        self.run_unsubscribe_feature_flag_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Notifys a feature flag.
    async fn notify_feature_flag_watchers(
        &self,
        request: tonic::Request<featureflags::NotifyFeatureFlagWatchersRequest>,
    ) -> Result<tonic::Response<featureflags::NotifyFeatureFlagWatchersResponse>, tonic::Status> {
        self.run_notify_feature_flag_watchers(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists feature flags a page at a time.
    async fn list_feature_flag_subscriptions(
        &self,
        request: tonic::Request<featureflags::ListFeatureFlagSubscriptionsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFeatureFlagSubscriptionsResponse>, tonic::Status> {
        self.run_list_feature_flag_subscriptions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Sends a feature flag.
    async fn send_feature_flag_test_notice(
        &self,
        request: tonic::Request<featureflags::SendFeatureFlagTestNoticeRequest>,
    ) -> Result<tonic::Response<featureflags::SendFeatureFlagTestNoticeResponse>, tonic::Status> {
        self.run_send_feature_flag_test_notice(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a feature flag.
    async fn get_feature_flag_preferences(
        &self,
        request: tonic::Request<featureflags::GetFeatureFlagPreferencesRequest>,
    ) -> Result<tonic::Response<featureflags::GetFeatureFlagPreferencesResponse>, tonic::Status> {
        self.run_get_feature_flag_preferences(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamFeatureFlagNoticesStream =
        tokio_stream::wrappers::ReceiverStream<Result<featureflags::StreamFeatureFlagNoticesResponse, tonic::Status>>;
    /// Streams the changes of feature flags as they happen.
    async fn stream_feature_flag_notices(
        &self,
        request: tonic::Request<featureflags::StreamFeatureFlagNoticesRequest>,
    ) -> Result<tonic::Response<Self::StreamFeatureFlagNoticesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_feature_flag_notices(request).await });
        Ok(tonic::Response::new(stream))
    }
}
