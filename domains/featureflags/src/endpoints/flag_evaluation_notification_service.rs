use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_notification_service_server::FlagEvaluationNotificationService;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Tells the people who follow a flag evaluation what changed.
#[derive(Clone)]
pub struct FlagEvaluationNotificationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagEvaluationNotificationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SubscribeFlagEvaluationUpdates after validating the tenant and durably recording the command.
    async fn run_subscribe_flag_evaluation_updates(
        &self,
        req: featureflags::SubscribeFlagEvaluationUpdatesRequest,
    ) -> Result<featureflags::SubscribeFlagEvaluationUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SubscribeFlagEvaluationUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubscribeFlagEvaluationUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = featureflags::SubscribeFlagEvaluationUpdatesResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
    /// Run UnsubscribeFlagEvaluationUpdates after validating the tenant and durably recording the command.
    async fn run_unsubscribe_flag_evaluation_updates(
        &self,
        req: featureflags::UnsubscribeFlagEvaluationUpdatesRequest,
    ) -> Result<featureflags::UnsubscribeFlagEvaluationUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UnsubscribeFlagEvaluationUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnsubscribeFlagEvaluationUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = venues::CheckVenueEligibilityRequest {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::CheckVenueEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_venue_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::UnsubscribeFlagEvaluationUpdatesResponse::default();
        let contribution: String = reply_0
            .venue
            .clone()
            .unwrap_or_default()
            .dimensions
            .clone()
            .unwrap_or_default()
            .unit
            .clone();
        if contribution != <String>::default() {
            if response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .measurements
                .is_empty()
            {
                response
                    .flag_evaluation
                    .get_or_insert_with(Default::default)
                    .measurements
                    .push(Default::default());
            }
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .measurements[0]
                .unit = contribution;
        }
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
    /// Run NotifyFlagEvaluationWatchers after validating the tenant and durably recording the command.
    async fn run_notify_flag_evaluation_watchers(
        &self,
        req: featureflags::NotifyFlagEvaluationWatchersRequest,
    ) -> Result<featureflags::NotifyFlagEvaluationWatchersResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NotifyFlagEvaluationWatchers", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NotifyFlagEvaluationWatchers", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = featureflags::NotifyFlagEvaluationWatchersResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
    /// Run ListFlagEvaluationSubscriptions after validating the tenant and durably recording the command.
    async fn run_list_flag_evaluation_subscriptions(
        &self,
        req: featureflags::ListFlagEvaluationSubscriptionsRequest,
    ) -> Result<featureflags::ListFlagEvaluationSubscriptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFlagEvaluationSubscriptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFlagEvaluationSubscriptions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFlagEvaluationSubscriptionsResponse::default();
        Ok(response)
    }
    /// Run SendFlagEvaluationTestNotice after validating the tenant and durably recording the command.
    async fn run_send_flag_evaluation_test_notice(
        &self,
        req: featureflags::SendFlagEvaluationTestNoticeRequest,
    ) -> Result<featureflags::SendFlagEvaluationTestNoticeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SendFlagEvaluationTestNotice", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SendFlagEvaluationTestNotice", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = featureflags::SendFlagEvaluationTestNoticeResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
    /// Run GetFlagEvaluationPreferences after validating the tenant and durably recording the command.
    async fn run_get_flag_evaluation_preferences(
        &self,
        req: featureflags::GetFlagEvaluationPreferencesRequest,
    ) -> Result<featureflags::GetFlagEvaluationPreferencesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetFlagEvaluationPreferences", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagEvaluationPreferences", "running workflow");
        let mut response = featureflags::GetFlagEvaluationPreferencesResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamFlagEvaluationNotices after validating the tenant and durably recording the command.
    async fn run_stream_flag_evaluation_notices(
        &self,
        req: featureflags::StreamFlagEvaluationNoticesRequest,
    ) -> Result<featureflags::StreamFlagEvaluationNoticesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFlagEvaluationNotices", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFlagEvaluationNotices", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let response = featureflags::StreamFlagEvaluationNoticesResponse::default();
        Ok(response)
    }
}
acme_platform::endpoints! { FlagEvaluationNotificationService, FlagEvaluationNotificationController;
unary {
subscribe_flag_evaluation_updates (featureflags::SubscribeFlagEvaluationUpdatesRequest) -> featureflags::SubscribeFlagEvaluationUpdatesResponse => run_subscribe_flag_evaluation_updates;
unsubscribe_flag_evaluation_updates (featureflags::UnsubscribeFlagEvaluationUpdatesRequest) -> featureflags::UnsubscribeFlagEvaluationUpdatesResponse => run_unsubscribe_flag_evaluation_updates;
notify_flag_evaluation_watchers (featureflags::NotifyFlagEvaluationWatchersRequest) -> featureflags::NotifyFlagEvaluationWatchersResponse => run_notify_flag_evaluation_watchers;
list_flag_evaluation_subscriptions (featureflags::ListFlagEvaluationSubscriptionsRequest) -> featureflags::ListFlagEvaluationSubscriptionsResponse => run_list_flag_evaluation_subscriptions;
send_flag_evaluation_test_notice (featureflags::SendFlagEvaluationTestNoticeRequest) -> featureflags::SendFlagEvaluationTestNoticeResponse => run_send_flag_evaluation_test_notice;
get_flag_evaluation_preferences (featureflags::GetFlagEvaluationPreferencesRequest) -> featureflags::GetFlagEvaluationPreferencesResponse => run_get_flag_evaluation_preferences;
}
server {
stream_flag_evaluation_notices (featureflags::StreamFlagEvaluationNoticesRequest) -> featureflags::StreamFlagEvaluationNoticesResponse, StreamFlagEvaluationNoticesStream => run_stream_flag_evaluation_notices;
}
client {
}
bidi {
}
}
