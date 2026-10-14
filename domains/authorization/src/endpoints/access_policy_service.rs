use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_server::AccessPolicyService;
use acme_proto::acme::common::v1 as common;

/// Manages access policies: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct AccessPolicyEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl AccessPolicyEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run CreateAccessPolicy after validating the tenant and durably recording the command.
    async fn run_create_access_policy(
        &self,
        req: authorization::CreateAccessPolicyRequest,
    ) -> Result<authorization::CreateAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::CreateAccessPolicyResponse::default();
        let contribution: i64 = req.access_policy.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.access_policy.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = req.access_policy.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_policy.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run GetAccessPolicy after validating the tenant and durably recording the command.
    async fn run_get_access_policy(
        &self,
        req: authorization::GetAccessPolicyRequest,
    ) -> Result<authorization::GetAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetAccessPolicy", "running workflow");
        let mut response = authorization::GetAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run UpdateAccessPolicy after validating the tenant and durably recording the command.
    async fn run_update_access_policy(
        &self,
        req: authorization::UpdateAccessPolicyRequest,
    ) -> Result<authorization::UpdateAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UpdateAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UpdateAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::UpdateAccessPolicyResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            req.access_policy.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        let contribution: String = req.access_policy.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_policy.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run DeleteAccessPolicy after validating the tenant and durably recording the command.
    async fn run_delete_access_policy(
        &self,
        req: authorization::DeleteAccessPolicyRequest,
    ) -> Result<authorization::DeleteAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy_id.clone(), "recorded audit field");
        let response = authorization::DeleteAccessPolicyResponse::default();
        Ok(response)
    }
    /// Run ListAccessPolicies after validating the tenant and durably recording the command.
    async fn run_list_access_policies(
        &self,
        req: authorization::ListAccessPoliciesRequest,
    ) -> Result<authorization::ListAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::ListAccessPoliciesResponse::default();
        Ok(response)
    }
    /// Run BatchGetAccessPolicies after validating the tenant and durably recording the command.
    async fn run_batch_get_access_policies(
        &self,
        req: authorization::BatchGetAccessPoliciesRequest,
    ) -> Result<authorization::BatchGetAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchGetAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchGetAccessPolicies", "running workflow");
        let response = authorization::BatchGetAccessPoliciesResponse::default();
        Ok(response)
    }
    /// Run WatchAccessPolicies after validating the tenant and durably recording the command.
    async fn run_watch_access_policies(
        &self,
        req: authorization::WatchAccessPoliciesRequest,
    ) -> Result<authorization::WatchAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("WatchAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "WatchAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let mut response = authorization::WatchAccessPoliciesResponse::default();
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run BatchCreateAccessPolicies after validating the tenant and durably recording the command.
    async fn run_batch_create_access_policies(
        &self,
        req: authorization::BatchCreateAccessPoliciesRequest,
    ) -> Result<authorization::BatchCreateAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("BatchCreateAccessPolicies", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(authorization::batch_create_access_policies_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let response = authorization::BatchCreateAccessPoliciesResponse::default();
        Ok(response)
    }
    /// Run CloneAccessPolicy after validating the tenant and durably recording the command.
    async fn run_clone_access_policy(
        &self,
        req: authorization::CloneAccessPolicyRequest,
    ) -> Result<authorization::CloneAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CloneAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CloneAccessPolicy", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy.clone().unwrap_or_default().access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::CloneAccessPolicyResponse::default();
        let contribution: String = req.access_policy.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_policy.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.access_policy.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_policy.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl AccessPolicyService for AccessPolicyEndpoint {
    /// Creates an access policy.
    async fn create_access_policy(
        &self,
        request: tonic::Request<authorization::CreateAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::CreateAccessPolicyResponse>, tonic::Status> {
        self.run_create_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an access policy.
    async fn get_access_policy(
        &self,
        request: tonic::Request<authorization::GetAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::GetAccessPolicyResponse>, tonic::Status> {
        self.run_get_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Changes an access policy.
    async fn update_access_policy(
        &self,
        request: tonic::Request<authorization::UpdateAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::UpdateAccessPolicyResponse>, tonic::Status> {
        self.run_update_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes an access policy.
    async fn delete_access_policy(
        &self,
        request: tonic::Request<authorization::DeleteAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::DeleteAccessPolicyResponse>, tonic::Status> {
        self.run_delete_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists access policies a page at a time.
    async fn list_access_policies(
        &self,
        request: tonic::Request<authorization::ListAccessPoliciesRequest>,
    ) -> Result<tonic::Response<authorization::ListAccessPoliciesResponse>, tonic::Status> {
        self.run_list_access_policies(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several access policies at once.
    async fn batch_get_access_policies(
        &self,
        request: tonic::Request<authorization::BatchGetAccessPoliciesRequest>,
    ) -> Result<tonic::Response<authorization::BatchGetAccessPoliciesResponse>, tonic::Status> {
        self.run_batch_get_access_policies(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type WatchAccessPoliciesStream =
        tokio_stream::wrappers::ReceiverStream<Result<authorization::WatchAccessPoliciesResponse, tonic::Status>>;
    /// Streams the changes of access policies as they happen.
    async fn watch_access_policies(
        &self,
        request: tonic::Request<authorization::WatchAccessPoliciesRequest>,
    ) -> Result<tonic::Response<Self::WatchAccessPoliciesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_watch_access_policies(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Takes a stream of access policies and answers once at the end.
    async fn batch_create_access_policies(
        &self,
        request: tonic::Request<tonic::Streaming<authorization::BatchCreateAccessPoliciesRequest>>,
    ) -> Result<tonic::Response<authorization::BatchCreateAccessPoliciesResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = authorization::BatchCreateAccessPoliciesResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_batch_create_access_policies(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Creates an access policy.
    async fn clone_access_policy(
        &self,
        request: tonic::Request<authorization::CloneAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::CloneAccessPolicyResponse>, tonic::Status> {
        self.run_clone_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
