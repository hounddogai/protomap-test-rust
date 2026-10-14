use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_client::CryptoKeyQueryServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_service_client::CryptoKeyServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_reporting_service_server::KeyRotationReportingService;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient;
use acme_proto::acme::warehousing::v1 as warehousing;
use acme_proto::acme::warehousing::v1::warehouse_query_service_client::WarehouseQueryServiceClient;

/// Builds and schedules reports about key rotations.
#[derive(Clone)]
pub struct KeyRotationReportingController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    schedule_key_rotation_report_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::ListQuotaBucketConflictsRequest,
            ratelimiting::ListQuotaBucketConflictsResponse,
        >,
    >,
    schedule_key_rotation_report_3_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::StartQuotaBucketWorkflowRequest,
                ratelimiting::StartQuotaBucketWorkflowResponse,
            >,
    >,
    share_key_rotation_report_0_port:
        acme_platform::ports::Callback<keymanagement::ScoreCryptoKeyRequest, keymanagement::ScoreCryptoKeyResponse>,
    share_key_rotation_report_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::RecalculateFlagEvaluationsRequest,
            featureflags::RecalculateFlagEvaluationsResponse,
        >,
    >,
}
impl KeyRotationReportingController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::ListQuotaBucketConflictsRequest,
            ratelimiting::ListQuotaBucketConflictsResponse,
        > = {
            let client = QuotaBucketSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ListQuotaBucketConflictsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ListQuotaBucketConflictsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_quota_bucket_conflicts(request).await?.into_inner())
})
})
        };
        let schedule_key_rotation_report_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::StartQuotaBucketWorkflowRequest,
            ratelimiting::StartQuotaBucketWorkflowResponse,
        > = {
            let client = QuotaBucketWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::StartQuotaBucketWorkflowRequest| -> acme_platform::ports::RpcFuture<ratelimiting::StartQuotaBucketWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.start_quota_bucket_workflow(request).await?.into_inner())
})
})
        };
        let schedule_key_rotation_report_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::ScoreCryptoKeyRequest,
            keymanagement::ScoreCryptoKeyResponse,
        > = {
            let client = CryptoKeyValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ScoreCryptoKeyRequest| -> acme_platform::ports::RpcFuture<keymanagement::ScoreCryptoKeyResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_crypto_key(request).await?.into_inner())
})
})
        };
        let share_key_rotation_report_0_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::RecalculateFlagEvaluationsRequest,
            featureflags::RecalculateFlagEvaluationsResponse,
        > = {
            let client = FlagEvaluationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::RecalculateFlagEvaluationsRequest| -> acme_platform::ports::RpcFuture<featureflags::RecalculateFlagEvaluationsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_flag_evaluations(request).await?.into_inner())
})
})
        };
        let share_key_rotation_report_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            schedule_key_rotation_report_2_port,
            schedule_key_rotation_report_3_port,
            share_key_rotation_report_0_port,
            share_key_rotation_report_1_port,
        }
    }
    /// Run GenerateKeyRotationReport after validating the tenant and durably recording the command.
    async fn run_generate_key_rotation_report(
        &self,
        req: keymanagement::GenerateKeyRotationReportRequest,
    ) -> Result<keymanagement::GenerateKeyRotationReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GenerateKeyRotationReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateKeyRotationReport", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ListCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::list_crypto_keys_request::Filter {
                labels: req.annotations.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.keymanagement.v1.ListCryptoKeysRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ListCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::ExplainCryptoKeyDecisionRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(keymanagement::explain_crypto_key_decision_request::Details {
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::ExplainCryptoKeyDecisionRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CryptoKeyValidationServiceClient::new(channel);
                let input: tonic::Request<keymanagement::ExplainCryptoKeyDecisionRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_crypto_key_decision(input).await?;
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
        let mut response = keymanagement::GenerateKeyRotationReportResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run FetchKeyRotationReport after validating the tenant and durably recording the command.
    async fn run_fetch_key_rotation_report(
        &self,
        req: keymanagement::FetchKeyRotationReportRequest,
    ) -> Result<keymanagement::FetchKeyRotationReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FetchKeyRotationReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchKeyRotationReport", "running workflow");
        let mut response = keymanagement::FetchKeyRotationReportResponse::default();
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleKeyRotationReport after validating the tenant and durably recording the command.
    async fn run_schedule_key_rotation_report(
        &self,
        req: keymanagement::ScheduleKeyRotationReportRequest,
    ) -> Result<keymanagement::ScheduleKeyRotationReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleKeyRotationReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleKeyRotationReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CancelQuotaBucketWorkflowRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(ratelimiting::cancel_quota_bucket_workflow_request::Details {
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
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                funding: Some(ratelimiting::quota_bucket::Funding::ExternalAccountId(
                    req.key_rotation_id.clone(),
                )),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CancelQuotaBucketWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.cancel_quota_bucket_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::UnsubscribeFeatureFlagUpdatesRequest {
            context: req.context.clone(),
            details: Some(featureflags::unsubscribe_feature_flag_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    phone: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FeatureFlagNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::UnsubscribeFeatureFlagUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.unsubscribe_feature_flag_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ListQuotaBucketConflictsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_quota_bucket_conflicts_request::Filter {
                created_after: reply_1
                    .feature_flag
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_0.quota_bucket.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 =
            acme_platform::ports::execute(self.schedule_key_rotation_report_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = ratelimiting::StartQuotaBucketWorkflowRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: reply_1
                        .feature_flag
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
            }),
            ..Default::default()
        };
        let reply_3 =
            acme_platform::ports::execute(self.schedule_key_rotation_report_3_port.as_ref(), outgoing_3).await?;
        let mut response = keymanagement::ScheduleKeyRotationReportResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run StreamKeyRotationMetrics after validating the tenant and durably recording the command.
    async fn run_stream_key_rotation_metrics(
        &self,
        req: keymanagement::StreamKeyRotationMetricsRequest,
    ) -> Result<keymanagement::StreamKeyRotationMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamKeyRotationMetrics", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamKeyRotationMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().owner_id.clone(), "recorded audit field");
        let command_0 = StreamKeyRotationMetricsDispatch0::from(req.clone());
        let outgoing_0: warehousing::SuggestWarehousesRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = WarehouseQueryServiceClient::new(channel);
            let input: tonic::Request<warehousing::SuggestWarehousesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_warehouses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::GenerateEnvironmentReportRequest {
            annotations: reply_0.result.clone().unwrap_or_default().details.clone(),
            context: req.context.clone(),
            details: Some(configuration::generate_environment_report_request::Details {
                filters: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.configuration.v1.GenerateEnvironmentReportRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::GenerateEnvironmentReportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_environment_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::StreamKeyRotationMetricsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListKeyRotationReports after validating the tenant and durably recording the command.
    async fn run_list_key_rotation_reports(
        &self,
        req: keymanagement::ListKeyRotationReportsRequest,
    ) -> Result<keymanagement::ListKeyRotationReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListKeyRotationReports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListKeyRotationReports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::SyncQuotaBucketsRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SyncQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.sync_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = keymanagement::ListKeyRotationReportsResponse::default();
        let contribution: Option<common::ContactInfo> =
            reply_0.quota_bucket.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            if response.key_rotations.is_empty() {
                response.key_rotations.push(Default::default());
            }
            if response.key_rotations[0].participants.is_empty() {
                response.key_rotations[0].participants.push(Default::default());
            }
            response.key_rotations[0].participants[0].contact = contribution;
        }
        Ok(response)
    }
    /// Run ShareKeyRotationReport after validating the tenant and durably recording the command.
    async fn run_share_key_rotation_report(
        &self,
        req: keymanagement::ShareKeyRotationReportRequest,
    ) -> Result<keymanagement::ShareKeyRotationReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareKeyRotationReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareKeyRotationReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ScoreCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.share_key_rotation_report_0_port)(outgoing_0).await?;
        let outgoing_1 = featureflags::RecalculateFlagEvaluationsRequest {
            context: req.context.clone(),
            details: Some(featureflags::recalculate_flag_evaluations_request::Details {
                currency_code: reply_0.crypto_key.clone().unwrap_or_default().currency_code.clone(),
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            self.share_key_rotation_report_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = keymanagement::SuggestCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::suggest_crypto_keys_request::Filter {
                labels: (reply_0.crypto_key.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CryptoKeyQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SuggestCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = keymanagement::CountCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::count_crypto_keys_request::Filter {
                created_after: reply_0
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: reply_0.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_1
                    .flag_evaluation
                    .clone()
                    .unwrap_or_default()
                    .flag_evaluation_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_3 = acme_platform::ports::RequestBuilder::<keymanagement::CountCryptoKeysRequest>::default()
            .map(|draft| *draft = outgoing_3)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = CryptoKeyQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::CountCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ShareKeyRotationReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by StreamKeyRotationMetrics for SuggestWarehouses.
struct StreamKeyRotationMetricsDispatch0 {
    context: Option<common::RequestContext>,
    filter_query: String,
}
impl From<keymanagement::StreamKeyRotationMetricsRequest> for StreamKeyRotationMetricsDispatch0 {
    fn from(req: keymanagement::StreamKeyRotationMetricsRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_query: req.filter.clone().unwrap_or_default().query.clone(),
        }
    }
}
impl From<StreamKeyRotationMetricsDispatch0> for warehousing::SuggestWarehousesRequest {
    fn from(value: StreamKeyRotationMetricsDispatch0) -> Self {
        warehousing::SuggestWarehousesRequest {
            context: value.context,
            filter: Some(warehousing::suggest_warehouses_request::Filter {
                query: value.filter_query,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRotationReportingService for KeyRotationReportingController {
    /// Generates a key rotation.
    async fn generate_key_rotation_report(
        &self,
        request: tonic::Request<keymanagement::GenerateKeyRotationReportRequest>,
    ) -> Result<tonic::Response<keymanagement::GenerateKeyRotationReportResponse>, tonic::Status> {
        self.run_generate_key_rotation_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key rotation.
    async fn fetch_key_rotation_report(
        &self,
        request: tonic::Request<keymanagement::FetchKeyRotationReportRequest>,
    ) -> Result<tonic::Response<keymanagement::FetchKeyRotationReportResponse>, tonic::Status> {
        self.run_fetch_key_rotation_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a key rotation.
    async fn schedule_key_rotation_report(
        &self,
        request: tonic::Request<keymanagement::ScheduleKeyRotationReportRequest>,
    ) -> Result<tonic::Response<keymanagement::ScheduleKeyRotationReportResponse>, tonic::Status> {
        self.run_schedule_key_rotation_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamKeyRotationMetricsStream =
        tokio_stream::wrappers::ReceiverStream<Result<keymanagement::StreamKeyRotationMetricsResponse, tonic::Status>>;
    /// Streams the changes of key rotations as they happen.
    async fn stream_key_rotation_metrics(
        &self,
        request: tonic::Request<keymanagement::StreamKeyRotationMetricsRequest>,
    ) -> Result<tonic::Response<Self::StreamKeyRotationMetricsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_key_rotation_metrics(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Lists key rotations a page at a time.
    async fn list_key_rotation_reports(
        &self,
        request: tonic::Request<keymanagement::ListKeyRotationReportsRequest>,
    ) -> Result<tonic::Response<keymanagement::ListKeyRotationReportsResponse>, tonic::Status> {
        self.run_list_key_rotation_reports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Shares a key rotation.
    async fn share_key_rotation_report(
        &self,
        request: tonic::Request<keymanagement::ShareKeyRotationReportRequest>,
    ) -> Result<tonic::Response<keymanagement::ShareKeyRotationReportResponse>, tonic::Status> {
        self.run_share_key_rotation_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
