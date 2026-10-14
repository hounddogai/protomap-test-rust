use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_export_service_server::PermissionCheckExportService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Exports permission checks to files and other systems.
#[derive(Clone)]
pub struct PermissionCheckExportApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl PermissionCheckExportApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StreamPermissionCheckExport after validating the tenant and durably recording the command.
    async fn run_stream_permission_check_export(
        &self,
        req: authorization::StreamPermissionCheckExportRequest,
    ) -> Result<authorization::StreamPermissionCheckExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamPermissionCheckExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamPermissionCheckExport", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let response = authorization::StreamPermissionCheckExportResponse::default();
        Ok(response)
    }
    /// Run GetPermissionCheckExport after validating the tenant and durably recording the command.
    async fn run_get_permission_check_export(
        &self,
        req: authorization::GetPermissionCheckExportRequest,
    ) -> Result<authorization::GetPermissionCheckExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetPermissionCheckExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetPermissionCheckExport", "running workflow");
        let mut response = authorization::GetPermissionCheckExportResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
    /// Run CancelPermissionCheckExport after validating the tenant and durably recording the command.
    async fn run_cancel_permission_check_export(
        &self,
        req: authorization::CancelPermissionCheckExportRequest,
    ) -> Result<authorization::CancelPermissionCheckExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelPermissionCheckExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelPermissionCheckExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = venues::CloneVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                owner: Some(identity::UserRef {
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let mut response = authorization::CancelPermissionCheckExportResponse::default();
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
                .permission_check
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ListPermissionCheckExports after validating the tenant and durably recording the command.
    async fn run_list_permission_check_exports(
        &self,
        req: authorization::ListPermissionCheckExportsRequest,
    ) -> Result<authorization::ListPermissionCheckExportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPermissionCheckExports", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPermissionCheckExports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::ListPermissionCheckExportsResponse::default();
        Ok(response)
    }
    /// Run SchedulePermissionCheckExport after validating the tenant and durably recording the command.
    async fn run_schedule_permission_check_export(
        &self,
        req: authorization::SchedulePermissionCheckExportRequest,
    ) -> Result<authorization::SchedulePermissionCheckExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SchedulePermissionCheckExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SchedulePermissionCheckExport", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::SchedulePermissionCheckExportResponse::default();
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run RetryPermissionCheckExport after validating the tenant and durably recording the command.
    async fn run_retry_permission_check_export(
        &self,
        req: authorization::RetryPermissionCheckExportRequest,
    ) -> Result<authorization::RetryPermissionCheckExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryPermissionCheckExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryPermissionCheckExport", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let mut response = authorization::RetryPermissionCheckExportResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.permission_check.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run SharePermissionCheckExport after validating the tenant and durably recording the command.
    async fn run_share_permission_check_export(
        &self,
        req: authorization::SharePermissionCheckExportRequest,
    ) -> Result<authorization::SharePermissionCheckExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SharePermissionCheckExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SharePermissionCheckExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = authorization::SharePermissionCheckExportResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { PermissionCheckExportService, PermissionCheckExportApi;
unary {
get_permission_check_export (authorization::GetPermissionCheckExportRequest) -> authorization::GetPermissionCheckExportResponse => run_get_permission_check_export;
cancel_permission_check_export (authorization::CancelPermissionCheckExportRequest) -> authorization::CancelPermissionCheckExportResponse => run_cancel_permission_check_export;
list_permission_check_exports (authorization::ListPermissionCheckExportsRequest) -> authorization::ListPermissionCheckExportsResponse => run_list_permission_check_exports;
schedule_permission_check_export (authorization::SchedulePermissionCheckExportRequest) -> authorization::SchedulePermissionCheckExportResponse => run_schedule_permission_check_export;
retry_permission_check_export (authorization::RetryPermissionCheckExportRequest) -> authorization::RetryPermissionCheckExportResponse => run_retry_permission_check_export;
share_permission_check_export (authorization::SharePermissionCheckExportRequest) -> authorization::SharePermissionCheckExportResponse => run_share_permission_check_export;
}
server {
stream_permission_check_export (authorization::StreamPermissionCheckExportRequest) -> authorization::StreamPermissionCheckExportResponse, StreamPermissionCheckExportStream => run_stream_permission_check_export;
}
client {
}
bidi {
}
}
