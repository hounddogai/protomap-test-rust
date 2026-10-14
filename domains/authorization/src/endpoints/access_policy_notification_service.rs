use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_notification_service_server::AccessPolicyNotificationService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_client::FlagRolloutQueryServiceClient;

/// Tells the people who follow an access policy what changed.
#[derive(Clone)]
pub struct AccessPolicyNotificationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl AccessPolicyNotificationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SubscribeAccessPolicyUpdates after validating the tenant and durably recording the command.
    async fn run_subscribe_access_policy_updates(
        &self,
        req: authorization::SubscribeAccessPolicyUpdatesRequest,
    ) -> Result<authorization::SubscribeAccessPolicyUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SubscribeAccessPolicyUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubscribeAccessPolicyUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = authorization::SubscribeAccessPolicyUpdatesResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
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
                .access_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run UnsubscribeAccessPolicyUpdates after validating the tenant and durably recording the command.
    async fn run_unsubscribe_access_policy_updates(
        &self,
        req: authorization::UnsubscribeAccessPolicyUpdatesRequest,
    ) -> Result<authorization::UnsubscribeAccessPolicyUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UnsubscribeAccessPolicyUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnsubscribeAccessPolicyUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = authorization::UnsubscribeAccessPolicyUpdatesResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
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
                .access_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListAccessPolicySubscriptions after validating the tenant and durably recording the command.
    async fn run_list_access_policy_subscriptions(
        &self,
        req: authorization::ListAccessPolicySubscriptionsRequest,
    ) -> Result<authorization::ListAccessPolicySubscriptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListAccessPolicySubscriptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListAccessPolicySubscriptions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::ListAccessPolicySubscriptionsResponse::default();
        Ok(response)
    }
    /// Run SendAccessPolicyTestNotice after validating the tenant and durably recording the command.
    async fn run_send_access_policy_test_notice(
        &self,
        req: authorization::SendAccessPolicyTestNoticeRequest,
    ) -> Result<authorization::SendAccessPolicyTestNoticeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SendAccessPolicyTestNotice", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SendAccessPolicyTestNotice", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = authorization::SendAccessPolicyTestNoticeResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
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
                .access_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run GetAccessPolicyPreferences after validating the tenant and durably recording the command.
    async fn run_get_access_policy_preferences(
        &self,
        req: authorization::GetAccessPolicyPreferencesRequest,
    ) -> Result<authorization::GetAccessPolicyPreferencesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetAccessPolicyPreferences", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetAccessPolicyPreferences", "running workflow");
        let outgoing_0 = featureflags::BrowseFlagRolloutsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::browse_flag_rollouts_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.access_policy_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::BrowseFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_flag_rollouts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::GetAccessPolicyPreferencesResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamAccessPolicyNotices after validating the tenant and durably recording the command.
    async fn run_stream_access_policy_notices(
        &self,
        req: authorization::StreamAccessPolicyNoticesRequest,
    ) -> Result<authorization::StreamAccessPolicyNoticesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamAccessPolicyNotices", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamAccessPolicyNotices", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let mut response = authorization::StreamAccessPolicyNoticesResponse::default();
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { AccessPolicyNotificationService, AccessPolicyNotificationController;
unary {
subscribe_access_policy_updates (authorization::SubscribeAccessPolicyUpdatesRequest) -> authorization::SubscribeAccessPolicyUpdatesResponse => run_subscribe_access_policy_updates;
unsubscribe_access_policy_updates (authorization::UnsubscribeAccessPolicyUpdatesRequest) -> authorization::UnsubscribeAccessPolicyUpdatesResponse => run_unsubscribe_access_policy_updates;
list_access_policy_subscriptions (authorization::ListAccessPolicySubscriptionsRequest) -> authorization::ListAccessPolicySubscriptionsResponse => run_list_access_policy_subscriptions;
send_access_policy_test_notice (authorization::SendAccessPolicyTestNoticeRequest) -> authorization::SendAccessPolicyTestNoticeResponse => run_send_access_policy_test_notice;
get_access_policy_preferences (authorization::GetAccessPolicyPreferencesRequest) -> authorization::GetAccessPolicyPreferencesResponse => run_get_access_policy_preferences;
}
server {
stream_access_policy_notices (authorization::StreamAccessPolicyNoticesRequest) -> authorization::StreamAccessPolicyNoticesResponse, StreamAccessPolicyNoticesStream => run_stream_access_policy_notices;
}
client {
}
bidi {
}
}
