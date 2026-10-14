use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_query_service_server::FeatureFlagQueryService;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;

/// Answers read-only questions about feature flags, for screens and reports.
#[derive(Clone)]
pub struct FeatureFlagQueryController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FeatureFlagQueryController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run LookupFeatureFlag after validating the tenant and durably recording the command.
    async fn run_lookup_feature_flag(
        &self,
        req: featureflags::LookupFeatureFlagRequest,
    ) -> Result<featureflags::LookupFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupFeatureFlag", "running workflow");
        let mut response = featureflags::LookupFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseFeatureFlags after validating the tenant and durably recording the command.
    async fn run_browse_feature_flags(
        &self,
        req: featureflags::BrowseFeatureFlagsRequest,
    ) -> Result<featureflags::BrowseFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CheckFlagRolloutEligibilityRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                location: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<featureflags::CheckFlagRolloutEligibilityRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::CheckFlagRolloutEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_flag_rollout_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::BrowseFeatureFlagsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run CountFeatureFlags after validating the tenant and durably recording the command.
    async fn run_count_feature_flags(
        &self,
        req: featureflags::CountFeatureFlagsRequest,
    ) -> Result<featureflags::CountFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::CountFeatureFlagsResponse::default();
        Ok(response)
    }
    /// Run ExportFeatureFlags after validating the tenant and durably recording the command.
    async fn run_export_feature_flags(
        &self,
        req: featureflags::ExportFeatureFlagsRequest,
    ) -> Result<featureflags::ExportFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let response = featureflags::ExportFeatureFlagsResponse::default();
        Ok(response)
    }
    /// Run SummarizeFeatureFlags after validating the tenant and durably recording the command.
    async fn run_summarize_feature_flags(
        &self,
        req: featureflags::SummarizeFeatureFlagsRequest,
    ) -> Result<featureflags::SummarizeFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::SummarizeFeatureFlagsResponse::default();
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestFeatureFlags after validating the tenant and durably recording the command.
    async fn run_suggest_feature_flags(
        &self,
        req: featureflags::SuggestFeatureFlagsRequest,
    ) -> Result<featureflags::SuggestFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::SuggestFeatureFlagsResponse::default();
        Ok(response)
    }
    /// Run PeekFeatureFlag after validating the tenant and durably recording the command.
    async fn run_peek_feature_flag(
        &self,
        req: featureflags::PeekFeatureFlagRequest,
    ) -> Result<featureflags::PeekFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekFeatureFlag", "running workflow");
        let mut response = featureflags::PeekFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FeatureFlagQueryService for FeatureFlagQueryController {
    /// Returns a feature flag.
    async fn lookup_feature_flag(
        &self,
        request: tonic::Request<featureflags::LookupFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::LookupFeatureFlagResponse>, tonic::Status> {
        self.run_lookup_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists feature flags a page at a time.
    async fn browse_feature_flags(
        &self,
        request: tonic::Request<featureflags::BrowseFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::BrowseFeatureFlagsResponse>, tonic::Status> {
        self.run_browse_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts feature flags by status.
    async fn count_feature_flags(
        &self,
        request: tonic::Request<featureflags::CountFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::CountFeatureFlagsResponse>, tonic::Status> {
        self.run_count_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportFeatureFlagsStream =
        tokio_stream::wrappers::ReceiverStream<Result<featureflags::ExportFeatureFlagsResponse, tonic::Status>>;
    /// Streams every matching feature flag.
    async fn export_feature_flags(
        &self,
        request: tonic::Request<featureflags::ExportFeatureFlagsRequest>,
    ) -> Result<tonic::Response<Self::ExportFeatureFlagsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_export_feature_flags(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a feature flag.
    async fn summarize_feature_flags(
        &self,
        request: tonic::Request<featureflags::SummarizeFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::SummarizeFeatureFlagsResponse>, tonic::Status> {
        self.run_summarize_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists feature flags a page at a time.
    async fn suggest_feature_flags(
        &self,
        request: tonic::Request<featureflags::SuggestFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::SuggestFeatureFlagsResponse>, tonic::Status> {
        self.run_suggest_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a feature flag.
    async fn peek_feature_flag(
        &self,
        request: tonic::Request<featureflags::PeekFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::PeekFeatureFlagResponse>, tonic::Status> {
        self.run_peek_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
