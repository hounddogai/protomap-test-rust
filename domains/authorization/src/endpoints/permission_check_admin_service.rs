use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_notification_service_client::AccessPolicyNotificationServiceClient;
use acme_proto::acme::authorization::v1::permission_check_admin_service_server::PermissionCheckAdminService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Operator actions on permission checks that support staff run by hand.
#[derive(Clone)]
pub struct PermissionCheckAdminController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl PermissionCheckAdminController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ArchivePermissionCheck after validating the tenant and durably recording the command.
    async fn run_archive_permission_check(
        &self,
        req: authorization::ArchivePermissionCheckRequest,
    ) -> Result<authorization::ArchivePermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ArchivePermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ArchivePermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = featureflags::RetryFlagEvaluationExportRequest {
            context: req.context.clone(),
            details: Some(featureflags::retry_flag_evaluation_export_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.featureflags.v1.RetryFlagEvaluationExportRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::RetryFlagEvaluationExportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.retry_flag_evaluation_export(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::ArchivePermissionCheckResponse::default();
        let contribution: String = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
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
    /// Run RestorePermissionCheck after validating the tenant and durably recording the command.
    async fn run_restore_permission_check(
        &self,
        req: authorization::RestorePermissionCheckRequest,
    ) -> Result<authorization::RestorePermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RestorePermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RestorePermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::RestorePermissionCheckResponse::default();
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
    /// Run PurgePermissionCheck after validating the tenant and durably recording the command.
    async fn run_purge_permission_check(
        &self,
        req: authorization::PurgePermissionCheckRequest,
    ) -> Result<authorization::PurgePermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PurgePermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PurgePermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check_id.clone(), "recorded audit field");
        let response = authorization::PurgePermissionCheckResponse::default();
        Ok(response)
    }
    /// Run ReassignPermissionCheck after validating the tenant and durably recording the command.
    async fn run_reassign_permission_check(
        &self,
        req: authorization::ReassignPermissionCheckRequest,
    ) -> Result<authorization::ReassignPermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ReassignPermissionCheck", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReassignPermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().assignee.clone().unwrap_or_default().user_id.clone(), "recorded audit field");
        let mut response = authorization::ReassignPermissionCheckResponse::default();
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
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
                .permission_check
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run LockPermissionCheck after validating the tenant and durably recording the command.
    async fn run_lock_permission_check(
        &self,
        req: authorization::LockPermissionCheckRequest,
    ) -> Result<authorization::LockPermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LockPermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LockPermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::LockPermissionCheckResponse::default();
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
    /// Run UnlockPermissionCheck after validating the tenant and durably recording the command.
    async fn run_unlock_permission_check(
        &self,
        req: authorization::UnlockPermissionCheckRequest,
    ) -> Result<authorization::UnlockPermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UnlockPermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnlockPermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = authorization::UnlockPermissionCheckResponse::default();
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
    /// Run TransferPermissionCheck after validating the tenant and durably recording the command.
    async fn run_transfer_permission_check(
        &self,
        req: authorization::TransferPermissionCheckRequest,
    ) -> Result<authorization::TransferPermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("TransferPermissionCheck", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TransferPermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = venues::ScoreVenueRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            venue: Some(venues::Venue {
                owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
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
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::ScoreVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = authorization::TransferPermissionCheckResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run RepairPermissionCheck after validating the tenant and durably recording the command.
    async fn run_repair_permission_check(
        &self,
        req: authorization::RepairPermissionCheckRequest,
    ) -> Result<authorization::RepairPermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RepairPermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RepairPermissionCheck", "running workflow");
        tracing::debug!(audit_value = ?req.permission_check.clone().unwrap_or_default().permission_check_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::SubscribeAccessPolicyUpdatesRequest {
            access_policy_id: req
                .permission_check
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .access_policy_id
                .clone(),
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessPolicyNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::SubscribeAccessPolicyUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_access_policy_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::RepairPermissionCheckResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = req.permission_check_id.clone();
        if contribution != <String>::default() {
            response
                .permission_check
                .get_or_insert_with(Default::default)
                .permission_check_id = contribution;
        }
        let contribution: Option<authorization::AccessPolicyRef> =
            req.permission_check.clone().unwrap_or_default().subject.clone();
        if contribution != <Option<authorization::AccessPolicyRef>>::default() {
            response.permission_check.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run InspectPermissionCheck after validating the tenant and durably recording the command.
    async fn run_inspect_permission_check(
        &self,
        req: authorization::InspectPermissionCheckRequest,
    ) -> Result<authorization::InspectPermissionCheckResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectPermissionCheck", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectPermissionCheck", "running workflow");
        let mut response = authorization::InspectPermissionCheckResponse::default();
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
#[tonic::async_trait]
impl PermissionCheckAdminService for PermissionCheckAdminController {
    /// Archives a permission check.
    async fn archive_permission_check(
        &self,
        request: tonic::Request<authorization::ArchivePermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::ArchivePermissionCheckResponse>, tonic::Status> {
        self.run_archive_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Restores a permission check.
    async fn restore_permission_check(
        &self,
        request: tonic::Request<authorization::RestorePermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::RestorePermissionCheckResponse>, tonic::Status> {
        self.run_restore_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a permission check.
    async fn purge_permission_check(
        &self,
        request: tonic::Request<authorization::PurgePermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::PurgePermissionCheckResponse>, tonic::Status> {
        self.run_purge_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reassigns a permission check.
    async fn reassign_permission_check(
        &self,
        request: tonic::Request<authorization::ReassignPermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::ReassignPermissionCheckResponse>, tonic::Status> {
        self.run_reassign_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Locks a permission check.
    async fn lock_permission_check(
        &self,
        request: tonic::Request<authorization::LockPermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::LockPermissionCheckResponse>, tonic::Status> {
        self.run_lock_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unlocks a permission check.
    async fn unlock_permission_check(
        &self,
        request: tonic::Request<authorization::UnlockPermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::UnlockPermissionCheckResponse>, tonic::Status> {
        self.run_unlock_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Transfers a permission check.
    async fn transfer_permission_check(
        &self,
        request: tonic::Request<authorization::TransferPermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::TransferPermissionCheckResponse>, tonic::Status> {
        self.run_transfer_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Repairs a permission check.
    async fn repair_permission_check(
        &self,
        request: tonic::Request<authorization::RepairPermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::RepairPermissionCheckResponse>, tonic::Status> {
        self.run_repair_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a permission check.
    async fn inspect_permission_check(
        &self,
        request: tonic::Request<authorization::InspectPermissionCheckRequest>,
    ) -> Result<tonic::Response<authorization::InspectPermissionCheckResponse>, tonic::Status> {
        self.run_inspect_permission_check(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
