use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_server::FlagRolloutQueryService;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;

/// Answers read-only questions about flag rollouts, for screens and reports.
#[derive(Clone)]
pub struct FlagRolloutQueryController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagRolloutQueryController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run LookupFlagRollout after validating the tenant and durably recording the command.
    async fn run_lookup_flag_rollout(
        &self,
        req: featureflags::LookupFlagRolloutRequest,
    ) -> Result<featureflags::LookupFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupFlagRollout", "running workflow");
        let mut response = featureflags::LookupFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseFlagRollouts after validating the tenant and durably recording the command.
    async fn run_browse_flag_rollouts(
        &self,
        req: featureflags::BrowseFlagRolloutsRequest,
    ) -> Result<featureflags::BrowseFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFlagRolloutConflictsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_rollout_conflicts_request::Filter {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                statuses: req.filter.clone().unwrap_or_default().statuses.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.featureflags.v1.ListFlagRolloutConflictsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ListFlagRolloutConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_flag_rollout_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::BrowseFlagRolloutsResponse::default();
        let contribution: i64 = reply_0.page_info.clone().unwrap_or_default().total_size.clone();
        if contribution != <i64>::default() {
            response.page_info.get_or_insert_with(Default::default).total_size = contribution;
        }
        Ok(response)
    }
    /// Run FindFlagRollouts after validating the tenant and durably recording the command.
    async fn run_find_flag_rollouts(
        &self,
        req: featureflags::FindFlagRolloutsRequest,
    ) -> Result<featureflags::FindFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::FindFlagRolloutsResponse::default();
        Ok(response)
    }
    /// Run ExportFlagRollouts after validating the tenant and durably recording the command.
    async fn run_export_flag_rollouts(
        &self,
        req: featureflags::ExportFlagRolloutsRequest,
    ) -> Result<featureflags::ExportFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let mut response = featureflags::ExportFlagRolloutsResponse::default();
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
    /// Run SummarizeFlagRollouts after validating the tenant and durably recording the command.
    async fn run_summarize_flag_rollouts(
        &self,
        req: featureflags::SummarizeFlagRolloutsRequest,
    ) -> Result<featureflags::SummarizeFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::SummarizeFlagRolloutsResponse::default();
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestFlagRollouts after validating the tenant and durably recording the command.
    async fn run_suggest_flag_rollouts(
        &self,
        req: featureflags::SuggestFlagRolloutsRequest,
    ) -> Result<featureflags::SuggestFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::SuggestFlagRolloutsResponse::default();
        Ok(response)
    }
    /// Run PeekFlagRollout after validating the tenant and durably recording the command.
    async fn run_peek_flag_rollout(
        &self,
        req: featureflags::PeekFlagRolloutRequest,
    ) -> Result<featureflags::PeekFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekFlagRollout", "running workflow");
        let mut response = featureflags::PeekFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FlagRolloutQueryService for FlagRolloutQueryController {
    /// Returns a flag rollout.
    async fn lookup_flag_rollout(
        &self,
        request: tonic::Request<featureflags::LookupFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::LookupFlagRolloutResponse>, tonic::Status> {
        self.run_lookup_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag rollouts a page at a time.
    async fn browse_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::BrowseFlagRolloutsRequest>,
    ) -> Result<tonic::Response<featureflags::BrowseFlagRolloutsResponse>, tonic::Status> {
        self.run_browse_flag_rollouts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag rollouts a page at a time.
    async fn find_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::FindFlagRolloutsRequest>,
    ) -> Result<tonic::Response<featureflags::FindFlagRolloutsResponse>, tonic::Status> {
        self.run_find_flag_rollouts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportFlagRolloutsStream =
        tokio_stream::wrappers::ReceiverStream<Result<featureflags::ExportFlagRolloutsResponse, tonic::Status>>;
    /// Streams every matching flag rollout.
    async fn export_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::ExportFlagRolloutsRequest>,
    ) -> Result<tonic::Response<Self::ExportFlagRolloutsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_export_flag_rollouts(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a flag rollout.
    async fn summarize_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::SummarizeFlagRolloutsRequest>,
    ) -> Result<tonic::Response<featureflags::SummarizeFlagRolloutsResponse>, tonic::Status> {
        self.run_summarize_flag_rollouts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag rollouts a page at a time.
    async fn suggest_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::SuggestFlagRolloutsRequest>,
    ) -> Result<tonic::Response<featureflags::SuggestFlagRolloutsResponse>, tonic::Status> {
        self.run_suggest_flag_rollouts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag rollout.
    async fn peek_flag_rollout(
        &self,
        request: tonic::Request<featureflags::PeekFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::PeekFlagRolloutResponse>, tonic::Status> {
        self.run_peek_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
