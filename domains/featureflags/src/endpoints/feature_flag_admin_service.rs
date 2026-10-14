use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_admin_service_server::FeatureFlagAdminService;
use acme_proto::acme::featureflags::v1::flag_rollout_export_service_client::FlagRolloutExportServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Operator actions on feature flags that support staff run by hand.
#[derive(Clone)]
pub struct FeatureFlagAdminController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FeatureFlagAdminController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ArchiveFeatureFlag after validating the tenant and durably recording the command.
    async fn run_archive_feature_flag(
        &self,
        req: featureflags::ArchiveFeatureFlagRequest,
    ) -> Result<featureflags::ArchiveFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ArchiveFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ArchiveFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::ArchiveFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RestoreFeatureFlag after validating the tenant and durably recording the command.
    async fn run_restore_feature_flag(
        &self,
        req: featureflags::RestoreFeatureFlagRequest,
    ) -> Result<featureflags::RestoreFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RestoreFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RestoreFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ScheduleFlagRolloutExportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(featureflags::schedule_flag_rollout_export_request::Details {
                filters: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = FlagRolloutExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::ScheduleFlagRolloutExportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_flag_rollout_export(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = featureflags::RestoreFeatureFlagResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run PurgeFeatureFlag after validating the tenant and durably recording the command.
    async fn run_purge_feature_flag(
        &self,
        req: featureflags::PurgeFeatureFlagRequest,
    ) -> Result<featureflags::PurgeFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PurgeFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PurgeFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag_id.clone(), "recorded audit field");
        let response = featureflags::PurgeFeatureFlagResponse::default();
        Ok(response)
    }
    /// Run ReassignFeatureFlag after validating the tenant and durably recording the command.
    async fn run_reassign_feature_flag(
        &self,
        req: featureflags::ReassignFeatureFlagRequest,
    ) -> Result<featureflags::ReassignFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReassignFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReassignFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().assignee.clone().unwrap_or_default().user_id.clone(), "recorded audit field");
        let outgoing_0 = venues::ValidateVenueRequest {
            context: req.context.clone(),
            details: Some(venues::validate_venue_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .assignee
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<venues::ValidateVenueRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = VenueValidationServiceClient::new(channel);
                let input: tonic::Request<venues::ValidateVenueRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.validate_venue(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_0)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_0 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = featureflags::ReassignFeatureFlagResponse::default();
        let contribution: String = reply_0
            .venue
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run LockFeatureFlag after validating the tenant and durably recording the command.
    async fn run_lock_feature_flag(
        &self,
        req: featureflags::LockFeatureFlagRequest,
    ) -> Result<featureflags::LockFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LockFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LockFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::LockFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run UnlockFeatureFlag after validating the tenant and durably recording the command.
    async fn run_unlock_feature_flag(
        &self,
        req: featureflags::UnlockFeatureFlagRequest,
    ) -> Result<featureflags::UnlockFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UnlockFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnlockFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::UnlockFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TransferFeatureFlag after validating the tenant and durably recording the command.
    async fn run_transfer_feature_flag(
        &self,
        req: featureflags::TransferFeatureFlagRequest,
    ) -> Result<featureflags::TransferFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TransferFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TransferFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::TransferFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RepairFeatureFlag after validating the tenant and durably recording the command.
    async fn run_repair_feature_flag(
        &self,
        req: featureflags::RepairFeatureFlagRequest,
    ) -> Result<featureflags::RepairFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RepairFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RepairFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::RepairFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req.feature_flag.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run InspectFeatureFlag after validating the tenant and durably recording the command.
    async fn run_inspect_feature_flag(
        &self,
        req: featureflags::InspectFeatureFlagRequest,
    ) -> Result<featureflags::InspectFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectFeatureFlag", "running workflow");
        let mut response = featureflags::InspectFeatureFlagResponse::default();
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
impl FeatureFlagAdminService for FeatureFlagAdminController {
    /// Archives a feature flag.
    async fn archive_feature_flag(
        &self,
        request: tonic::Request<featureflags::ArchiveFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::ArchiveFeatureFlagResponse>, tonic::Status> {
        self.run_archive_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Restores a feature flag.
    async fn restore_feature_flag(
        &self,
        request: tonic::Request<featureflags::RestoreFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::RestoreFeatureFlagResponse>, tonic::Status> {
        self.run_restore_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a feature flag.
    async fn purge_feature_flag(
        &self,
        request: tonic::Request<featureflags::PurgeFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::PurgeFeatureFlagResponse>, tonic::Status> {
        self.run_purge_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reassigns a feature flag.
    async fn reassign_feature_flag(
        &self,
        request: tonic::Request<featureflags::ReassignFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::ReassignFeatureFlagResponse>, tonic::Status> {
        self.run_reassign_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Locks a feature flag.
    async fn lock_feature_flag(
        &self,
        request: tonic::Request<featureflags::LockFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::LockFeatureFlagResponse>, tonic::Status> {
        self.run_lock_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unlocks a feature flag.
    async fn unlock_feature_flag(
        &self,
        request: tonic::Request<featureflags::UnlockFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::UnlockFeatureFlagResponse>, tonic::Status> {
        self.run_unlock_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Transfers a feature flag.
    async fn transfer_feature_flag(
        &self,
        request: tonic::Request<featureflags::TransferFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::TransferFeatureFlagResponse>, tonic::Status> {
        self.run_transfer_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Repairs a feature flag.
    async fn repair_feature_flag(
        &self,
        request: tonic::Request<featureflags::RepairFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::RepairFeatureFlagResponse>, tonic::Status> {
        self.run_repair_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a feature flag.
    async fn inspect_feature_flag(
        &self,
        request: tonic::Request<featureflags::InspectFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::InspectFeatureFlagResponse>, tonic::Status> {
        self.run_inspect_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
