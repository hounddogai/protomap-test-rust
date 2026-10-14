use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_admin_service_server::FlagRolloutAdminService;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Operator actions on flag rollouts that support staff run by hand.
#[derive(Clone)]
pub struct FlagRolloutAdminHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagRolloutAdminHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ArchiveFlagRollout after validating the tenant and durably recording the command.
    async fn run_archive_flag_rollout(
        &self,
        req: featureflags::ArchiveFlagRolloutRequest,
    ) -> Result<featureflags::ArchiveFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ArchiveFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ArchiveFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::ArchiveFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RestoreFlagRollout after validating the tenant and durably recording the command.
    async fn run_restore_flag_rollout(
        &self,
        req: featureflags::RestoreFlagRolloutRequest,
    ) -> Result<featureflags::RestoreFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RestoreFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RestoreFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::RestoreFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run PurgeFlagRollout after validating the tenant and durably recording the command.
    async fn run_purge_flag_rollout(
        &self,
        req: featureflags::PurgeFlagRolloutRequest,
    ) -> Result<featureflags::PurgeFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PurgeFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PurgeFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout_id.clone(), "recorded audit field");
        let response = featureflags::PurgeFlagRolloutResponse::default();
        Ok(response)
    }
    /// Run LockFlagRollout after validating the tenant and durably recording the command.
    async fn run_lock_flag_rollout(
        &self,
        req: featureflags::LockFlagRolloutRequest,
    ) -> Result<featureflags::LockFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LockFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LockFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = venues::SummarizeVenuesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = VenueQueryServiceClient::new(channel);
            let input: tonic::Request<venues::SummarizeVenuesRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = featureflags::LockFlagRolloutResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TransferFlagRollout after validating the tenant and durably recording the command.
    async fn run_transfer_flag_rollout(
        &self,
        req: featureflags::TransferFlagRolloutRequest,
    ) -> Result<featureflags::TransferFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TransferFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TransferFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::TransferFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RepairFlagRollout after validating the tenant and durably recording the command.
    async fn run_repair_flag_rollout(
        &self,
        req: featureflags::RepairFlagRolloutRequest,
    ) -> Result<featureflags::RepairFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RepairFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RepairFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::RepairFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.flag_rollout.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run InspectFlagRollout after validating the tenant and durably recording the command.
    async fn run_inspect_flag_rollout(
        &self,
        req: featureflags::InspectFlagRolloutRequest,
    ) -> Result<featureflags::InspectFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectFlagRollout", "running workflow");
        let outgoing_0 = featureflags::PullFeatureFlagsRequest {
            context: req.context.clone(),
            details: Some(featureflags::pull_feature_flags_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                owner: Some(identity::UserRef {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
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
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::PullFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pull_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::InspectFlagRolloutResponse::default();
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
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
impl FlagRolloutAdminService for FlagRolloutAdminHandler {
    /// Archives a flag rollout.
    async fn archive_flag_rollout(
        &self,
        request: tonic::Request<featureflags::ArchiveFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::ArchiveFlagRolloutResponse>, tonic::Status> {
        self.run_archive_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Restores a flag rollout.
    async fn restore_flag_rollout(
        &self,
        request: tonic::Request<featureflags::RestoreFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::RestoreFlagRolloutResponse>, tonic::Status> {
        self.run_restore_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a flag rollout.
    async fn purge_flag_rollout(
        &self,
        request: tonic::Request<featureflags::PurgeFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::PurgeFlagRolloutResponse>, tonic::Status> {
        self.run_purge_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Locks a flag rollout.
    async fn lock_flag_rollout(
        &self,
        request: tonic::Request<featureflags::LockFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::LockFlagRolloutResponse>, tonic::Status> {
        self.run_lock_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Transfers a flag rollout.
    async fn transfer_flag_rollout(
        &self,
        request: tonic::Request<featureflags::TransferFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::TransferFlagRolloutResponse>, tonic::Status> {
        self.run_transfer_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Repairs a flag rollout.
    async fn repair_flag_rollout(
        &self,
        request: tonic::Request<featureflags::RepairFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::RepairFlagRolloutResponse>, tonic::Status> {
        self.run_repair_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag rollout.
    async fn inspect_flag_rollout(
        &self,
        request: tonic::Request<featureflags::InspectFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::InspectFlagRolloutResponse>, tonic::Status> {
        self.run_inspect_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
