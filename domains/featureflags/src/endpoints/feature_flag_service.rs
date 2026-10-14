use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_service_server::FeatureFlagService;

/// Manages feature flags: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct FeatureFlagApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FeatureFlagApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run CreateFeatureFlag after validating the tenant and durably recording the command.
    async fn run_create_feature_flag(
        &self,
        req: featureflags::CreateFeatureFlagRequest,
    ) -> Result<featureflags::CreateFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::CreateFeatureFlagResponse::default();
        let contribution: i32 = req.feature_flag.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.feature_flag.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.feature_flag.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.feature_flag.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run GetFeatureFlag after validating the tenant and durably recording the command.
    async fn run_get_feature_flag(
        &self,
        req: featureflags::GetFeatureFlagRequest,
    ) -> Result<featureflags::GetFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFeatureFlag", "running workflow");
        let mut response = featureflags::GetFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        Ok(response)
    }
    /// Run ListFeatureFlags after validating the tenant and durably recording the command.
    async fn run_list_feature_flags(
        &self,
        req: featureflags::ListFeatureFlagsRequest,
    ) -> Result<featureflags::ListFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFeatureFlagsResponse::default();
        Ok(response)
    }
    /// Run BatchGetFeatureFlags after validating the tenant and durably recording the command.
    async fn run_batch_get_feature_flags(
        &self,
        req: featureflags::BatchGetFeatureFlagsRequest,
    ) -> Result<featureflags::BatchGetFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchGetFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchGetFeatureFlags", "running workflow");
        let response = featureflags::BatchGetFeatureFlagsResponse::default();
        Ok(response)
    }
    /// Run BatchCreateFeatureFlags after validating the tenant and durably recording the command.
    async fn run_batch_create_feature_flags(
        &self,
        req: featureflags::BatchCreateFeatureFlagsRequest,
    ) -> Result<featureflags::BatchCreateFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("BatchCreateFeatureFlags", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(featureflags::batch_create_feature_flags_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let response = featureflags::BatchCreateFeatureFlagsResponse::default();
        Ok(response)
    }
    /// Run CloneFeatureFlag after validating the tenant and durably recording the command.
    async fn run_clone_feature_flag(
        &self,
        req: featureflags::CloneFeatureFlagRequest,
    ) -> Result<featureflags::CloneFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CloneFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CloneFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::CloneFeatureFlagResponse::default();
        let contribution: String = req.feature_flag.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: String = req.feature_flag.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FeatureFlagService for FeatureFlagApi {
    /// Creates a feature flag.
    async fn create_feature_flag(
        &self,
        request: tonic::Request<featureflags::CreateFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::CreateFeatureFlagResponse>, tonic::Status> {
        self.run_create_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a feature flag.
    async fn get_feature_flag(
        &self,
        request: tonic::Request<featureflags::GetFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::GetFeatureFlagResponse>, tonic::Status> {
        self.run_get_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists feature flags a page at a time.
    async fn list_feature_flags(
        &self,
        request: tonic::Request<featureflags::ListFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFeatureFlagsResponse>, tonic::Status> {
        self.run_list_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several feature flags at once.
    async fn batch_get_feature_flags(
        &self,
        request: tonic::Request<featureflags::BatchGetFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::BatchGetFeatureFlagsResponse>, tonic::Status> {
        self.run_batch_get_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Takes a stream of feature flags and answers once at the end.
    async fn batch_create_feature_flags(
        &self,
        request: tonic::Request<tonic::Streaming<featureflags::BatchCreateFeatureFlagsRequest>>,
    ) -> Result<tonic::Response<featureflags::BatchCreateFeatureFlagsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = featureflags::BatchCreateFeatureFlagsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_batch_create_feature_flags(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Creates a feature flag.
    async fn clone_feature_flag(
        &self,
        request: tonic::Request<featureflags::CloneFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::CloneFeatureFlagResponse>, tonic::Status> {
        self.run_clone_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
