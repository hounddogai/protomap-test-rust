use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_history_service_server::PermissionCheckHistoryService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_export_service_client::FeatureFlagExportServiceClient;

/// Keeps every version of a permission check and restores earlier ones.
#[derive(Clone)]
pub struct PermissionCheckHistoryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl PermissionCheckHistoryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run RecordPermissionCheckVersion after validating the tenant and durably recording the command.
    async fn run_record_permission_check_version(
        &self,
        req: authorization::RecordPermissionCheckVersionRequest,
    ) -> Result<authorization::RecordPermissionCheckVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecordPermissionCheckVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecordPermissionCheckVersion", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::RecordPermissionCheckVersionResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: String = req.permission_check.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.permission_check.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run ListPermissionCheckVersions after validating the tenant and durably recording the command.
    async fn run_list_permission_check_versions(
        &self,
        req: authorization::ListPermissionCheckVersionsRequest,
    ) -> Result<authorization::ListPermissionCheckVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPermissionCheckVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPermissionCheckVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::ListPermissionCheckVersionsResponse::default();
        Ok(response)
    }
    /// Run GetPermissionCheckVersion after validating the tenant and durably recording the command.
    async fn run_get_permission_check_version(
        &self,
        req: authorization::GetPermissionCheckVersionRequest,
    ) -> Result<authorization::GetPermissionCheckVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetPermissionCheckVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetPermissionCheckVersion", "running workflow");
        let mut response = authorization::GetPermissionCheckVersionResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
    /// Run ComparePermissionCheckVersions after validating the tenant and durably recording the command.
    async fn run_compare_permission_check_versions(
        &self,
        req: authorization::ComparePermissionCheckVersionsRequest,
    ) -> Result<authorization::ComparePermissionCheckVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ComparePermissionCheckVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ComparePermissionCheckVersions", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::ComparePermissionCheckVersionsResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: i64 = req.details.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        Ok(response)
    }
    /// Run RevertPermissionCheck after validating the tenant and durably recording the command.
    async fn run_revert_permission_check(
        &self,
        req: authorization::RevertPermissionCheckRequest,
    ) -> Result<authorization::RevertPermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RevertPermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RevertPermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::RevertPermissionCheckResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: String = req.permission_check.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run AnnotatePermissionCheckVersion after validating the tenant and durably recording the command.
    async fn run_annotate_permission_check_version(
        &self,
        req: authorization::AnnotatePermissionCheckVersionRequest,
    ) -> Result<authorization::AnnotatePermissionCheckVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AnnotatePermissionCheckVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AnnotatePermissionCheckVersion", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFeatureFlagExportsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_exports_request::Filter {
                owner_id: req
                    .permission_check
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .access_policy_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::ListFeatureFlagExportsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_feature_flag_exports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::AnnotatePermissionCheckVersionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.permission_check.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .occurred_at = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl PermissionCheckHistoryService for PermissionCheckHistoryEndpoint {
    /// Records a permission check.
    async fn record_permission_check_version(
        &self,
        request: tonic::Request<authorization::RecordPermissionCheckVersionRequest>,
    ) -> Result<tonic::Response<authorization::RecordPermissionCheckVersionResponse>, tonic::Status> {
        self.run_record_permission_check_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists permission checks a page at a time.
    async fn list_permission_check_versions(
        &self,
        request: tonic::Request<authorization::ListPermissionCheckVersionsRequest>,
    ) -> Result<tonic::Response<authorization::ListPermissionCheckVersionsResponse>, tonic::Status> {
        self.run_list_permission_check_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a permission check.
    async fn get_permission_check_version(
        &self,
        request: tonic::Request<authorization::GetPermissionCheckVersionRequest>,
    ) -> Result<tonic::Response<authorization::GetPermissionCheckVersionResponse>, tonic::Status> {
        self.run_get_permission_check_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a permission check.
    async fn compare_permission_check_versions(
        &self,
        request: tonic::Request<authorization::ComparePermissionCheckVersionsRequest>,
    ) -> Result<tonic::Response<authorization::ComparePermissionCheckVersionsResponse>, tonic::Status> {
        self.run_compare_permission_check_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reverts a permission check.
    async fn revert_permission_check(
        &self,
        request: tonic::Request<authorization::RevertPermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::RevertPermissionCheckResponse>, tonic::Status> {
        self.run_revert_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Annotates a permission check.
    async fn annotate_permission_check_version(
        &self,
        request: tonic::Request<authorization::AnnotatePermissionCheckVersionRequest>,
    ) -> Result<tonic::Response<authorization::AnnotatePermissionCheckVersionResponse>, tonic::Status> {
        self.run_annotate_permission_check_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
