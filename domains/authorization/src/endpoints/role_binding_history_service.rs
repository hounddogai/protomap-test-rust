use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::authorization::v1::role_binding_history_service_server::RoleBindingHistoryService;
use acme_proto::acme::common::v1 as common;

/// Keeps every version of a role binding and restores earlier ones.
#[derive(Clone)]
pub struct RoleBindingHistoryHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl RoleBindingHistoryHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run RecordRoleBindingVersion after validating the tenant and durably recording the command.
    async fn run_record_role_binding_version(
        &self,
        req: authorization::RecordRoleBindingVersionRequest,
    ) -> Result<authorization::RecordRoleBindingVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecordRoleBindingVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecordRoleBindingVersion", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let jobs_0 = req
            .role_binding
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = authorization::RevertAccessPolicyRequest {
                        access_policy: Some(authorization::AccessPolicy {
                            audit: Some(common::AuditInfo {
                                created_at: req
                                    .role_binding
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .created_at
                                    .clone(),
                                etag: req
                                    .role_binding
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .etag
                                    .clone(),
                                ..Default::default()
                            }),
                            description: item.description.clone(),
                            action: Some(authorization::access_policy::Action::Discount(
                                authorization::AccessPolicyDiscountAction {
                                    amount: item.tax.clone().unwrap_or_default().amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AccessPolicyHistoryServiceClient::new(channel);
                        let input: tonic::Request<authorization::RevertAccessPolicyRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.revert_access_policy(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = authorization::RecordRoleBindingVersionResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: Option<common::Money> = req.role_binding.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.role_binding.get_or_insert_with(Default::default).total = contribution;
        }
        Ok(response)
    }
    /// Run ListRoleBindingVersions after validating the tenant and durably recording the command.
    async fn run_list_role_binding_versions(
        &self,
        req: authorization::ListRoleBindingVersionsRequest,
    ) -> Result<authorization::ListRoleBindingVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListRoleBindingVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRoleBindingVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::ListRoleBindingVersionsResponse::default();
        Ok(response)
    }
    /// Run GetRoleBindingVersion after validating the tenant and durably recording the command.
    async fn run_get_role_binding_version(
        &self,
        req: authorization::GetRoleBindingVersionRequest,
    ) -> Result<authorization::GetRoleBindingVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRoleBindingVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRoleBindingVersion", "running workflow");
        let mut response = authorization::GetRoleBindingVersionResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run CompareRoleBindingVersions after validating the tenant and durably recording the command.
    async fn run_compare_role_binding_versions(
        &self,
        req: authorization::CompareRoleBindingVersionsRequest,
    ) -> Result<authorization::CompareRoleBindingVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareRoleBindingVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareRoleBindingVersions", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let jobs_0 = req
            .role_binding
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = authorization::RevertAccessPolicyRequest {
                        access_policy: Some(authorization::AccessPolicy {
                            action: Some(authorization::access_policy::Action::Discount(
                                authorization::AccessPolicyDiscountAction {
                                    amount: item.tax.clone().unwrap_or_default().amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AccessPolicyHistoryServiceClient::new(channel);
                        let input: tonic::Request<authorization::RevertAccessPolicyRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.revert_access_policy(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = authorization::CompareRoleBindingVersionsResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: Option<authorization::AccessPolicyRef> =
            req.role_binding.clone().unwrap_or_default().counterparty.clone();
        if contribution != <Option<authorization::AccessPolicyRef>>::default() {
            response.role_binding.get_or_insert_with(Default::default).counterparty = contribution;
        }
        Ok(response)
    }
    /// Run RevertRoleBinding after validating the tenant and durably recording the command.
    async fn run_revert_role_binding(
        &self,
        req: authorization::RevertRoleBindingRequest,
    ) -> Result<authorization::RevertRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RevertRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RevertRoleBinding", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::RevertRoleBindingResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: String = req.role_binding.clone().unwrap_or_default().notes.clone();
        if contribution != <String>::default() {
            response.role_binding.get_or_insert_with(Default::default).notes = contribution;
        }
        Ok(response)
    }
    /// Run AnnotateRoleBindingVersion after validating the tenant and durably recording the command.
    async fn run_annotate_role_binding_version(
        &self,
        req: authorization::AnnotateRoleBindingVersionRequest,
    ) -> Result<authorization::AnnotateRoleBindingVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AnnotateRoleBindingVersion", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AnnotateRoleBindingVersion", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding.clone().unwrap_or_default().role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::AnnotateRoleBindingVersionResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        let contribution: Option<common::PostalAddress> =
            req.role_binding.clone().unwrap_or_default().billing_address.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .billing_address = contribution;
        }
        Ok(response)
    }
    /// Run PruneRoleBindingVersions after validating the tenant and durably recording the command.
    async fn run_prune_role_binding_versions(
        &self,
        req: authorization::PruneRoleBindingVersionsRequest,
    ) -> Result<authorization::PruneRoleBindingVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PruneRoleBindingVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PruneRoleBindingVersions", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding_id.clone(), "recorded audit field");
        let response = authorization::PruneRoleBindingVersionsResponse::default();
        Ok(response)
    }
}
#[tonic::async_trait]
impl RoleBindingHistoryService for RoleBindingHistoryHandler {
    /// Records a role binding.
    async fn record_role_binding_version(
        &self,
        request: tonic::Request<authorization::RecordRoleBindingVersionRequest>,
    ) -> Result<tonic::Response<authorization::RecordRoleBindingVersionResponse>, tonic::Status> {
        self.run_record_role_binding_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists role bindings a page at a time.
    async fn list_role_binding_versions(
        &self,
        request: tonic::Request<authorization::ListRoleBindingVersionsRequest>,
    ) -> Result<tonic::Response<authorization::ListRoleBindingVersionsResponse>, tonic::Status> {
        self.run_list_role_binding_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a role binding.
    async fn get_role_binding_version(
        &self,
        request: tonic::Request<authorization::GetRoleBindingVersionRequest>,
    ) -> Result<tonic::Response<authorization::GetRoleBindingVersionResponse>, tonic::Status> {
        self.run_get_role_binding_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a role binding.
    async fn compare_role_binding_versions(
        &self,
        request: tonic::Request<authorization::CompareRoleBindingVersionsRequest>,
    ) -> Result<tonic::Response<authorization::CompareRoleBindingVersionsResponse>, tonic::Status> {
        self.run_compare_role_binding_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reverts a role binding.
    async fn revert_role_binding(
        &self,
        request: tonic::Request<authorization::RevertRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::RevertRoleBindingResponse>, tonic::Status> {
        self.run_revert_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Annotates a role binding.
    async fn annotate_role_binding_version(
        &self,
        request: tonic::Request<authorization::AnnotateRoleBindingVersionRequest>,
    ) -> Result<tonic::Response<authorization::AnnotateRoleBindingVersionResponse>, tonic::Status> {
        self.run_annotate_role_binding_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a role binding.
    async fn prune_role_binding_versions(
        &self,
        request: tonic::Request<authorization::PruneRoleBindingVersionsRequest>,
    ) -> Result<tonic::Response<authorization::PruneRoleBindingVersionsResponse>, tonic::Status> {
        self.run_prune_role_binding_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
