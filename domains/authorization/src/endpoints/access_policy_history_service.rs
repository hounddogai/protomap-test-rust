use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_server::AccessPolicyHistoryService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_query_service_client::FeatureFlagQueryServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Keeps every version of an access policy and restores earlier ones.
#[derive(Clone)]
pub struct AccessPolicyHistoryHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    list_access_policy_versions_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::CheckFeatureFlagEligibilityRequest,
            featureflags::CheckFeatureFlagEligibilityResponse,
        >,
    >,
}
impl AccessPolicyHistoryHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::CheckFeatureFlagEligibilityRequest,
            featureflags::CheckFeatureFlagEligibilityResponse,
        > = {
            let client = FeatureFlagValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::CheckFeatureFlagEligibilityRequest| -> acme_platform::ports::RpcFuture<featureflags::CheckFeatureFlagEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_feature_flag_eligibility(request).await?.into_inner())
})
})
        };
        let list_access_policy_versions_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            list_access_policy_versions_0_port,
        }
    }
    /// Run RecordAccessPolicyVersion after validating the tenant and durably recording the command.
    async fn run_record_access_policy_version(
        &self,
        req: authorization::RecordAccessPolicyVersionRequest,
    ) -> Result<authorization::RecordAccessPolicyVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecordAccessPolicyVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecordAccessPolicyVersion", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::RecordAccessPolicyVersionResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.access_policy.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ListAccessPolicyVersions after validating the tenant and durably recording the command.
    async fn run_list_access_policy_versions(
        &self,
        req: authorization::ListAccessPolicyVersionsRequest,
    ) -> Result<authorization::ListAccessPolicyVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListAccessPolicyVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListAccessPolicyVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CheckFeatureFlagEligibilityRequest {
            context: req.context.clone(),
            details: Some(featureflags::check_feature_flag_eligibility_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
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
        let reply_0 =
            acme_platform::ports::execute(self.list_access_policy_versions_0_port.as_ref(), outgoing_0).await?;
        let mut response = authorization::ListAccessPolicyVersionsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run GetAccessPolicyVersion after validating the tenant and durably recording the command.
    async fn run_get_access_policy_version(
        &self,
        req: authorization::GetAccessPolicyVersionRequest,
    ) -> Result<authorization::GetAccessPolicyVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetAccessPolicyVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetAccessPolicyVersion", "running workflow");
        let mut response = authorization::GetAccessPolicyVersionResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run CompareAccessPolicyVersions after validating the tenant and durably recording the command.
    async fn run_compare_access_policy_versions(
        &self,
        req: authorization::CompareAccessPolicyVersionsRequest,
    ) -> Result<authorization::CompareAccessPolicyVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareAccessPolicyVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareAccessPolicyVersions", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::CompareAccessPolicyVersionsResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: i64 = req.details.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.access_policy.get_or_insert_with(Default::default).version = contribution;
        }
        Ok(response)
    }
    /// Run RevertAccessPolicy after validating the tenant and durably recording the command.
    async fn run_revert_access_policy(
        &self,
        req: authorization::RevertAccessPolicyRequest,
    ) -> Result<authorization::RevertAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RevertAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RevertAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let outgoing_0 = venues::ExplainVenueDecisionRequest {
            context: req.context.clone(),
            reason: (match req.access_policy.clone().unwrap_or_default().action.clone() {
                Some(authorization::access_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            venue: Some(venues::Venue {
                owner: Some(identity::UserRef {
                    display_name: req
                        .access_policy
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .access_policy
                        .clone()
                        .unwrap_or_default()
                        .owner
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
            let input: tonic::Request<venues::ExplainVenueDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_venue_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::RevertAccessPolicyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: i32 = req.access_policy.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.access_policy.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run AnnotateAccessPolicyVersion after validating the tenant and durably recording the command.
    async fn run_annotate_access_policy_version(
        &self,
        req: authorization::AnnotateAccessPolicyVersionRequest,
    ) -> Result<authorization::AnnotateAccessPolicyVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AnnotateAccessPolicyVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AnnotateAccessPolicyVersion", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ExportFeatureFlagsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::export_feature_flags_request::Filter {
                created_after: req
                    .access_policy
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.access_policy.clone().unwrap_or_default().parameters.clone(),
                owner_id: req.access_policy.clone().unwrap_or_default().access_policy_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = FeatureFlagQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExportFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_feature_flags(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::ExportFeatureFlagsResponse::default();
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
        let mut response = authorization::AnnotateAccessPolicyVersionResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        let contribution: i32 = req.access_policy.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.access_policy.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run PruneAccessPolicyVersions after validating the tenant and durably recording the command.
    async fn run_prune_access_policy_versions(
        &self,
        req: authorization::PruneAccessPolicyVersionsRequest,
    ) -> Result<authorization::PruneAccessPolicyVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PruneAccessPolicyVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PruneAccessPolicyVersions", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy_id.clone(), "recorded audit field");
        let response = authorization::PruneAccessPolicyVersionsResponse::default();
        Ok(response)
    }
}
#[tonic::async_trait]
impl AccessPolicyHistoryService for AccessPolicyHistoryHandler {
    /// Records an access policy.
    async fn record_access_policy_version(
        &self,
        request: tonic::Request<authorization::RecordAccessPolicyVersionRequest>,
    ) -> Result<tonic::Response<authorization::RecordAccessPolicyVersionResponse>, tonic::Status> {
        self.run_record_access_policy_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists access policies a page at a time.
    async fn list_access_policy_versions(
        &self,
        request: tonic::Request<authorization::ListAccessPolicyVersionsRequest>,
    ) -> Result<tonic::Response<authorization::ListAccessPolicyVersionsResponse>, tonic::Status> {
        self.run_list_access_policy_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an access policy.
    async fn get_access_policy_version(
        &self,
        request: tonic::Request<authorization::GetAccessPolicyVersionRequest>,
    ) -> Result<tonic::Response<authorization::GetAccessPolicyVersionResponse>, tonic::Status> {
        self.run_get_access_policy_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares an access policy.
    async fn compare_access_policy_versions(
        &self,
        request: tonic::Request<authorization::CompareAccessPolicyVersionsRequest>,
    ) -> Result<tonic::Response<authorization::CompareAccessPolicyVersionsResponse>, tonic::Status> {
        self.run_compare_access_policy_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reverts an access policy.
    async fn revert_access_policy(
        &self,
        request: tonic::Request<authorization::RevertAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::RevertAccessPolicyResponse>, tonic::Status> {
        self.run_revert_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Annotates an access policy.
    async fn annotate_access_policy_version(
        &self,
        request: tonic::Request<authorization::AnnotateAccessPolicyVersionRequest>,
    ) -> Result<tonic::Response<authorization::AnnotateAccessPolicyVersionResponse>, tonic::Status> {
        self.run_annotate_access_policy_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes an access policy.
    async fn prune_access_policy_versions(
        &self,
        request: tonic::Request<authorization::PruneAccessPolicyVersionsRequest>,
    ) -> Result<tonic::Response<authorization::PruneAccessPolicyVersionsResponse>, tonic::Status> {
        self.run_prune_access_policy_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
