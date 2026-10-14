use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_notification_service_client::PermissionCheckNotificationServiceClient;
use acme_proto::acme::authorization::v1::role_binding_query_service_server::RoleBindingQueryService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;

/// Answers read-only questions about role bindings, for screens and reports.
#[derive(Clone)]
pub struct RoleBindingQueryApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
}
impl RoleBindingQueryApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::AuthorizationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run LookupRoleBinding after validating the tenant and durably recording the command.
    async fn run_lookup_role_binding(
        &self,
        req: authorization::LookupRoleBindingRequest,
    ) -> Result<authorization::LookupRoleBindingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupRoleBinding", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupRoleBinding", "running workflow");
        let mut response = authorization::LookupRoleBindingResponse::default();
        let contribution: String = req.role_binding_id.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .role_binding_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseRoleBindings after validating the tenant and durably recording the command.
    async fn run_browse_role_bindings(
        &self,
        req: authorization::BrowseRoleBindingsRequest,
    ) -> Result<authorization::BrowseRoleBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseRoleBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseRoleBindings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = authorization::BrowseRoleBindingsResponse::default();
        Ok(response)
    }
    /// Run CountRoleBindings after validating the tenant and durably recording the command.
    async fn run_count_role_bindings(
        &self,
        req: authorization::CountRoleBindingsRequest,
    ) -> Result<authorization::CountRoleBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountRoleBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountRoleBindings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = authorization::NotifyPermissionCheckWatchersRequest {
            annotations: (req.context.clone().unwrap_or_default().baggage.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(authorization::notify_permission_check_watchers_request::Details {
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
                variables: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = PermissionCheckNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::NotifyPermissionCheckWatchersRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.notify_permission_check_watchers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::CountRoleBindingsResponse::default();
        let contribution: String = reply_0
            .permission_check
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run ResolveRoleBindings after validating the tenant and durably recording the command.
    async fn run_resolve_role_bindings(
        &self,
        req: authorization::ResolveRoleBindingsRequest,
    ) -> Result<authorization::ResolveRoleBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveRoleBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveRoleBindings", "running workflow");
        let response = authorization::ResolveRoleBindingsResponse::default();
        Ok(response)
    }
    /// Run ExportRoleBindings after validating the tenant and durably recording the command.
    async fn run_export_role_bindings(
        &self,
        req: authorization::ExportRoleBindingsRequest,
    ) -> Result<authorization::ExportRoleBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportRoleBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportRoleBindings", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let mut response = authorization::ExportRoleBindingsResponse::default();
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .role_binding
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeRoleBindings after validating the tenant and durably recording the command.
    async fn run_summarize_role_bindings(
        &self,
        req: authorization::SummarizeRoleBindingsRequest,
    ) -> Result<authorization::SummarizeRoleBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeRoleBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeRoleBindings", "running workflow");
        tracing::debug!(audit_value = ?req.role_binding_id.clone(), "recorded audit field");
        let mut response = authorization::SummarizeRoleBindingsResponse::default();
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestRoleBindings after validating the tenant and durably recording the command.
    async fn run_suggest_role_bindings(
        &self,
        req: authorization::SuggestRoleBindingsRequest,
    ) -> Result<authorization::SuggestRoleBindingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestRoleBindings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestRoleBindings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::PullFlagEvaluationsRequest {
            context: req.context.clone(),
            details: Some(featureflags::pull_flag_evaluations_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    starts_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::PullFlagEvaluationsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pull_flag_evaluations(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = authorization::SuggestRoleBindingsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.flag_evaluation.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            if response.role_bindings.is_empty() {
                response.role_bindings.push(Default::default());
            }
            if response.role_bindings[0].lines.is_empty() {
                response.role_bindings[0].lines.push(Default::default());
            }
            response.role_bindings[0].lines[0].attributes.extend(contribution);
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl RoleBindingQueryService for RoleBindingQueryApi {
    /// Returns a role binding.
    async fn lookup_role_binding(
        &self,
        request: tonic::Request<authorization::LookupRoleBindingRequest>,
    ) -> Result<tonic::Response<authorization::LookupRoleBindingResponse>, tonic::Status> {
        self.run_lookup_role_binding(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists role bindings a page at a time.
    async fn browse_role_bindings(
        &self,
        request: tonic::Request<authorization::BrowseRoleBindingsRequest>,
    ) -> Result<tonic::Response<authorization::BrowseRoleBindingsResponse>, tonic::Status> {
        self.run_browse_role_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts role bindings by status.
    async fn count_role_bindings(
        &self,
        request: tonic::Request<authorization::CountRoleBindingsRequest>,
    ) -> Result<tonic::Response<authorization::CountRoleBindingsResponse>, tonic::Status> {
        self.run_count_role_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several role bindings at once.
    async fn resolve_role_bindings(
        &self,
        request: tonic::Request<authorization::ResolveRoleBindingsRequest>,
    ) -> Result<tonic::Response<authorization::ResolveRoleBindingsResponse>, tonic::Status> {
        self.run_resolve_role_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportRoleBindingsStream =
        tokio_stream::wrappers::ReceiverStream<Result<authorization::ExportRoleBindingsResponse, tonic::Status>>;
    /// Streams every matching role binding.
    async fn export_role_bindings(
        &self,
        request: tonic::Request<authorization::ExportRoleBindingsRequest>,
    ) -> Result<tonic::Response<Self::ExportRoleBindingsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_export_role_bindings(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a role binding.
    async fn summarize_role_bindings(
        &self,
        request: tonic::Request<authorization::SummarizeRoleBindingsRequest>,
    ) -> Result<tonic::Response<authorization::SummarizeRoleBindingsResponse>, tonic::Status> {
        self.run_summarize_role_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists role bindings a page at a time.
    async fn suggest_role_bindings(
        &self,
        request: tonic::Request<authorization::SuggestRoleBindingsRequest>,
    ) -> Result<tonic::Response<authorization::SuggestRoleBindingsResponse>, tonic::Status> {
        self.run_suggest_role_bindings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
