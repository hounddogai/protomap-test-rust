use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_service_client::DocumentServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::gateway_policy_reporting_service_server::GatewayPolicyReportingService;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::fanportal::v1::fan_session_query_service_client::FanSessionQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_export_service_client::FeatureFlagExportServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_service_client::UserServiceClient;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient;
use acme_proto::acme::media::v1 as media;
use acme_proto::acme::media::v1::media_asset_service_client::MediaAssetServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_approval_service_client::ThrottleApprovalServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient;

/// Builds and schedules reports about gateway policies.
#[derive(Clone)]
pub struct GatewayPolicyReportingApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    count_gateway_policy_totals_0_port:
        acme_platform::ports::Callback<media::ListMediaAssetsRequest, media::ListMediaAssetsResponse>,
    generate_gateway_policy_report_3_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::ListFeatureFlagExportsRequest,
            featureflags::ListFeatureFlagExportsResponse,
        >,
    >,
    schedule_gateway_policy_report_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::CheckFlagRolloutEligibilityRequest,
            featureflags::CheckFlagRolloutEligibilityResponse,
        >,
    >,
    schedule_gateway_policy_report_1_port: acme_platform::ports::Callback<
        ratelimiting::GenerateQuotaBucketReportRequest,
        ratelimiting::GenerateQuotaBucketReportResponse,
    >,
}
impl GatewayPolicyReportingApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::ListFeatureFlagExportsRequest,
            featureflags::ListFeatureFlagExportsResponse,
        > = {
            let client = FeatureFlagExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ListFeatureFlagExportsRequest| -> acme_platform::ports::RpcFuture<featureflags::ListFeatureFlagExportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_feature_flag_exports(request).await?.into_inner())
})
})
        };
        let generate_gateway_policy_report_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::CheckFlagRolloutEligibilityRequest,
            featureflags::CheckFlagRolloutEligibilityResponse,
        > = {
            let client = FlagRolloutValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::CheckFlagRolloutEligibilityRequest| -> acme_platform::ports::RpcFuture<featureflags::CheckFlagRolloutEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_flag_rollout_eligibility(request).await?.into_inner())
})
})
        };
        let schedule_gateway_policy_report_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::GenerateQuotaBucketReportRequest,
            ratelimiting::GenerateQuotaBucketReportResponse,
        > = {
            let client = QuotaBucketReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::GenerateQuotaBucketReportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::GenerateQuotaBucketReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.generate_quota_bucket_report(request).await?.into_inner())
})
})
        };
        let schedule_gateway_policy_report_1_port = callback;
        let callback: acme_platform::ports::Callback<media::ListMediaAssetsRequest, media::ListMediaAssetsResponse> = {
            let client = MediaAssetServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: media::ListMediaAssetsRequest| -> acme_platform::ports::RpcFuture<media::ListMediaAssetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_media_assets(request).await?.into_inner())
})
})
        };
        let count_gateway_policy_totals_0_port = callback;
        Self {
            channel,
            repository,
            count_gateway_policy_totals_0_port,
            generate_gateway_policy_report_3_port,
            schedule_gateway_policy_report_0_port,
            schedule_gateway_policy_report_1_port,
        }
    }
    /// Run GenerateGatewayPolicyReport after validating the tenant and durably recording the command.
    async fn run_generate_gateway_policy_report(
        &self,
        req: edgegateway::GenerateGatewayPolicyReportRequest,
    ) -> Result<edgegateway::GenerateGatewayPolicyReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GenerateGatewayPolicyReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateGatewayPolicyReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamQuotaBucketMetricsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::stream_quota_bucket_metrics_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.details.clone().unwrap_or_default().filters.clone(),
                owner_id: req.gateway_policy_id.clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
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
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_quota_bucket_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamQuotaBucketMetricsResponse::default();
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
        let outgoing_1 = identity::BatchGetUsersRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::BatchGetUsersRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_users(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::CountQuotaBucketTotalsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::count_quota_bucket_totals_request::Filter {
                labels: reply_1.result.clone().unwrap_or_default().details.clone(),
                owner_id: req.gateway_policy_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CountQuotaBucketTotalsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_quota_bucket_totals(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::ListFeatureFlagExportsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_exports_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: reply_2.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_2.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_3 =
            acme_platform::ports::execute(self.generate_gateway_policy_report_3_port.as_ref(), outgoing_3).await?;
        let outgoing_4 = ratelimiting::BatchCreateRateLimitsRequest {
            context: req.context.clone(),
            chunk: Some(ratelimiting::batch_create_rate_limits_request::Chunk::Record(
                ratelimiting::RateLimit {
                    action: Some(ratelimiting::rate_limit::Action::Block(
                        ratelimiting::RateLimitBlockAction {
                            reason: req.reason.clone(),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = RateLimitServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::BatchCreateRateLimitsRequest> =
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
            let response = client.batch_create_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = ratelimiting::StreamQuotaBucketChangesRequest {
            frame: Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Ack(
                ratelimiting::stream_quota_bucket_changes_request::ChangeAck {
                    status: reply_3.result.clone().unwrap_or_default().status.clone(),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_5 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_5);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_quota_bucket_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamQuotaBucketChangesResponse::default();
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
        let mut response = edgegateway::GenerateGatewayPolicyReportResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_3.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run FetchGatewayPolicyReport after validating the tenant and durably recording the command.
    async fn run_fetch_gateway_policy_report(
        &self,
        req: edgegateway::FetchGatewayPolicyReportRequest,
    ) -> Result<edgegateway::FetchGatewayPolicyReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("FetchGatewayPolicyReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchGatewayPolicyReport", "running workflow");
        let outgoing_0 = identity::WatchUsersRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
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
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::WatchUsersRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_users(input).await?;
            let mut frames = response.into_inner();
            let mut last = identity::WatchUsersResponse::default();
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
        let mut response = edgegateway::FetchGatewayPolicyReportResponse::default();
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleGatewayPolicyReport after validating the tenant and durably recording the command.
    async fn run_schedule_gateway_policy_report(
        &self,
        req: edgegateway::ScheduleGatewayPolicyReportRequest,
    ) -> Result<edgegateway::ScheduleGatewayPolicyReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleGatewayPolicyReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleGatewayPolicyReport", "running workflow");
        tracing::debug!(audit_value = ?req.gateway_policy_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CheckFlagRolloutEligibilityRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
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
                participants: vec![featureflags::FlagRolloutParticipant {
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
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.schedule_gateway_policy_report_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = ratelimiting::GenerateQuotaBucketReportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::generate_quota_bucket_report_request::Details {
                cron: req.details.clone().unwrap_or_default().cron.clone(),
                format: req.details.clone().unwrap_or_default().format.clone(),
                recipient_emails: req.details.clone().unwrap_or_default().recipient_emails.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.schedule_gateway_policy_report_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let outgoing_2 = secrets::RetrySecretVersionWorkflowRequest {
            context: req.context.clone(),
            secret_version: Some(secrets::SecretVersion {
                correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                subject: Some(secrets::AccessGrantRef {
                    display_name: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .organizer
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<secrets::RetrySecretVersionWorkflowRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = SecretVersionWorkflowServiceClient::new(channel);
            let input: tonic::Request<secrets::RetrySecretVersionWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.retry_secret_version_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_3 = ScheduleGatewayPolicyReportDispatch3::from(req.clone());
        command_3.key_rotation_location_postal_code = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        command_3.key_rotation_weekly_interval =
            (match reply_0.flag_rollout.clone().unwrap_or_default().recurrence.clone() {
                Some(featureflags::flag_rollout::Recurrence::Weekly(value)) => value,
                _ => Default::default(),
            })
            .interval
            .clone();
        command_3.key_rotation_location_locality = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .locality
            .clone();
        let outgoing_3: keymanagement::StartKeyRotationWorkflowRequest = command_3.into();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = KeyRotationWorkflowServiceClient::new(channel);
            let input: tonic::Request<keymanagement::StartKeyRotationWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.start_key_rotation_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::ScheduleGatewayPolicyReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2
            .secret_version
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run StreamGatewayPolicyMetrics after validating the tenant and durably recording the command.
    async fn run_stream_gateway_policy_metrics(
        &self,
        req: edgegateway::StreamGatewayPolicyMetricsRequest,
    ) -> Result<edgegateway::StreamGatewayPolicyMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamGatewayPolicyMetrics", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamGatewayPolicyMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.subscriber.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = configuration::AdvanceConfigChangeWorkflowRequest {
            context: req.context.clone(),
            details: Some(configuration::advance_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.AdvanceConfigChangeWorkflowRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::AdvanceConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = documents::BatchGetDocumentsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<documents::BatchGetDocumentsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = DocumentServiceClient::new(channel);
                let input: tonic::Request<documents::BatchGetDocumentsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.batch_get_documents(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_1)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_1 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let outgoing_2 = identity::WatchUsersRequest {
            context: req.context.clone(),
            filter: Some(identity::watch_users_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_2 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::WatchUsersRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_users(input).await?;
            let mut frames = response.into_inner();
            let mut last = identity::WatchUsersResponse::default();
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
        let outgoing_3 = identity::SearchUsersRequest {
            context: req.context.clone(),
            filter: Some(identity::search_users_request::Filter {
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::SearchUsersRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_users(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::StreamGatewayPolicyMetricsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListGatewayPolicyReports after validating the tenant and durably recording the command.
    async fn run_list_gateway_policy_reports(
        &self,
        req: edgegateway::ListGatewayPolicyReportsRequest,
    ) -> Result<edgegateway::ListGatewayPolicyReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListGatewayPolicyReports", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListGatewayPolicyReports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = edgegateway::ListGatewayPolicyReportsResponse::default();
        Ok(response)
    }
    /// Run CountGatewayPolicyTotals after validating the tenant and durably recording the command.
    async fn run_count_gateway_policy_totals(
        &self,
        req: edgegateway::CountGatewayPolicyTotalsRequest,
    ) -> Result<edgegateway::CountGatewayPolicyTotalsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CountGatewayPolicyTotals", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountGatewayPolicyTotals", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = media::ListMediaAssetsRequest {
            context: req.context.clone(),
            filter: Some(media::list_media_assets_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.count_gateway_policy_totals_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = edgegateway::CountGatewayPolicyTotalsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run DeleteGatewayPolicyReport after validating the tenant and durably recording the command.
    async fn run_delete_gateway_policy_report(
        &self,
        req: edgegateway::DeleteGatewayPolicyReportRequest,
    ) -> Result<edgegateway::DeleteGatewayPolicyReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DeleteGatewayPolicyReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteGatewayPolicyReport", "running workflow");
        let outgoing_0 = fanportal::BrowseFanSessionsRequest {
            context: req.context.clone(),
            filter: Some(fanportal::browse_fan_sessions_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                owner_id: req.gateway_policy_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<fanportal::BrowseFanSessionsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FanSessionQueryServiceClient::new(channel);
            let input: tonic::Request<fanportal::BrowseFanSessionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_fan_sessions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ReconcileRateLimitsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::reconcile_rate_limits_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    etag: req.etag.clone(),
                    ..Default::default()
                }),
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ReconcileRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::EscalateThrottleRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::escalate_throttle_request::Details {
                approver: Some(identity::UserRef {
                    email: reply_1
                        .rate_limit
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
            throttle: Some(ratelimiting::Throttle {
                correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 =
            acme_platform::codec::through_any(&outgoing_2, "acme.ratelimiting.v1.EscalateThrottleRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ThrottleApprovalServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EscalateThrottleRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.escalate_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::DeleteGatewayPolicyReportResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.effective_at = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
}
/// The domain command prepared by ScheduleGatewayPolicyReport for StartKeyRotationWorkflow.
struct ScheduleGatewayPolicyReportDispatch3 {
    context: Option<common::RequestContext>,
    key_rotation_location_postal_code: String,
    key_rotation_weekly_interval: i32,
    key_rotation_location_locality: String,
}
impl From<edgegateway::ScheduleGatewayPolicyReportRequest> for ScheduleGatewayPolicyReportDispatch3 {
    fn from(req: edgegateway::ScheduleGatewayPolicyReportRequest) -> Self {
        Self {
            context: req.context.clone(),
            key_rotation_location_postal_code: Default::default(),
            key_rotation_weekly_interval: Default::default(),
            key_rotation_location_locality: Default::default(),
        }
    }
}
impl From<ScheduleGatewayPolicyReportDispatch3> for keymanagement::StartKeyRotationWorkflowRequest {
    fn from(value: ScheduleGatewayPolicyReportDispatch3) -> Self {
        keymanagement::StartKeyRotationWorkflowRequest {
            context: value.context,
            key_rotation: Some(keymanagement::KeyRotation {
                location: Some(common::PostalAddress {
                    locality: value.key_rotation_location_locality,
                    postal_code: value.key_rotation_location_postal_code,
                    ..Default::default()
                }),
                recurrence: Some(keymanagement::key_rotation::Recurrence::Weekly(
                    keymanagement::KeyRotationWeeklyRule {
                        interval: value.key_rotation_weekly_interval,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl GatewayPolicyReportingService for GatewayPolicyReportingApi {
    /// Generates a gateway policy.
    async fn generate_gateway_policy_report(
        &self,
        request: tonic::Request<edgegateway::GenerateGatewayPolicyReportRequest>,
    ) -> Result<tonic::Response<edgegateway::GenerateGatewayPolicyReportResponse>, tonic::Status> {
        self.run_generate_gateway_policy_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a gateway policy.
    async fn fetch_gateway_policy_report(
        &self,
        request: tonic::Request<edgegateway::FetchGatewayPolicyReportRequest>,
    ) -> Result<tonic::Response<edgegateway::FetchGatewayPolicyReportResponse>, tonic::Status> {
        self.run_fetch_gateway_policy_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a gateway policy.
    async fn schedule_gateway_policy_report(
        &self,
        request: tonic::Request<edgegateway::ScheduleGatewayPolicyReportRequest>,
    ) -> Result<tonic::Response<edgegateway::ScheduleGatewayPolicyReportResponse>, tonic::Status> {
        self.run_schedule_gateway_policy_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamGatewayPolicyMetricsStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::StreamGatewayPolicyMetricsResponse, tonic::Status>>;
    /// Streams the changes of gateway policies as they happen.
    async fn stream_gateway_policy_metrics(
        &self,
        request: tonic::Request<edgegateway::StreamGatewayPolicyMetricsRequest>,
    ) -> Result<tonic::Response<Self::StreamGatewayPolicyMetricsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(
                async move { handler.run_stream_gateway_policy_metrics(request).await },
            );
        Ok(tonic::Response::new(stream))
    }
    /// Lists gateway policies a page at a time.
    async fn list_gateway_policy_reports(
        &self,
        request: tonic::Request<edgegateway::ListGatewayPolicyReportsRequest>,
    ) -> Result<tonic::Response<edgegateway::ListGatewayPolicyReportsResponse>, tonic::Status> {
        self.run_list_gateway_policy_reports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts gateway policies by status.
    async fn count_gateway_policy_totals(
        &self,
        request: tonic::Request<edgegateway::CountGatewayPolicyTotalsRequest>,
    ) -> Result<tonic::Response<edgegateway::CountGatewayPolicyTotalsResponse>, tonic::Status> {
        self.run_count_gateway_policy_totals(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a gateway policy.
    async fn delete_gateway_policy_report(
        &self,
        request: tonic::Request<edgegateway::DeleteGatewayPolicyReportRequest>,
    ) -> Result<tonic::Response<edgegateway::DeleteGatewayPolicyReportResponse>, tonic::Status> {
        self.run_delete_gateway_policy_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
