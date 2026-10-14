use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_server::ApiKeyWorkflowService;
use acme_proto::acme::edgegateway::v1::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_validation_service_client::SearchIndexValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_version_approval_service_client::SecretVersionApprovalServiceClient;
use acme_proto::acme::yards::v1 as yards;
use acme_proto::acme::yards::v1::yard_service_client::YardServiceClient;

/// Runs the long-lived workflow that takes an api key from start to finish.
#[derive(Clone)]
pub struct ApiKeyWorkflowApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    retry_api_key_workflow_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<secrets::SubmitSecretVersionRequest, secrets::SubmitSecretVersionResponse>,
    >,
    retry_api_key_workflow_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                edgegateway::ScheduleGatewayPolicyReportRequest,
                edgegateway::ScheduleGatewayPolicyReportResponse,
            >,
    >,
    signal_api_key_workflow_2_port:
        acme_platform::ports::Callback<search::LintSearchIndexesRequest, search::LintSearchIndexesResponse>,
}
impl ApiKeyWorkflowApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            secrets::SubmitSecretVersionRequest,
            secrets::SubmitSecretVersionResponse,
        > = {
            let client = SecretVersionApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::SubmitSecretVersionRequest| -> acme_platform::ports::RpcFuture<secrets::SubmitSecretVersionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.submit_secret_version(request).await?.into_inner())
})
})
        };
        let retry_api_key_workflow_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            edgegateway::ScheduleGatewayPolicyReportRequest,
            edgegateway::ScheduleGatewayPolicyReportResponse,
        > = {
            let client = GatewayPolicyReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::ScheduleGatewayPolicyReportRequest| -> acme_platform::ports::RpcFuture<edgegateway::ScheduleGatewayPolicyReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.schedule_gateway_policy_report(request).await?.into_inner())
})
})
        };
        let retry_api_key_workflow_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            search::LintSearchIndexesRequest,
            search::LintSearchIndexesResponse,
        > = {
            let client = SearchIndexValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: search::LintSearchIndexesRequest| -> acme_platform::ports::RpcFuture<search::LintSearchIndexesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_search_indexes(request).await?.into_inner())
})
})
        };
        let signal_api_key_workflow_2_port = callback;
        Self {
            channel,
            repository,
            retry_api_key_workflow_1_port,
            retry_api_key_workflow_2_port,
            signal_api_key_workflow_2_port,
        }
    }
    /// Run StartApiKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_start_api_key_workflow(
        &self,
        req: edgegateway::StartApiKeyWorkflowRequest,
    ) -> Result<edgegateway::StartApiKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StartApiKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartApiKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.api_key.clone().unwrap_or_default().api_key_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CommentQuotaBucketApprovalRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    etag: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_by: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                funding: Some(ratelimiting::quota_bucket::Funding::Bank(
                    ratelimiting::QuotaBucketBankFunding {
                        bic: (match req.api_key.clone().unwrap_or_default().funding.clone() {
                            Some(edgegateway::api_key::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .bic
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = edgegateway::ScheduleGatewayPolicyReportRequest {
            context: req.context.clone(),
            gateway_policy_id: req
                .api_key
                .clone()
                .unwrap_or_default()
                .holder
                .clone()
                .unwrap_or_default()
                .gateway_policy_id
                .clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = QuotaBucketApprovalServiceClient::new(channel_0);
                let input: tonic::Request<ratelimiting::CommentQuotaBucketApprovalRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.comment_quota_bucket_approval(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = GatewayPolicyReportingServiceClient::new(channel_1);
                let input: tonic::Request<edgegateway::ScheduleGatewayPolicyReportRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.schedule_gateway_policy_report(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = edgegateway::StreamGatewayPolicyMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                preferred_channel: reply_0
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_2 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::StreamGatewayPolicyMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_gateway_policy_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::StreamGatewayPolicyMetricsResponse::default();
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
        let outgoing_3 = yards::BatchCreateYardsRequest {
            context: req.context.clone(),
            chunk: Some(yards::batch_create_yards_request::Chunk::Header(
                yards::batch_create_yards_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
                        name: req
                            .api_key
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = YardServiceClient::new(channel);
            let input: tonic::Request<yards::BatchCreateYardsRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_yards(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        created_by: reply_0
                            .quota_bucket
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
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
        let reply_4 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_4);
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
        let mut response = edgegateway::StartApiKeyWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.api_key.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run AdvanceApiKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_api_key_workflow(
        &self,
        req: edgegateway::AdvanceApiKeyWorkflowRequest,
    ) -> Result<edgegateway::AdvanceApiKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("AdvanceApiKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceApiKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.api_key.clone().unwrap_or_default().api_key_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::CountGatewayPolicyTotalsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::count_gateway_policy_totals_request::Filter {
                labels: req.api_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::ListPendingConfigChangesRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_pending_config_changes_request::Filter {
                created_after: req
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.api_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = GatewayPolicyReportingServiceClient::new(channel_0);
                let input: tonic::Request<edgegateway::CountGatewayPolicyTotalsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_gateway_policy_totals(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ConfigChangeApprovalServiceClient::new(channel_1);
                let input: tonic::Request<configuration::ListPendingConfigChangesRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_pending_config_changes(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = edgegateway::AdvanceApiKeyWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
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
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.api_key.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.api_key.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
    /// Run RetryApiKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_api_key_workflow(
        &self,
        req: edgegateway::RetryApiKeyWorkflowRequest,
    ) -> Result<edgegateway::RetryApiKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryApiKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryApiKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.api_key.clone().unwrap_or_default().api_key_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::StreamGatewayPolicyMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                name: req
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                phone: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: req
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::StreamGatewayPolicyMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_gateway_policy_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::StreamGatewayPolicyMetricsResponse::default();
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
        let outgoing_1 = secrets::SubmitSecretVersionRequest {
            context: req.context.clone(),
            details: Some(secrets::submit_secret_version_request::Details {
                approver: Some(identity::UserRef {
                    email: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                approver_email: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                ..Default::default()
            }),
            secret_version: Some(secrets::SecretVersion {
                subject: Some(secrets::AccessGrantRef {
                    display_name: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .version
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.retry_api_key_workflow_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = edgegateway::ScheduleGatewayPolicyReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::schedule_gateway_policy_report_request::Details {
                filters: req.api_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            gateway_policy_id: req
                .api_key
                .clone()
                .unwrap_or_default()
                .holder
                .clone()
                .unwrap_or_default()
                .gateway_policy_id
                .clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.retry_api_key_workflow_2_port.as_ref(), outgoing_2).await?;
        let mut response = edgegateway::RetryApiKeyWorkflowResponse::default();
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
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
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.api_key.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.api_key.get_or_insert_with(Default::default).contact = contribution;
        }
        Ok(response)
    }
    /// Run PauseApiKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_api_key_workflow(
        &self,
        req: edgegateway::PauseApiKeyWorkflowRequest,
    ) -> Result<edgegateway::PauseApiKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PauseApiKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseApiKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.api_key.clone().unwrap_or_default().api_key_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::CommentSecretVersionApprovalRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(secrets::comment_secret_version_approval_request::Details {
                approver: Some(identity::UserRef {
                    display_name: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret_version: Some(secrets::SecretVersion {
                subject: Some(secrets::AccessGrantRef {
                    version: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .version
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = edgegateway::ListGatewayPolicyReportsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_gateway_policy_reports_request::Filter {
                created_after: req
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.api_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = SecretVersionApprovalServiceClient::new(channel_0);
                let input: tonic::Request<secrets::CommentSecretVersionApprovalRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.comment_secret_version_approval(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = GatewayPolicyReportingServiceClient::new(channel_1);
                let input: tonic::Request<edgegateway::ListGatewayPolicyReportsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_gateway_policy_reports(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = edgegateway::FetchGatewayPolicyReportRequest {
            context: req.context.clone(),
            gateway_policy_id: req
                .api_key
                .clone()
                .unwrap_or_default()
                .holder
                .clone()
                .unwrap_or_default()
                .gateway_policy_id
                .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::FetchGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.fetch_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::PauseApiKeyWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_2
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_2
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .gateway_policy_id
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: String = req.api_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run InspectApiKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_api_key_workflow(
        &self,
        req: edgegateway::InspectApiKeyWorkflowRequest,
    ) -> Result<edgegateway::InspectApiKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectApiKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectApiKeyWorkflow", "running workflow");
        let outgoing_0 = edgegateway::ScheduleGatewayPolicyReportRequest {
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<edgegateway::ScheduleGatewayPolicyReportRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ScheduleGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = InspectApiKeyWorkflowDispatch1::from(req.clone());
        command_1.config_change_correlation_id = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        let outgoing_1: configuration::CancelConfigChangeWorkflowRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::CancelConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.cancel_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::InspectApiKeyWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
    /// Run ListApiKeyWorkflows after validating the tenant and durably recording the command.
    async fn run_list_api_key_workflows(
        &self,
        req: edgegateway::ListApiKeyWorkflowsRequest,
    ) -> Result<edgegateway::ListApiKeyWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListApiKeyWorkflows", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListApiKeyWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::StartConfigChangeWorkflowRequest {
            config_change: Some(configuration::ConfigChange {
                correlation_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                occurred_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::start_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
                parameters: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.StartConfigChangeWorkflowRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::StartConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.start_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::ListApiKeyWorkflowsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run SignalApiKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_api_key_workflow(
        &self,
        req: edgegateway::SignalApiKeyWorkflowRequest,
    ) -> Result<edgegateway::SignalApiKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SignalApiKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalApiKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.api_key.clone().unwrap_or_default().api_key_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::CountGatewayPolicyTotalsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::count_gateway_policy_totals_request::Filter {
                labels: req.api_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = featureflags::LintFlagRolloutsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .api_key
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = GatewayPolicyReportingServiceClient::new(channel_0);
                let input: tonic::Request<edgegateway::CountGatewayPolicyTotalsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_gateway_policy_totals(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = FlagRolloutValidationServiceClient::new(channel_1);
                let input: tonic::Request<featureflags::LintFlagRolloutsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.lint_flag_rollouts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = search::LintSearchIndexesRequest {
            context: req.context.clone(),
            search_index: Some(search::SearchIndex {
                locator: Some(search::search_index::Locator::Address(common::PostalAddress {
                    region: reply_1
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .region
                        .clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    etag: req
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let port = acme_platform::ports::opaque(self.signal_api_key_workflow_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let mut response = edgegateway::SignalApiKeyWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_2
            .search_index
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        let contribution: Option<edgegateway::GatewayPolicyRef> =
            req.api_key.clone().unwrap_or_default().holder.clone();
        if contribution != <Option<edgegateway::GatewayPolicyRef>>::default() {
            response.api_key.get_or_insert_with(Default::default).holder = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by InspectApiKeyWorkflow for CancelConfigChangeWorkflow.
struct InspectApiKeyWorkflowDispatch1 {
    context: Option<common::RequestContext>,
    config_change_correlation_id: String,
    details_notify_email: String,
}
impl From<edgegateway::InspectApiKeyWorkflowRequest> for InspectApiKeyWorkflowDispatch1 {
    fn from(req: edgegateway::InspectApiKeyWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_change_correlation_id: Default::default(),
            details_notify_email: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
        }
    }
}
impl From<InspectApiKeyWorkflowDispatch1> for configuration::CancelConfigChangeWorkflowRequest {
    fn from(value: InspectApiKeyWorkflowDispatch1) -> Self {
        configuration::CancelConfigChangeWorkflowRequest {
            config_change: Some(configuration::ConfigChange {
                correlation_id: value.config_change_correlation_id,
                ..Default::default()
            }),
            context: value.context,
            details: Some(configuration::cancel_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: value.details_notify_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ApiKeyWorkflowService for ApiKeyWorkflowApi {
    /// Starts an api key.
    async fn start_api_key_workflow(
        &self,
        request: tonic::Request<edgegateway::StartApiKeyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::StartApiKeyWorkflowResponse>, tonic::Status> {
        self.run_start_api_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances an api key.
    async fn advance_api_key_workflow(
        &self,
        request: tonic::Request<edgegateway::AdvanceApiKeyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::AdvanceApiKeyWorkflowResponse>, tonic::Status> {
        self.run_advance_api_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys an api key.
    async fn retry_api_key_workflow(
        &self,
        request: tonic::Request<edgegateway::RetryApiKeyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::RetryApiKeyWorkflowResponse>, tonic::Status> {
        self.run_retry_api_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses an api key.
    async fn pause_api_key_workflow(
        &self,
        request: tonic::Request<edgegateway::PauseApiKeyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::PauseApiKeyWorkflowResponse>, tonic::Status> {
        self.run_pause_api_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an api key.
    async fn inspect_api_key_workflow(
        &self,
        request: tonic::Request<edgegateway::InspectApiKeyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::InspectApiKeyWorkflowResponse>, tonic::Status> {
        self.run_inspect_api_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists api keys a page at a time.
    async fn list_api_key_workflows(
        &self,
        request: tonic::Request<edgegateway::ListApiKeyWorkflowsRequest>,
    ) -> Result<tonic::Response<edgegateway::ListApiKeyWorkflowsResponse>, tonic::Status> {
        self.run_list_api_key_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals an api key.
    async fn signal_api_key_workflow(
        &self,
        request: tonic::Request<edgegateway::SignalApiKeyWorkflowRequest>,
    ) -> Result<tonic::Response<edgegateway::SignalApiKeyWorkflowResponse>, tonic::Status> {
        self.run_signal_api_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
