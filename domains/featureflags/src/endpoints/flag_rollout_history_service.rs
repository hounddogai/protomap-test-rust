use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_history_service_server::FlagRolloutHistoryService;
use acme_proto::acme::identity::v1 as identity;

/// Keeps every version of a flag rollout and restores earlier ones.
#[derive(Clone)]
pub struct FlagRolloutHistoryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagRolloutHistoryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run RecordFlagRolloutVersion after validating the tenant and durably recording the command.
    async fn run_record_flag_rollout_version(
        &self,
        req: featureflags::RecordFlagRolloutVersionRequest,
    ) -> Result<featureflags::RecordFlagRolloutVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecordFlagRolloutVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecordFlagRolloutVersion", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::RecordFlagRolloutVersionResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.flag_rollout.clone().unwrap_or_default().organizer.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).organizer = contribution;
        }
        Ok(response)
    }
    /// Run ListFlagRolloutVersions after validating the tenant and durably recording the command.
    async fn run_list_flag_rollout_versions(
        &self,
        req: featureflags::ListFlagRolloutVersionsRequest,
    ) -> Result<featureflags::ListFlagRolloutVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFlagRolloutVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFlagRolloutVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFlagRolloutVersionsResponse::default();
        Ok(response)
    }
    /// Run GetFlagRolloutVersion after validating the tenant and durably recording the command.
    async fn run_get_flag_rollout_version(
        &self,
        req: featureflags::GetFlagRolloutVersionRequest,
    ) -> Result<featureflags::GetFlagRolloutVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFlagRolloutVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagRolloutVersion", "running workflow");
        let mut response = featureflags::GetFlagRolloutVersionResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        Ok(response)
    }
    /// Run CompareFlagRolloutVersions after validating the tenant and durably recording the command.
    async fn run_compare_flag_rollout_versions(
        &self,
        req: featureflags::CompareFlagRolloutVersionsRequest,
    ) -> Result<featureflags::CompareFlagRolloutVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareFlagRolloutVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareFlagRolloutVersions", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::CompareFlagRolloutVersionsResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.flag_rollout.clone().unwrap_or_default().window.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).window = contribution;
        }
        Ok(response)
    }
    /// Run RevertFlagRollout after validating the tenant and durably recording the command.
    async fn run_revert_flag_rollout(
        &self,
        req: featureflags::RevertFlagRolloutRequest,
    ) -> Result<featureflags::RevertFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RevertFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RevertFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::NotifyFeatureFlagWatchersRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(featureflags::notify_feature_flag_watchers_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .organizer
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::NotifyFeatureFlagWatchersRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.notify_feature_flag_watchers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::RevertFlagRolloutResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.flag_rollout.clone().unwrap_or_default().organizer.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).organizer = contribution;
        }
        Ok(response)
    }
    /// Run PruneFlagRolloutVersions after validating the tenant and durably recording the command.
    async fn run_prune_flag_rollout_versions(
        &self,
        req: featureflags::PruneFlagRolloutVersionsRequest,
    ) -> Result<featureflags::PruneFlagRolloutVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PruneFlagRolloutVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PruneFlagRolloutVersions", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout_id.clone(), "recorded audit field");
        let response = featureflags::PruneFlagRolloutVersionsResponse::default();
        Ok(response)
    }
}
#[tonic::async_trait]
impl FlagRolloutHistoryService for FlagRolloutHistoryEndpoint {
    /// Records a flag rollout.
    async fn record_flag_rollout_version(
        &self,
        request: tonic::Request<featureflags::RecordFlagRolloutVersionRequest>,
    ) -> Result<tonic::Response<featureflags::RecordFlagRolloutVersionResponse>, tonic::Status> {
        self.run_record_flag_rollout_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag rollouts a page at a time.
    async fn list_flag_rollout_versions(
        &self,
        request: tonic::Request<featureflags::ListFlagRolloutVersionsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFlagRolloutVersionsResponse>, tonic::Status> {
        self.run_list_flag_rollout_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag rollout.
    async fn get_flag_rollout_version(
        &self,
        request: tonic::Request<featureflags::GetFlagRolloutVersionRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagRolloutVersionResponse>, tonic::Status> {
        self.run_get_flag_rollout_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a flag rollout.
    async fn compare_flag_rollout_versions(
        &self,
        request: tonic::Request<featureflags::CompareFlagRolloutVersionsRequest>,
    ) -> Result<tonic::Response<featureflags::CompareFlagRolloutVersionsResponse>, tonic::Status> {
        self.run_compare_flag_rollout_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reverts a flag rollout.
    async fn revert_flag_rollout(
        &self,
        request: tonic::Request<featureflags::RevertFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::RevertFlagRolloutResponse>, tonic::Status> {
        self.run_revert_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a flag rollout.
    async fn prune_flag_rollout_versions(
        &self,
        request: tonic::Request<featureflags::PruneFlagRolloutVersionsRequest>,
    ) -> Result<tonic::Response<featureflags::PruneFlagRolloutVersionsResponse>, tonic::Status> {
        self.run_prune_flag_rollout_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
