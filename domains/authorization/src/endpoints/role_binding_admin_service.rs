use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_export_service_client::PermissionCheckExportServiceClient;
use acme_proto::acme::authorization::v1::role_binding_admin_service_server::RoleBindingAdminService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient;

/// Operator actions on role bindings that support staff run by hand.
#[derive(Clone)]
pub struct RoleBindingAdminApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    archive_role_binding_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::RetryFlagEvaluationExportRequest,
            featureflags::RetryFlagEvaluationExportResponse,
        >,
    >,
}
impl RoleBindingAdminApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::RetryFlagEvaluationExportRequest,
            featureflags::RetryFlagEvaluationExportResponse,
        > = {
            let client = FlagEvaluationExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::RetryFlagEvaluationExportRequest| -> acme_platform::ports::RpcFuture<featureflags::RetryFlagEvaluationExportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.retry_flag_evaluation_export(request).await?.into_inner())
})
})
        };
        let archive_role_binding_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            archive_role_binding_0_port,
        }
    }
    /// Run ArchiveRoleBinding after validating the tenant and durably recording the command.
    async fn run_archive_role_binding(
        &self,
        req: authorization::ArchiveRoleBindingRequest,
    ) -> Result<authorization::ArchiveRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ArchiveRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ArchiveRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = featureflags::RetryFlagEvaluationExportRequest {
            context: req.context.clone(),
            details: Some(featureflags::retry_flag_evaluation_export_request::Details {
                notify: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.archive_role_binding_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = authorization::ArchiveRoleBindingResponse::default();
        let contribution: String = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
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
    /// Run RestoreRoleBinding after validating the tenant and durably recording the command.
    async fn run_restore_role_binding(
        &self,
        req: authorization::RestoreRoleBindingRequest,
    ) -> Result<authorization::RestoreRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RestoreRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RestoreRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::RestoreRoleBindingResponse::default();
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
    /// Run PurgeRoleBinding after validating the tenant and durably recording the command.
    async fn run_purge_role_binding(
        &self,
        req: authorization::PurgeRoleBindingRequest,
    ) -> Result<authorization::PurgeRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PurgeRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PurgeRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding_id.clone(), "recorded audit field");
        let response = authorization::PurgeRoleBindingResponse::default();
        Ok(response)
    }
    /// Run ReassignRoleBinding after validating the tenant and durably recording the command.
    async fn run_reassign_role_binding(
        &self,
        req: authorization::ReassignRoleBindingRequest,
    ) -> Result<authorization::ReassignRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReassignRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReassignRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().assignee.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let outgoing_0 = authorization::StreamPermissionCheckExportRequest {
            context: req.context.clone(),
            filter: Some(authorization::stream_permission_check_export_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.annotations.clone(),
                owner_id: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .assignee
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = PermissionCheckExportServiceClient::new(channel);
            let input: tonic::Request<authorization::StreamPermissionCheckExportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_permission_check_export(input).await?;
            let mut frames = response.into_inner();
            let mut last = authorization::StreamPermissionCheckExportResponse::default();
            // The caller asks for a bounded snapshot, then drops its subscription.
            for _ in 0..8 {
                match frames.message().await? {
                    Some(frame) => last = frame,
                    None => break,
                }
            }
            Ok::<_, tonic::Status>(last)
        }
        .await?;
        let mut response = authorization::ReassignRoleBindingResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run LockRoleBinding after validating the tenant and durably recording the command.
    async fn run_lock_role_binding(
        &self,
        req: authorization::LockRoleBindingRequest,
    ) -> Result<authorization::LockRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LockRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LockRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::LockRoleBindingResponse::default();
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
    /// Run UnlockRoleBinding after validating the tenant and durably recording the command.
    async fn run_unlock_role_binding(
        &self,
        req: authorization::UnlockRoleBindingRequest,
    ) -> Result<authorization::UnlockRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UnlockRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnlockRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::UnlockRoleBindingResponse::default();
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
    /// Run TransferRoleBinding after validating the tenant and durably recording the command.
    async fn run_transfer_role_binding(
        &self,
        req: authorization::TransferRoleBindingRequest,
    ) -> Result<authorization::TransferRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TransferRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TransferRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::TransferRoleBindingResponse::default();
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
    /// Run RepairRoleBinding after validating the tenant and durably recording the command.
    async fn run_repair_role_binding(
        &self,
        req: authorization::RepairRoleBindingRequest,
    ) -> Result<authorization::RepairRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RepairRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RepairRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::RepairRoleBindingResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: i32 = req.role_binding.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.role_binding.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run InspectRoleBinding after validating the tenant and durably recording the command.
    async fn run_inspect_role_binding(
        &self,
        req: authorization::InspectRoleBindingRequest,
    ) -> Result<authorization::InspectRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectRoleBinding", "running workflow");
        let mut response = authorization::InspectRoleBindingResponse::default();
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
#[tonic::async_trait]
impl RoleBindingAdminService for RoleBindingAdminApi {
    /// Archives a role binding.
    async fn archive_role_binding(
        &self,
        request: tonic::Request<authorization::ArchiveRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::ArchiveRoleBindingResponse>, tonic::Status> {
        self.run_archive_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Restores a role binding.
    async fn restore_role_binding(
        &self,
        request: tonic::Request<authorization::RestoreRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::RestoreRoleBindingResponse>, tonic::Status> {
        self.run_restore_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a role binding.
    async fn purge_role_binding(
        &self,
        request: tonic::Request<authorization::PurgeRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::PurgeRoleBindingResponse>, tonic::Status> {
        self.run_purge_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reassigns a role binding.
    async fn reassign_role_binding(
        &self,
        request: tonic::Request<authorization::ReassignRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::ReassignRoleBindingResponse>, tonic::Status> {
        self.run_reassign_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Locks a role binding.
    async fn lock_role_binding(
        &self,
        request: tonic::Request<authorization::LockRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::LockRoleBindingResponse>, tonic::Status> {
        self.run_lock_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unlocks a role binding.
    async fn unlock_role_binding(
        &self,
        request: tonic::Request<authorization::UnlockRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::UnlockRoleBindingResponse>, tonic::Status> {
        self.run_unlock_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Transfers a role binding.
    async fn transfer_role_binding(
        &self,
        request: tonic::Request<authorization::TransferRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::TransferRoleBindingResponse>, tonic::Status> {
        self.run_transfer_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Repairs a role binding.
    async fn repair_role_binding(
        &self,
        request: tonic::Request<authorization::RepairRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::RepairRoleBindingResponse>, tonic::Status> {
        self.run_repair_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a role binding.
    async fn inspect_role_binding(
        &self,
        request: tonic::Request<authorization::InspectRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::InspectRoleBindingResponse>, tonic::Status> {
        self.run_inspect_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
