use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_history_service_server::FlagEvaluationHistoryService;
use acme_proto::acme::featureflags::v1::flag_rollout_export_service_client::FlagRolloutExportServiceClient;

/// Keeps every version of a flag evaluation and restores earlier ones.
#[derive(Clone)]
pub struct FlagEvaluationHistoryHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    record_flag_evaluation_version_0_port: acme_platform::ports::Callback<
        featureflags::UnsubscribeFeatureFlagUpdatesRequest,
        featureflags::UnsubscribeFeatureFlagUpdatesResponse,
    >,
}
impl FlagEvaluationHistoryHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::UnsubscribeFeatureFlagUpdatesRequest,
            featureflags::UnsubscribeFeatureFlagUpdatesResponse,
        > = {
            let client = FeatureFlagNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::UnsubscribeFeatureFlagUpdatesRequest| -> acme_platform::ports::RpcFuture<featureflags::UnsubscribeFeatureFlagUpdatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.unsubscribe_feature_flag_updates(request).await?.into_inner())
})
})
        };
        let record_flag_evaluation_version_0_port = callback;
        Self {
            channel,
            repository,
            record_flag_evaluation_version_0_port,
        }
    }
    /// Run RecordFlagEvaluationVersion after validating the tenant and durably recording the command.
    async fn run_record_flag_evaluation_version(
        &self,
        req: featureflags::RecordFlagEvaluationVersionRequest,
    ) -> Result<featureflags::RecordFlagEvaluationVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecordFlagEvaluationVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecordFlagEvaluationVersion", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::UnsubscribeFeatureFlagUpdatesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(featureflags::unsubscribe_feature_flag_updates_request::Details {
                recipient: Some(common::ContactInfo {
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 = (self.record_flag_evaluation_version_0_port)(outgoing_0).await?;
        let mut response = featureflags::RecordFlagEvaluationVersionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ListFlagEvaluationVersions after validating the tenant and durably recording the command.
    async fn run_list_flag_evaluation_versions(
        &self,
        req: featureflags::ListFlagEvaluationVersionsRequest,
    ) -> Result<featureflags::ListFlagEvaluationVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFlagEvaluationVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFlagEvaluationVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFlagEvaluationVersionsResponse::default();
        Ok(response)
    }
    /// Run GetFlagEvaluationVersion after validating the tenant and durably recording the command.
    async fn run_get_flag_evaluation_version(
        &self,
        req: featureflags::GetFlagEvaluationVersionRequest,
    ) -> Result<featureflags::GetFlagEvaluationVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetFlagEvaluationVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagEvaluationVersion", "running workflow");
        let outgoing_0 = featureflags::ShareFlagRolloutExportRequest {
            context: req.context.clone(),
            details: Some(featureflags::share_flag_rollout_export_request::Details {
                recipient: Some(common::ContactInfo {
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
                variables: (req.context.clone().unwrap_or_default().baggage.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::ShareFlagRolloutExportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_flag_rollout_export(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::GetFlagEvaluationVersionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run CompareFlagEvaluationVersions after validating the tenant and durably recording the command.
    async fn run_compare_flag_evaluation_versions(
        &self,
        req: featureflags::CompareFlagEvaluationVersionsRequest,
    ) -> Result<featureflags::CompareFlagEvaluationVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareFlagEvaluationVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareFlagEvaluationVersions", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::CompareFlagEvaluationVersionsResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.flag_evaluation.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run RevertFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_revert_flag_evaluation(
        &self,
        req: featureflags::RevertFlagEvaluationRequest,
    ) -> Result<featureflags::RevertFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RevertFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RevertFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::RevertFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.flag_evaluation.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run PruneFlagEvaluationVersions after validating the tenant and durably recording the command.
    async fn run_prune_flag_evaluation_versions(
        &self,
        req: featureflags::PruneFlagEvaluationVersionsRequest,
    ) -> Result<featureflags::PruneFlagEvaluationVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PruneFlagEvaluationVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PruneFlagEvaluationVersions", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation_id.clone(), "recorded audit field");
        let response = featureflags::PruneFlagEvaluationVersionsResponse::default();
        Ok(response)
    }
}
#[tonic::async_trait]
impl FlagEvaluationHistoryService for FlagEvaluationHistoryHandler {
    /// Records a flag evaluation.
    async fn record_flag_evaluation_version(
        &self,
        request: tonic::Request<featureflags::RecordFlagEvaluationVersionRequest>,
    ) -> Result<tonic::Response<featureflags::RecordFlagEvaluationVersionResponse>, tonic::Status> {
        self.run_record_flag_evaluation_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag evaluations a page at a time.
    async fn list_flag_evaluation_versions(
        &self,
        request: tonic::Request<featureflags::ListFlagEvaluationVersionsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFlagEvaluationVersionsResponse>, tonic::Status> {
        self.run_list_flag_evaluation_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag evaluation.
    async fn get_flag_evaluation_version(
        &self,
        request: tonic::Request<featureflags::GetFlagEvaluationVersionRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagEvaluationVersionResponse>, tonic::Status> {
        self.run_get_flag_evaluation_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a flag evaluation.
    async fn compare_flag_evaluation_versions(
        &self,
        request: tonic::Request<featureflags::CompareFlagEvaluationVersionsRequest>,
    ) -> Result<tonic::Response<featureflags::CompareFlagEvaluationVersionsResponse>, tonic::Status> {
        self.run_compare_flag_evaluation_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reverts a flag evaluation.
    async fn revert_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::RevertFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::RevertFlagEvaluationResponse>, tonic::Status> {
        self.run_revert_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a flag evaluation.
    async fn prune_flag_evaluation_versions(
        &self,
        request: tonic::Request<featureflags::PruneFlagEvaluationVersionsRequest>,
    ) -> Result<tonic::Response<featureflags::PruneFlagEvaluationVersionsResponse>, tonic::Status> {
        self.run_prune_flag_evaluation_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
