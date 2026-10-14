use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_admin_service_server::AccessPolicyAdminService;

/// Operator actions on access policies that support staff run by hand.
#[derive(Clone)]
pub struct AccessPolicyAdminEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl AccessPolicyAdminEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ArchiveAccessPolicy after validating the tenant and durably recording the command.
    async fn run_archive_access_policy(
        &self,
        req: authorization::ArchiveAccessPolicyRequest,
    ) -> Result<authorization::ArchiveAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ArchiveAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ArchiveAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::ArchiveAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RestoreAccessPolicy after validating the tenant and durably recording the command.
    async fn run_restore_access_policy(
        &self,
        req: authorization::RestoreAccessPolicyRequest,
    ) -> Result<authorization::RestoreAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RestoreAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RestoreAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::RestoreAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run PurgeAccessPolicy after validating the tenant and durably recording the command.
    async fn run_purge_access_policy(
        &self,
        req: authorization::PurgeAccessPolicyRequest,
    ) -> Result<authorization::PurgeAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PurgeAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PurgeAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy_id.clone(), "recorded audit field");
        let response = authorization::PurgeAccessPolicyResponse::default();
        Ok(response)
    }
    /// Run ReassignAccessPolicy after validating the tenant and durably recording the command.
    async fn run_reassign_access_policy(
        &self,
        req: authorization::ReassignAccessPolicyRequest,
    ) -> Result<authorization::ReassignAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReassignAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReassignAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().assignee.clone().unwrap_or_default().display_name.clone(), "recorded audit field");
        let mut response = authorization::ReassignAccessPolicyResponse::default();
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
            .assignee
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        Ok(response)
    }
    /// Run LockAccessPolicy after validating the tenant and durably recording the command.
    async fn run_lock_access_policy(
        &self,
        req: authorization::LockAccessPolicyRequest,
    ) -> Result<authorization::LockAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LockAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LockAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::LockAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run UnlockAccessPolicy after validating the tenant and durably recording the command.
    async fn run_unlock_access_policy(
        &self,
        req: authorization::UnlockAccessPolicyRequest,
    ) -> Result<authorization::UnlockAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UnlockAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnlockAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::UnlockAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TransferAccessPolicy after validating the tenant and durably recording the command.
    async fn run_transfer_access_policy(
        &self,
        req: authorization::TransferAccessPolicyRequest,
    ) -> Result<authorization::TransferAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TransferAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TransferAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::TransferAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RepairAccessPolicy after validating the tenant and durably recording the command.
    async fn run_repair_access_policy(
        &self,
        req: authorization::RepairAccessPolicyRequest,
    ) -> Result<authorization::RepairAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RepairAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RepairAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::RepairAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: i64 = req.access_policy.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.access_policy.get_or_insert_with(Default::default).version = contribution;
        }
        Ok(response)
    }
    /// Run InspectAccessPolicy after validating the tenant and durably recording the command.
    async fn run_inspect_access_policy(
        &self,
        req: authorization::InspectAccessPolicyRequest,
    ) -> Result<authorization::InspectAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectAccessPolicy", "running workflow");
        let mut response = authorization::InspectAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl AccessPolicyAdminService for AccessPolicyAdminEndpoint {
    /// Archives an access policy.
    async fn archive_access_policy(
        &self,
        request: tonic::Request<authorization::ArchiveAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::ArchiveAccessPolicyResponse>, tonic::Status> {
        self.run_archive_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Restores an access policy.
    async fn restore_access_policy(
        &self,
        request: tonic::Request<authorization::RestoreAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::RestoreAccessPolicyResponse>, tonic::Status> {
        self.run_restore_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes an access policy.
    async fn purge_access_policy(
        &self,
        request: tonic::Request<authorization::PurgeAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::PurgeAccessPolicyResponse>, tonic::Status> {
        self.run_purge_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reassigns an access policy.
    async fn reassign_access_policy(
        &self,
        request: tonic::Request<authorization::ReassignAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::ReassignAccessPolicyResponse>, tonic::Status> {
        self.run_reassign_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Locks an access policy.
    async fn lock_access_policy(
        &self,
        request: tonic::Request<authorization::LockAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::LockAccessPolicyResponse>, tonic::Status> {
        self.run_lock_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unlocks an access policy.
    async fn unlock_access_policy(
        &self,
        request: tonic::Request<authorization::UnlockAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::UnlockAccessPolicyResponse>, tonic::Status> {
        self.run_unlock_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Transfers an access policy.
    async fn transfer_access_policy(
        &self,
        request: tonic::Request<authorization::TransferAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::TransferAccessPolicyResponse>, tonic::Status> {
        self.run_transfer_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Repairs an access policy.
    async fn repair_access_policy(
        &self,
        request: tonic::Request<authorization::RepairAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::RepairAccessPolicyResponse>, tonic::Status> {
        self.run_repair_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an access policy.
    async fn inspect_access_policy(
        &self,
        request: tonic::Request<authorization::InspectAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::InspectAccessPolicyResponse>, tonic::Status> {
        self.run_inspect_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
