use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_server::FlagRolloutNotificationService;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Tells the people who follow a flag rollout what changed.
#[derive(Clone)]
pub struct FlagRolloutNotificationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagRolloutNotificationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SubscribeFlagRolloutUpdates after validating the tenant and durably recording the command.
    async fn run_subscribe_flag_rollout_updates(
        &self,
        req: featureflags::SubscribeFlagRolloutUpdatesRequest,
    ) -> Result<featureflags::SubscribeFlagRolloutUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SubscribeFlagRolloutUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubscribeFlagRolloutUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = featureflags::SubscribeFlagRolloutUpdatesResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
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
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run UnsubscribeFlagRolloutUpdates after validating the tenant and durably recording the command.
    async fn run_unsubscribe_flag_rollout_updates(
        &self,
        req: featureflags::UnsubscribeFlagRolloutUpdatesRequest,
    ) -> Result<featureflags::UnsubscribeFlagRolloutUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UnsubscribeFlagRolloutUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnsubscribeFlagRolloutUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = featureflags::UnsubscribeFlagRolloutUpdatesResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
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
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run NotifyFlagRolloutWatchers after validating the tenant and durably recording the command.
    async fn run_notify_flag_rollout_watchers(
        &self,
        req: featureflags::NotifyFlagRolloutWatchersRequest,
    ) -> Result<featureflags::NotifyFlagRolloutWatchersResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NotifyFlagRolloutWatchers", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NotifyFlagRolloutWatchers", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = featureflags::NotifyFlagRolloutWatchersResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
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
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListFlagRolloutSubscriptions after validating the tenant and durably recording the command.
    async fn run_list_flag_rollout_subscriptions(
        &self,
        req: featureflags::ListFlagRolloutSubscriptionsRequest,
    ) -> Result<featureflags::ListFlagRolloutSubscriptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFlagRolloutSubscriptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFlagRolloutSubscriptions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFlagRolloutSubscriptionsResponse::default();
        Ok(response)
    }
    /// Run SendFlagRolloutTestNotice after validating the tenant and durably recording the command.
    async fn run_send_flag_rollout_test_notice(
        &self,
        req: featureflags::SendFlagRolloutTestNoticeRequest,
    ) -> Result<featureflags::SendFlagRolloutTestNoticeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SendFlagRolloutTestNotice", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SendFlagRolloutTestNotice", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = featureflags::SendFlagRolloutTestNoticeResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
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
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run GetFlagRolloutPreferences after validating the tenant and durably recording the command.
    async fn run_get_flag_rollout_preferences(
        &self,
        req: featureflags::GetFlagRolloutPreferencesRequest,
    ) -> Result<featureflags::GetFlagRolloutPreferencesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetFlagRolloutPreferences", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagRolloutPreferences", "running workflow");
        let outgoing_0 = venues::UpdateVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<venues::UpdateVenueRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::UpdateVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::GetFlagRolloutPreferencesResponse::default();
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
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamFlagRolloutNotices after validating the tenant and durably recording the command.
    async fn run_stream_flag_rollout_notices(
        &self,
        req: featureflags::StreamFlagRolloutNoticesRequest,
    ) -> Result<featureflags::StreamFlagRolloutNoticesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFlagRolloutNotices", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFlagRolloutNotices", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let command_0 = StreamFlagRolloutNoticesDispatch0::from(req.clone());
        let outgoing_0: venues::CloneVenueRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::CloneVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::StreamFlagRolloutNoticesResponse::default();
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
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .venue
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by StreamFlagRolloutNotices for CloneVenue.
struct StreamFlagRolloutNoticesDispatch0 {
    context: Option<common::RequestContext>,
    venue_address_region: String,
}
impl From<featureflags::StreamFlagRolloutNoticesRequest> for StreamFlagRolloutNoticesDispatch0 {
    fn from(req: featureflags::StreamFlagRolloutNoticesRequest) -> Self {
        Self {
            context: req.context.clone(),
            venue_address_region: req.filter.clone().unwrap_or_default().region.clone(),
        }
    }
}
impl From<StreamFlagRolloutNoticesDispatch0> for venues::CloneVenueRequest {
    fn from(value: StreamFlagRolloutNoticesDispatch0) -> Self {
        venues::CloneVenueRequest {
            context: value.context,
            venue: Some(venues::Venue {
                locator: Some(venues::venue::Locator::Address(common::PostalAddress {
                    region: value.venue_address_region,
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl FlagRolloutNotificationService for FlagRolloutNotificationHandler {
    /// Subscribes a flag rollout.
    async fn subscribe_flag_rollout_updates(
        &self,
        request: tonic::Request<featureflags::SubscribeFlagRolloutUpdatesRequest>,
    ) -> Result<tonic::Response<featureflags::SubscribeFlagRolloutUpdatesResponse>, tonic::Status> {
        self.run_subscribe_flag_rollout_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unsubscribes a flag rollout.
    async fn unsubscribe_flag_rollout_updates(
        &self,
        request: tonic::Request<featureflags::UnsubscribeFlagRolloutUpdatesRequest>,
    ) -> Result<tonic::Response<featureflags::UnsubscribeFlagRolloutUpdatesResponse>, tonic::Status> {
        self.run_unsubscribe_flag_rollout_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Notifys a flag rollout.
    async fn notify_flag_rollout_watchers(
        &self,
        request: tonic::Request<featureflags::NotifyFlagRolloutWatchersRequest>,
    ) -> Result<tonic::Response<featureflags::NotifyFlagRolloutWatchersResponse>, tonic::Status> {
        self.run_notify_flag_rollout_watchers(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag rollouts a page at a time.
    async fn list_flag_rollout_subscriptions(
        &self,
        request: tonic::Request<featureflags::ListFlagRolloutSubscriptionsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFlagRolloutSubscriptionsResponse>, tonic::Status> {
        self.run_list_flag_rollout_subscriptions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Sends a flag rollout.
    async fn send_flag_rollout_test_notice(
        &self,
        request: tonic::Request<featureflags::SendFlagRolloutTestNoticeRequest>,
    ) -> Result<tonic::Response<featureflags::SendFlagRolloutTestNoticeResponse>, tonic::Status> {
        self.run_send_flag_rollout_test_notice(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag rollout.
    async fn get_flag_rollout_preferences(
        &self,
        request: tonic::Request<featureflags::GetFlagRolloutPreferencesRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagRolloutPreferencesResponse>, tonic::Status> {
        self.run_get_flag_rollout_preferences(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamFlagRolloutNoticesStream =
        tokio_stream::wrappers::ReceiverStream<Result<featureflags::StreamFlagRolloutNoticesResponse, tonic::Status>>;
    /// Streams the changes of flag rollouts as they happen.
    async fn stream_flag_rollout_notices(
        &self,
        request: tonic::Request<featureflags::StreamFlagRolloutNoticesRequest>,
    ) -> Result<tonic::Response<Self::StreamFlagRolloutNoticesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_flag_rollout_notices(request).await });
        Ok(tonic::Response::new(stream))
    }
}
