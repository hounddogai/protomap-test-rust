use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_query_service_server::ApiKeyQueryService;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_client::RouteBindingIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_query_service_client::RouteBindingQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_validation_service_client::UserValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;
use acme_proto::acme::tours::v1 as tours;
use acme_proto::acme::tours::v1::tour_service_client::TourServiceClient;

/// Answers read-only questions about api keys, for screens and reports.
#[derive(Clone)]
pub struct ApiKeyQueryHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
}
impl ApiKeyQueryHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run LookupApiKey after validating the tenant and durably recording the command.
    async fn run_lookup_api_key(
        &self,
        req: edgegateway::LookupApiKeyRequest,
    ) -> Result<edgegateway::LookupApiKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupApiKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupApiKey", "running workflow");
        let outgoing_0 = tours::BatchCreateToursRequest {
            context: req.context.clone(),
            chunk: Some(tours::batch_create_tours_request::Chunk::Record(tours::Tour {
                organizer: Some(identity::UserRef {
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
                participants: vec![tours::TourParticipant {
                    contact: Some(common::ContactInfo {
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
                }],
                ..Default::default()
            })),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = TourServiceClient::new(channel);
            let input: tonic::Request<tours::BatchCreateToursRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_tours(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        updated_at: req.context.clone().unwrap_or_default().deadline.clone(),
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_feature_flag_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFeatureFlagChangesResponse::default();
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
        let mut response = edgegateway::LookupApiKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseApiKeys after validating the tenant and durably recording the command.
    async fn run_browse_api_keys(
        &self,
        req: edgegateway::BrowseApiKeysRequest,
    ) -> Result<edgegateway::BrowseApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseApiKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::TestRouteBindingConnectionRequest {
            context: req.context.clone(),
            details: Some(edgegateway::test_route_binding_connection_request::Details {
                received_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::TestRouteBindingConnectionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.test_route_binding_connection(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::BrowseApiKeysResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0
            .route_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            if response.api_keys.is_empty() {
                response.api_keys.push(Default::default());
            }
            response.api_keys[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        Ok(response)
    }
    /// Run CountApiKeys after validating the tenant and durably recording the command.
    async fn run_count_api_keys(
        &self,
        req: edgegateway::CountApiKeysRequest,
    ) -> Result<edgegateway::CountApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountApiKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = edgegateway::CountApiKeysResponse::default();
        Ok(response)
    }
    /// Run ResolveApiKeys after validating the tenant and durably recording the command.
    async fn run_resolve_api_keys(
        &self,
        req: edgegateway::ResolveApiKeysRequest,
    ) -> Result<edgegateway::ResolveApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveApiKeys", "running workflow");
        let outgoing_0 = edgegateway::StartGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            gateway_policy: Some(edgegateway::GatewayPolicy {
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
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::StartGatewayPolicyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.start_gateway_policy_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::ResolveApiKeysResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.api_keys.is_empty() {
                response.api_keys.push(Default::default());
            }
            response.api_keys[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        Ok(response)
    }
    /// Run ExportApiKeys after validating the tenant and durably recording the command.
    async fn run_export_api_keys(
        &self,
        req: edgegateway::ExportApiKeysRequest,
    ) -> Result<edgegateway::ExportApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportApiKeys", "running workflow");
        tracing::debug!(audit_value = ?req.destination_uri.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFlagEvaluationConflictsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_evaluation_conflicts_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ListFlagEvaluationConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_flag_evaluation_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::SummarizeRouteBindingsRequest {
            annotations: reply_0.result.clone().unwrap_or_default().details.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::summarize_route_bindings_request::Details {
                filters: req.filter.clone().unwrap_or_default().labels.clone(),
                format: req.format.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RouteBindingQueryServiceClient::new(channel);
            let input: tonic::Request<edgegateway::SummarizeRouteBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_route_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = identity::LintUsersRequest {
            context: req.context.clone(),
            details: Some(identity::lint_users_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            user: Some(identity::User {
                contact: Some(common::ContactInfo {
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
                primary_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                relationship_manager: Some(identity::UserRef {
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
        let reply_2 = async move {
            let mut client = UserValidationServiceClient::new(channel);
            let input: tonic::Request<identity::LintUsersRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_users(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::ExportApiKeysResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.filter.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run SummarizeApiKeys after validating the tenant and durably recording the command.
    async fn run_summarize_api_keys(
        &self,
        req: edgegateway::SummarizeApiKeysRequest,
    ) -> Result<edgegateway::SummarizeApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeApiKeys", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::ListGatewayPolicyWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_gateway_policy_workflows_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.details.clone().unwrap_or_default().filters.clone(),
                owner_id: req.api_key_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListGatewayPolicyWorkflowsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_gateway_policy_workflows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::CountThrottleTotalsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::count_throttle_totals_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: req.api_key_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CountThrottleTotalsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_throttle_totals(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = identity::NormalizeUserRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = UserValidationServiceClient::new(channel);
            let input: tonic::Request<identity::NormalizeUserRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_user(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::SummarizeApiKeysResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestApiKeys after validating the tenant and durably recording the command.
    async fn run_suggest_api_keys(
        &self,
        req: edgegateway::SuggestApiKeysRequest,
    ) -> Result<edgegateway::SuggestApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestApiKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = identity::CheckUserEligibilityRequest {
            context: req.context.clone(),
            user: Some(identity::User {
                contact: Some(common::ContactInfo {
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
                primary_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
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
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = UserValidationServiceClient::new(channel);
            let input: tonic::Request<identity::CheckUserEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_user_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::SuggestApiKeysResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .user
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.api_keys.is_empty() {
                response.api_keys.push(Default::default());
            }
            response.api_keys[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        Ok(response)
    }
    /// Run PeekApiKey after validating the tenant and durably recording the command.
    async fn run_peek_api_key(
        &self,
        req: edgegateway::PeekApiKeyRequest,
    ) -> Result<edgegateway::PeekApiKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekApiKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekApiKey", "running workflow");
        let outgoing_0 = featureflags::NormalizeFlagEvaluationRequest {
            context: req.context.clone(),
            details: Some(featureflags::normalize_flag_evaluation_request::Details {
                contact: Some(common::ContactInfo {
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
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::NormalizeFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::PeekApiKeyResponse::default();
        let contribution: String = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { ApiKeyQueryService, ApiKeyQueryHandler;
unary {
lookup_api_key (edgegateway::LookupApiKeyRequest) -> edgegateway::LookupApiKeyResponse => run_lookup_api_key;
browse_api_keys (edgegateway::BrowseApiKeysRequest) -> edgegateway::BrowseApiKeysResponse => run_browse_api_keys;
count_api_keys (edgegateway::CountApiKeysRequest) -> edgegateway::CountApiKeysResponse => run_count_api_keys;
resolve_api_keys (edgegateway::ResolveApiKeysRequest) -> edgegateway::ResolveApiKeysResponse => run_resolve_api_keys;
summarize_api_keys (edgegateway::SummarizeApiKeysRequest) -> edgegateway::SummarizeApiKeysResponse => run_summarize_api_keys;
suggest_api_keys (edgegateway::SuggestApiKeysRequest) -> edgegateway::SuggestApiKeysResponse => run_suggest_api_keys;
peek_api_key (edgegateway::PeekApiKeyRequest) -> edgegateway::PeekApiKeyResponse => run_peek_api_key;
}
server {
export_api_keys (edgegateway::ExportApiKeysRequest) -> edgegateway::ExportApiKeysResponse, ExportApiKeysStream => run_export_api_keys;
}
client {
}
bidi {
}
}
