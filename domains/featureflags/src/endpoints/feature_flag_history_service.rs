use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_server::FeatureFlagHistoryService;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;

/// Keeps every version of a feature flag and restores earlier ones.
#[derive(Clone)]
pub struct FeatureFlagHistoryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FeatureFlagHistoryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run RecordFeatureFlagVersion after validating the tenant and durably recording the command.
    async fn run_record_feature_flag_version(
        &self,
        req: featureflags::RecordFeatureFlagVersionRequest,
    ) -> Result<featureflags::RecordFeatureFlagVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecordFeatureFlagVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecordFeatureFlagVersion", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::SimulateFlagEvaluationRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: req
                        .feature_flag
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: (match req.feature_flag.clone().unwrap_or_default().action.clone() {
                Some(featureflags::feature_flag::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::SimulateFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::RecordFeatureFlagVersionResponse::default();
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
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.feature_flag.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.feature_flag.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run ListFeatureFlagVersions after validating the tenant and durably recording the command.
    async fn run_list_feature_flag_versions(
        &self,
        req: featureflags::ListFeatureFlagVersionsRequest,
    ) -> Result<featureflags::ListFeatureFlagVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFeatureFlagVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFeatureFlagVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFeatureFlagVersionsResponse::default();
        Ok(response)
    }
    /// Run GetFeatureFlagVersion after validating the tenant and durably recording the command.
    async fn run_get_feature_flag_version(
        &self,
        req: featureflags::GetFeatureFlagVersionRequest,
    ) -> Result<featureflags::GetFeatureFlagVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFeatureFlagVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFeatureFlagVersion", "running workflow");
        let mut response = featureflags::GetFeatureFlagVersionResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        Ok(response)
    }
    /// Run CompareFeatureFlagVersions after validating the tenant and durably recording the command.
    async fn run_compare_feature_flag_versions(
        &self,
        req: featureflags::CompareFeatureFlagVersionsRequest,
    ) -> Result<featureflags::CompareFeatureFlagVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareFeatureFlagVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareFeatureFlagVersions", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::CompareFeatureFlagVersionsResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.feature_flag.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run RevertFeatureFlag after validating the tenant and durably recording the command.
    async fn run_revert_feature_flag(
        &self,
        req: featureflags::RevertFeatureFlagRequest,
    ) -> Result<featureflags::RevertFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RevertFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RevertFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::SyncFeatureFlagsRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    etag: req
                        .feature_flag
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_by: req
                        .feature_flag
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: req
                        .feature_flag
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                action: Some(featureflags::feature_flag::Action::Route(
                    featureflags::FeatureFlagRouteAction {
                        target: (match req.feature_flag.clone().unwrap_or_default().action.clone() {
                            Some(featureflags::feature_flag::Action::Route(value)) => value,
                            _ => Default::default(),
                        })
                        .target
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::SyncFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.sync_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::RevertFeatureFlagResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: i32 = req.feature_flag.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.feature_flag.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run AnnotateFeatureFlagVersion after validating the tenant and durably recording the command.
    async fn run_annotate_feature_flag_version(
        &self,
        req: featureflags::AnnotateFeatureFlagVersionRequest,
    ) -> Result<featureflags::AnnotateFeatureFlagVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AnnotateFeatureFlagVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AnnotateFeatureFlagVersion", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::AnnotateFeatureFlagVersionResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.feature_flag.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PruneFeatureFlagVersions after validating the tenant and durably recording the command.
    async fn run_prune_feature_flag_versions(
        &self,
        req: featureflags::PruneFeatureFlagVersionsRequest,
    ) -> Result<featureflags::PruneFeatureFlagVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PruneFeatureFlagVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PruneFeatureFlagVersions", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag_id.clone(), "recorded audit field");
        let response = featureflags::PruneFeatureFlagVersionsResponse::default();
        Ok(response)
    }
}
acme_platform::endpoints! { FeatureFlagHistoryService, FeatureFlagHistoryEndpoint;
unary {
record_feature_flag_version (featureflags::RecordFeatureFlagVersionRequest) -> featureflags::RecordFeatureFlagVersionResponse => run_record_feature_flag_version;
list_feature_flag_versions (featureflags::ListFeatureFlagVersionsRequest) -> featureflags::ListFeatureFlagVersionsResponse => run_list_feature_flag_versions;
get_feature_flag_version (featureflags::GetFeatureFlagVersionRequest) -> featureflags::GetFeatureFlagVersionResponse => run_get_feature_flag_version;
compare_feature_flag_versions (featureflags::CompareFeatureFlagVersionsRequest) -> featureflags::CompareFeatureFlagVersionsResponse => run_compare_feature_flag_versions;
revert_feature_flag (featureflags::RevertFeatureFlagRequest) -> featureflags::RevertFeatureFlagResponse => run_revert_feature_flag;
annotate_feature_flag_version (featureflags::AnnotateFeatureFlagVersionRequest) -> featureflags::AnnotateFeatureFlagVersionResponse => run_annotate_feature_flag_version;
prune_feature_flag_versions (featureflags::PruneFeatureFlagVersionsRequest) -> featureflags::PruneFeatureFlagVersionsResponse => run_prune_feature_flag_versions;
}
server {
}
client {
}
bidi {
}
}
