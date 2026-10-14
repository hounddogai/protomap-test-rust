use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_admin_service_server::FlagEvaluationAdminService;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Operator actions on flag evaluations that support staff run by hand.
#[derive(Clone)]
pub struct FlagEvaluationAdminController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagEvaluationAdminController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ArchiveFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_archive_flag_evaluation(
        &self,
        req: featureflags::ArchiveFlagEvaluationRequest,
    ) -> Result<featureflags::ArchiveFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ArchiveFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ArchiveFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::ArchiveFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RestoreFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_restore_flag_evaluation(
        &self,
        req: featureflags::RestoreFlagEvaluationRequest,
    ) -> Result<featureflags::RestoreFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RestoreFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RestoreFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::RestoreFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run PurgeFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_purge_flag_evaluation(
        &self,
        req: featureflags::PurgeFlagEvaluationRequest,
    ) -> Result<featureflags::PurgeFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PurgeFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PurgeFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation_id.clone(), "recorded audit field");
        let outgoing_0 = venues::CheckVenueEligibilityRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            venue: Some(venues::Venue {
                audit: Some(common::AuditInfo {
                    etag: req.etag.clone(),
                    ..Default::default()
                }),
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
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::CheckVenueEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_venue_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = featureflags::PurgeFlagEvaluationResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.effective_at = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ReassignFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_reassign_flag_evaluation(
        &self,
        req: featureflags::ReassignFlagEvaluationRequest,
    ) -> Result<featureflags::ReassignFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReassignFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReassignFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().assignee.clone().unwrap_or_default().user_id.clone(), "recorded audit field");
        let outgoing_0 = venues::CheckVenueEligibilityRequest {
            context: req.context.clone(),
            details: Some(venues::check_venue_eligibility_request::Details {
                contact: Some(common::ContactInfo {
                    email: req.details.clone().unwrap_or_default().assignee_email.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            venue: Some(venues::Venue {
                owner: Some(identity::UserRef {
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::CheckVenueEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_venue_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::ReassignFlagEvaluationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
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
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run LockFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_lock_flag_evaluation(
        &self,
        req: featureflags::LockFlagEvaluationRequest,
    ) -> Result<featureflags::LockFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LockFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LockFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::LockFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run UnlockFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_unlock_flag_evaluation(
        &self,
        req: featureflags::UnlockFlagEvaluationRequest,
    ) -> Result<featureflags::UnlockFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UnlockFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnlockFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = venues::FindVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::find_venues_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                owner_id: req.flag_evaluation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueQueryServiceClient::new(channel);
            let input: tonic::Request<venues::FindVenuesRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::UnlockFlagEvaluationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TransferFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_transfer_flag_evaluation(
        &self,
        req: featureflags::TransferFlagEvaluationRequest,
    ) -> Result<featureflags::TransferFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TransferFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TransferFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::TransferFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RepairFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_repair_flag_evaluation(
        &self,
        req: featureflags::RepairFlagEvaluationRequest,
    ) -> Result<featureflags::RepairFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RepairFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RepairFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::RepairFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.flag_evaluation.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run InspectFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_inspect_flag_evaluation(
        &self,
        req: featureflags::InspectFlagEvaluationRequest,
    ) -> Result<featureflags::InspectFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectFlagEvaluation", "running workflow");
        let mut response = featureflags::InspectFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FlagEvaluationAdminService for FlagEvaluationAdminController {
    /// Archives a flag evaluation.
    async fn archive_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::ArchiveFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::ArchiveFlagEvaluationResponse>, tonic::Status> {
        self.run_archive_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Restores a flag evaluation.
    async fn restore_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::RestoreFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::RestoreFlagEvaluationResponse>, tonic::Status> {
        self.run_restore_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a flag evaluation.
    async fn purge_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::PurgeFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::PurgeFlagEvaluationResponse>, tonic::Status> {
        self.run_purge_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reassigns a flag evaluation.
    async fn reassign_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::ReassignFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::ReassignFlagEvaluationResponse>, tonic::Status> {
        self.run_reassign_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Locks a flag evaluation.
    async fn lock_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::LockFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::LockFlagEvaluationResponse>, tonic::Status> {
        self.run_lock_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unlocks a flag evaluation.
    async fn unlock_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::UnlockFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::UnlockFlagEvaluationResponse>, tonic::Status> {
        self.run_unlock_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Transfers a flag evaluation.
    async fn transfer_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::TransferFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::TransferFlagEvaluationResponse>, tonic::Status> {
        self.run_transfer_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Repairs a flag evaluation.
    async fn repair_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::RepairFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::RepairFlagEvaluationResponse>, tonic::Status> {
        self.run_repair_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag evaluation.
    async fn inspect_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::InspectFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::InspectFlagEvaluationResponse>, tonic::Status> {
        self.run_inspect_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
