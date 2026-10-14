use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_notification_service_client::PermissionCheckNotificationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::concessions::v1::concession_stand_service_client::ConcessionStandServiceClient;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::labeling::v1::labeling_task_service_client::LabelingTaskServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_approval_service_client::RateLimitApprovalServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_client::ThrottleCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_workflow_service_server::ThrottleWorkflowService;

/// Runs the long-lived workflow that takes a throttle from start to finish.
#[derive(Clone)]
pub struct ThrottleWorkflowHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
}
impl ThrottleWorkflowHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StartThrottleWorkflow after validating the tenant and durably recording the command.
    async fn run_start_throttle_workflow(
        &self,
        req: ratelimiting::StartThrottleWorkflowRequest,
    ) -> Result<ratelimiting::StartThrottleWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StartThrottleWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartThrottleWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let jobs_0 = req
            .throttle
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = configuration::CountEnvironmentTotalsRequest {
                        context: req.context.clone(),
                        filter: Some(configuration::count_environment_totals_request::Filter {
                            created_after: item.observed_at.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = EnvironmentReportingServiceClient::new(channel);
                        let input: tonic::Request<configuration::CountEnvironmentTotalsRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.count_environment_totals(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = ratelimiting::PushThrottleRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                kind: req.throttle.clone().unwrap_or_default().kind.clone(),
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    rate_limit_id: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .rate_limit_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushThrottleRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::PauseQuotaBucketWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::pause_quota_bucket_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(ratelimiting::RateLimitRef {
                    version: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PauseQuotaBucketWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pause_quota_bucket_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::StartThrottleWorkflowResponse::default();
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .rate_limit_id
            .clone();
        if contribution != <String>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .rate_limit_id = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: String = req.throttle.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceThrottleWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_throttle_workflow(
        &self,
        req: ratelimiting::AdvanceThrottleWorkflowRequest,
    ) -> Result<ratelimiting::AdvanceThrottleWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceThrottleWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceThrottleWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = labeling::BatchGetLabelingTasksRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = LabelingTaskServiceClient::new(channel);
            let input: tonic::Request<labeling::BatchGetLabelingTasksRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_labeling_tasks(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::EstimateThrottleRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                kind: req.throttle.clone().unwrap_or_default().kind.clone(),
                subject: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .rate_limit_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateThrottleRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::AdvanceThrottleWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: Option<ratelimiting::RateLimitRef> = req.throttle.clone().unwrap_or_default().subject.clone();
        if contribution != <Option<ratelimiting::RateLimitRef>>::default() {
            response.throttle.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run CancelThrottleWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_throttle_workflow(
        &self,
        req: ratelimiting::CancelThrottleWorkflowRequest,
    ) -> Result<ratelimiting::CancelThrottleWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CancelThrottleWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelThrottleWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ListThrottleConflictsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_throttle_conflicts_request::Filter {
                labels: req.details.clone().unwrap_or_default().parameters.clone(),
                owner_id: req
                    .throttle
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .rate_limit_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListThrottleConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_throttle_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::GetRateLimitApprovalStatusRequest {
            context: req.context.clone(),
            rate_limit_id: req
                .throttle
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .rate_limit_id
                .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitApprovalServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::GetRateLimitApprovalStatusRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_rate_limit_approval_status(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::StreamQuotaBucketChangesRequest {
            frame: Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(
                ratelimiting::QuotaBucket {
                    audit: Some(common::AuditInfo {
                        updated_by: reply_1
                            .rate_limit
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_by
                            .clone(),
                        ..Default::default()
                    }),
                    contact: Some(common::ContactInfo {
                        name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify
                            .clone()
                            .unwrap_or_default()
                            .name
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
        let _reply_2 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
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
        let mut response = ratelimiting::CancelThrottleWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.throttle.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run RetryThrottleWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_throttle_workflow(
        &self,
        req: ratelimiting::RetryThrottleWorkflowRequest,
    ) -> Result<ratelimiting::RetryThrottleWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryThrottleWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryThrottleWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().kind.clone(), "recorded audit field");
        let jobs_0 = req
            .throttle
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = featureflags::ListFlagEvaluationVersionsRequest {
                        context: req.context.clone(),
                        filter: Some(featureflags::list_flag_evaluation_versions_request::Filter {
                            created_after: item.observed_at.clone(),
                            labels: req.throttle.clone().unwrap_or_default().attributes.clone(),
                            owner_id: req
                                .throttle
                                .clone()
                                .unwrap_or_default()
                                .subject
                                .clone()
                                .unwrap_or_default()
                                .rate_limit_id
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = FlagEvaluationHistoryServiceClient::new(channel);
                        let input: tonic::Request<featureflags::ListFlagEvaluationVersionsRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.list_flag_evaluation_versions(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = concessions::SearchConcessionStandsRequest {
            context: req.context.clone(),
            filter: Some(concessions::search_concession_stands_request::Filter {
                created_after: req.throttle.clone().unwrap_or_default().occurred_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req.throttle.clone().unwrap_or_default().throttle_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConcessionStandServiceClient::new(channel);
            let input: tonic::Request<concessions::SearchConcessionStandsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_concession_stands(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::SignalRateLimitWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::signal_rate_limit_workflow_request::Details {
                parameters: req.details.clone().unwrap_or_default().parameters.clone(),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
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
                rate_limit_id: req
                    .throttle
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .rate_limit_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SignalRateLimitWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_rate_limit_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::RetryThrottleWorkflowResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
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
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.throttle.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.throttle.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run ResumeThrottleWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_throttle_workflow(
        &self,
        req: ratelimiting::ResumeThrottleWorkflowRequest,
    ) -> Result<ratelimiting::ResumeThrottleWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResumeThrottleWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeThrottleWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = concessions::UpdateConcessionStandRequest {
            concession_stand: Some(concessions::ConcessionStand {
                owner: Some(identity::UserRef {
                    display_name: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
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
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConcessionStandServiceClient::new(channel);
            let input: tonic::Request<concessions::UpdateConcessionStandRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_concession_stand(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ScheduleConfigChangeReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ScheduleConfigChangeReportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_config_change_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ResumeThrottleWorkflowResponse::default();
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
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: String = req.throttle.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run InspectThrottleWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_throttle_workflow(
        &self,
        req: ratelimiting::InspectThrottleWorkflowRequest,
    ) -> Result<ratelimiting::InspectThrottleWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectThrottleWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectThrottleWorkflow", "running workflow");
        let outgoing_0 = authorization::SendPermissionCheckTestNoticeRequest {
            context: req.context.clone(),
            details: Some(authorization::send_permission_check_test_notice_request::Details {
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
                variables: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = PermissionCheckNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::SendPermissionCheckTestNoticeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.send_permission_check_test_notice(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::InspectThrottleWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.permission_check.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).source = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
    /// Run ListThrottleWorkflows after validating the tenant and durably recording the command.
    async fn run_list_throttle_workflows(
        &self,
        req: ratelimiting::ListThrottleWorkflowsRequest,
    ) -> Result<ratelimiting::ListThrottleWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListThrottleWorkflows", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListThrottleWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = ratelimiting::ListThrottleWorkflowsResponse::default();
        Ok(response)
    }
    /// Run SignalThrottleWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_throttle_workflow(
        &self,
        req: ratelimiting::SignalThrottleWorkflowRequest,
    ) -> Result<ratelimiting::SignalThrottleWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SignalThrottleWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalThrottleWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.throttle.clone().unwrap_or_default().throttle_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::SyncThrottlesRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                sequence: req.throttle.clone().unwrap_or_default().sequence.clone(),
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    rate_limit_id: req
                        .throttle
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .rate_limit_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SyncThrottlesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.sync_throttles(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::SignalThrottleWorkflowResponse::default();
        let contribution: String = reply_0.throttle.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).kind = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.throttle.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
}
acme_platform::endpoints! { ThrottleWorkflowService, ThrottleWorkflowHandler;
unary {
start_throttle_workflow (ratelimiting::StartThrottleWorkflowRequest) -> ratelimiting::StartThrottleWorkflowResponse => run_start_throttle_workflow;
advance_throttle_workflow (ratelimiting::AdvanceThrottleWorkflowRequest) -> ratelimiting::AdvanceThrottleWorkflowResponse => run_advance_throttle_workflow;
cancel_throttle_workflow (ratelimiting::CancelThrottleWorkflowRequest) -> ratelimiting::CancelThrottleWorkflowResponse => run_cancel_throttle_workflow;
retry_throttle_workflow (ratelimiting::RetryThrottleWorkflowRequest) -> ratelimiting::RetryThrottleWorkflowResponse => run_retry_throttle_workflow;
resume_throttle_workflow (ratelimiting::ResumeThrottleWorkflowRequest) -> ratelimiting::ResumeThrottleWorkflowResponse => run_resume_throttle_workflow;
inspect_throttle_workflow (ratelimiting::InspectThrottleWorkflowRequest) -> ratelimiting::InspectThrottleWorkflowResponse => run_inspect_throttle_workflow;
list_throttle_workflows (ratelimiting::ListThrottleWorkflowsRequest) -> ratelimiting::ListThrottleWorkflowsResponse => run_list_throttle_workflows;
signal_throttle_workflow (ratelimiting::SignalThrottleWorkflowRequest) -> ratelimiting::SignalThrottleWorkflowResponse => run_signal_throttle_workflow;
}
server {
}
client {
}
bidi {
}
}
