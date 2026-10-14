use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_query_service_server::AccessPolicyQueryService;
use acme_proto::acme::authorization::v1::role_binding_export_service_client::RoleBindingExportServiceClient;
use acme_proto::acme::authorization::v1::role_binding_validation_service_client::RoleBindingValidationServiceClient;
use acme_proto::acme::common::v1 as common;

/// Answers read-only questions about access policies, for screens and reports.
#[derive(Clone)]
pub struct AccessPolicyQueryApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    browse_access_policies_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            authorization::ShareRoleBindingExportRequest,
            authorization::ShareRoleBindingExportResponse,
        >,
    >,
    export_access_policies_0_port:
        acme_platform::ports::Callback<authorization::ScoreRoleBindingRequest, authorization::ScoreRoleBindingResponse>,
}
impl AccessPolicyQueryApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            authorization::ShareRoleBindingExportRequest,
            authorization::ShareRoleBindingExportResponse,
        > = {
            let client = RoleBindingExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::ShareRoleBindingExportRequest| -> acme_platform::ports::RpcFuture<authorization::ShareRoleBindingExportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_role_binding_export(request).await?.into_inner())
})
})
        };
        let browse_access_policies_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            authorization::ScoreRoleBindingRequest,
            authorization::ScoreRoleBindingResponse,
        > = {
            let client = RoleBindingValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::ScoreRoleBindingRequest| -> acme_platform::ports::RpcFuture<authorization::ScoreRoleBindingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_role_binding(request).await?.into_inner())
})
})
        };
        let export_access_policies_0_port = callback;
        Self {
            channel,
            repository,
            browse_access_policies_0_port,
            export_access_policies_0_port,
        }
    }
    /// Run LookupAccessPolicy after validating the tenant and durably recording the command.
    async fn run_lookup_access_policy(
        &self,
        req: authorization::LookupAccessPolicyRequest,
    ) -> Result<authorization::LookupAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupAccessPolicy", "running workflow");
        let mut response = authorization::LookupAccessPolicyResponse::default();
        let contribution: String = req.access_policy_id.clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .access_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseAccessPolicies after validating the tenant and durably recording the command.
    async fn run_browse_access_policies(
        &self,
        req: authorization::BrowseAccessPoliciesRequest,
    ) -> Result<authorization::BrowseAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = authorization::ShareRoleBindingExportRequest {
            context: req.context.clone(),
            details: Some(authorization::share_role_binding_export_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
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
        let reply_0 =
            self.browse_access_policies_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = authorization::BrowseAccessPoliciesResponse::default();
        let contribution: String = reply_0
            .role_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response.access_policies.is_empty() {
                response.access_policies.push(Default::default());
            }
            response.access_policies[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_0
            .role_binding
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .access_policy_id
            .clone();
        if contribution != <String>::default() {
            if response.access_policies.is_empty() {
                response.access_policies.push(Default::default());
            }
            response.access_policies[0].access_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run FindAccessPolicies after validating the tenant and durably recording the command.
    async fn run_find_access_policies(
        &self,
        req: authorization::FindAccessPoliciesRequest,
    ) -> Result<authorization::FindAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::FindAccessPoliciesResponse::default();
        Ok(response)
    }
    /// Run CountAccessPolicies after validating the tenant and durably recording the command.
    async fn run_count_access_policies(
        &self,
        req: authorization::CountAccessPoliciesRequest,
    ) -> Result<authorization::CountAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::CountAccessPoliciesResponse::default();
        Ok(response)
    }
    /// Run ResolveAccessPolicies after validating the tenant and durably recording the command.
    async fn run_resolve_access_policies(
        &self,
        req: authorization::ResolveAccessPoliciesRequest,
    ) -> Result<authorization::ResolveAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveAccessPolicies", "running workflow");
        let response = authorization::ResolveAccessPoliciesResponse::default();
        Ok(response)
    }
    /// Run ExportAccessPolicies after validating the tenant and durably recording the command.
    async fn run_export_access_policies(
        &self,
        req: authorization::ExportAccessPoliciesRequest,
    ) -> Result<authorization::ExportAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportAccessPolicies", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = authorization::ScoreRoleBindingRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
                audit: Some(common::AuditInfo {
                    created_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.export_access_policies_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = authorization::ExportAccessPoliciesResponse::default();
        let contribution: String = reply_0
            .role_binding
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .access_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeAccessPolicies after validating the tenant and durably recording the command.
    async fn run_summarize_access_policies(
        &self,
        req: authorization::SummarizeAccessPoliciesRequest,
    ) -> Result<authorization::SummarizeAccessPoliciesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SummarizeAccessPolicies", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeAccessPolicies", "running workflow");
        tracing::debug!(audit_value = ?req.access_policy_id.clone(), "recorded audit field");
        let mut response = authorization::SummarizeAccessPoliciesResponse::default();
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run PeekAccessPolicy after validating the tenant and durably recording the command.
    async fn run_peek_access_policy(
        &self,
        req: authorization::PeekAccessPolicyRequest,
    ) -> Result<authorization::PeekAccessPolicyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekAccessPolicy", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekAccessPolicy", "running workflow");
        let mut response = authorization::PeekAccessPolicyResponse::default();
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
impl AccessPolicyQueryService for AccessPolicyQueryApi {
    /// Returns an access policy.
    async fn lookup_access_policy(
        &self,
        request: tonic::Request<authorization::LookupAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::LookupAccessPolicyResponse>, tonic::Status> {
        self.run_lookup_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists access policies a page at a time.
    async fn browse_access_policies(
        &self,
        request: tonic::Request<authorization::BrowseAccessPoliciesRequest>,
    ) -> Result<tonic::Response<authorization::BrowseAccessPoliciesResponse>, tonic::Status> {
        self.run_browse_access_policies(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists access policies a page at a time.
    async fn find_access_policies(
        &self,
        request: tonic::Request<authorization::FindAccessPoliciesRequest>,
    ) -> Result<tonic::Response<authorization::FindAccessPoliciesResponse>, tonic::Status> {
        self.run_find_access_policies(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts access policies by status.
    async fn count_access_policies(
        &self,
        request: tonic::Request<authorization::CountAccessPoliciesRequest>,
    ) -> Result<tonic::Response<authorization::CountAccessPoliciesResponse>, tonic::Status> {
        self.run_count_access_policies(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several access policies at once.
    async fn resolve_access_policies(
        &self,
        request: tonic::Request<authorization::ResolveAccessPoliciesRequest>,
    ) -> Result<tonic::Response<authorization::ResolveAccessPoliciesResponse>, tonic::Status> {
        self.run_resolve_access_policies(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportAccessPoliciesStream =
        tokio_stream::wrappers::ReceiverStream<Result<authorization::ExportAccessPoliciesResponse, tonic::Status>>;
    /// Streams every matching access policy.
    async fn export_access_policies(
        &self,
        request: tonic::Request<authorization::ExportAccessPoliciesRequest>,
    ) -> Result<tonic::Response<Self::ExportAccessPoliciesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_export_access_policies(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes an access policy.
    async fn summarize_access_policies(
        &self,
        request: tonic::Request<authorization::SummarizeAccessPoliciesRequest>,
    ) -> Result<tonic::Response<authorization::SummarizeAccessPoliciesResponse>, tonic::Status> {
        self.run_summarize_access_policies(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an access policy.
    async fn peek_access_policy(
        &self,
        request: tonic::Request<authorization::PeekAccessPolicyRequest>,
    ) -> Result<tonic::Response<authorization::PeekAccessPolicyResponse>, tonic::Status> {
        self.run_peek_access_policy(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
