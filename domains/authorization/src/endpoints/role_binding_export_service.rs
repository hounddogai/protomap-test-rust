use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_export_service_server::RoleBindingExportService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_query_service_client::FeatureFlagQueryServiceClient;

/// Exports role bindings to files and other systems.
#[derive(Clone)]
pub struct RoleBindingExportEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl RoleBindingExportEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StreamRoleBindingExport after validating the tenant and durably recording the command.
    async fn run_stream_role_binding_export(
        &self,
        req: authorization::StreamRoleBindingExportRequest,
    ) -> Result<authorization::StreamRoleBindingExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamRoleBindingExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamRoleBindingExport", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let mut response = authorization::StreamRoleBindingExportResponse::default();
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run GetRoleBindingExport after validating the tenant and durably recording the command.
    async fn run_get_role_binding_export(
        &self,
        req: authorization::GetRoleBindingExportRequest,
    ) -> Result<authorization::GetRoleBindingExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRoleBindingExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRoleBindingExport", "running workflow");
        let mut response = authorization::GetRoleBindingExportResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run CancelRoleBindingExport after validating the tenant and durably recording the command.
    async fn run_cancel_role_binding_export(
        &self,
        req: authorization::CancelRoleBindingExportRequest,
    ) -> Result<authorization::CancelRoleBindingExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelRoleBindingExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelRoleBindingExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::CancelRoleBindingExportResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ListRoleBindingExports after validating the tenant and durably recording the command.
    async fn run_list_role_binding_exports(
        &self,
        req: authorization::ListRoleBindingExportsRequest,
    ) -> Result<authorization::ListRoleBindingExportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListRoleBindingExports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRoleBindingExports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::ListRoleBindingExportsResponse::default();
        Ok(response)
    }
    /// Run RetryRoleBindingExport after validating the tenant and durably recording the command.
    async fn run_retry_role_binding_export(
        &self,
        req: authorization::RetryRoleBindingExportRequest,
    ) -> Result<authorization::RetryRoleBindingExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryRoleBindingExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryRoleBindingExport", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CountFeatureFlagsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::count_feature_flags_request::Filter {
                created_after: req.role_binding.clone().unwrap_or_default().due_at.clone(),
                region: req
                    .role_binding
                    .clone()
                    .unwrap_or_default()
                    .billing_address
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::CountFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::RetryRoleBindingExportResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.role_binding.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.role_binding.get_or_insert_with(Default::default).due_at = contribution;
        }
        Ok(response)
    }
    /// Run ShareRoleBindingExport after validating the tenant and durably recording the command.
    async fn run_share_role_binding_export(
        &self,
        req: authorization::ShareRoleBindingExportRequest,
    ) -> Result<authorization::ShareRoleBindingExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareRoleBindingExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareRoleBindingExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::ShareRoleBindingExportResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { RoleBindingExportService, RoleBindingExportEndpoint;
unary {
get_role_binding_export (authorization::GetRoleBindingExportRequest) -> authorization::GetRoleBindingExportResponse => run_get_role_binding_export;
cancel_role_binding_export (authorization::CancelRoleBindingExportRequest) -> authorization::CancelRoleBindingExportResponse => run_cancel_role_binding_export;
list_role_binding_exports (authorization::ListRoleBindingExportsRequest) -> authorization::ListRoleBindingExportsResponse => run_list_role_binding_exports;
retry_role_binding_export (authorization::RetryRoleBindingExportRequest) -> authorization::RetryRoleBindingExportResponse => run_retry_role_binding_export;
share_role_binding_export (authorization::ShareRoleBindingExportRequest) -> authorization::ShareRoleBindingExportResponse => run_share_role_binding_export;
}
server {
stream_role_binding_export (authorization::StreamRoleBindingExportRequest) -> authorization::StreamRoleBindingExportResponse, StreamRoleBindingExportStream => run_stream_role_binding_export;
}
client {
}
bidi {
}
}
