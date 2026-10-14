use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::concessions::v1::concession_stand_service_client::ConcessionStandServiceClient;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_workflow_service_client::EnvironmentWorkflowServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::labeling::v1::labeling_task_query_service_client::LabelingTaskQueryServiceClient;
use acme_proto::acme::pricing::v1 as pricing;
use acme_proto::acme::pricing::v1::price_list_service_client::PriceListServiceClient;
use acme_proto::acme::procurement::v1 as procurement;
use acme_proto::acme::procurement::v1::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_server::QuotaBucketReportingService;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_client::ThrottleCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_client::ThrottleImportServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::yards::v1 as yards;
use acme_proto::acme::yards::v1::yard_service_client::YardServiceClient;

/// Builds and schedules reports about quota buckets.
#[derive(Clone)]
pub struct QuotaBucketReportingHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
}
impl QuotaBucketReportingHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run GenerateQuotaBucketReport after validating the tenant and durably recording the command.
    async fn run_generate_quota_bucket_report(
        &self,
        req: ratelimiting::GenerateQuotaBucketReportRequest,
    ) -> Result<ratelimiting::GenerateQuotaBucketReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GenerateQuotaBucketReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateQuotaBucketReport", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::EstimateRateLimitRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        notify_email: req
                            .context
                            .clone()
                            .unwrap_or_default()
                            .actor
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
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
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::DelegateConfigChangeApprovalRequest {
            context: req.context.clone(),
            details: Some(configuration::delegate_config_change_approval_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: reply_0
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
            reason: (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::DelegateConfigChangeApprovalRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.delegate_config_change_approval(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ValidateThrottleImportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::validate_throttle_import_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
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
                subject: Some(ratelimiting::RateLimitRef {
                    version: reply_0.rate_limit.clone().unwrap_or_default().version.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ThrottleImportServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ValidateThrottleImportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_throttle_import(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::GenerateQuotaBucketReportResponse::default();
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
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run FetchQuotaBucketReport after validating the tenant and durably recording the command.
    async fn run_fetch_quota_bucket_report(
        &self,
        req: ratelimiting::FetchQuotaBucketReportRequest,
    ) -> Result<ratelimiting::FetchQuotaBucketReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FetchQuotaBucketReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchQuotaBucketReport", "running workflow");
        let outgoing_0 = configuration::ShareConfigSetReportRequest {
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
            context: req.context.clone(),
            details: Some(configuration::share_config_set_report_request::Details {
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ShareConfigSetReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_config_set_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::FetchQuotaBucketReportResponse::default();
        let contribution: String = reply_0.config_set.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .limits
                .is_empty()
            {
                response
                    .quota_bucket
                    .get_or_insert_with(Default::default)
                    .limits
                    .push(Default::default());
            }
            response.quota_bucket.get_or_insert_with(Default::default).limits[0].name = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleQuotaBucketReport after validating the tenant and durably recording the command.
    async fn run_schedule_quota_bucket_report(
        &self,
        req: ratelimiting::ScheduleQuotaBucketReportRequest,
    ) -> Result<ratelimiting::ScheduleQuotaBucketReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleQuotaBucketReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleQuotaBucketReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamRateLimitMetricsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::stream_rate_limit_metrics_request::Filter {
                labels: req.annotations.clone(),
                owner_id: req.quota_bucket_id.clone(),
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
            let mut client = RateLimitReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamRateLimitMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_rate_limit_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamRateLimitMetricsResponse::default();
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
        let outgoing_1 = ratelimiting::GetQuotaBucketRatesRequest {
            context: req.context.clone(),
            quota_bucket_id: req.quota_bucket_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::GetQuotaBucketRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_quota_bucket_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = labeling::BrowseLabelingTasksRequest {
            context: req.context.clone(),
            filter: Some(labeling::browse_labeling_tasks_request::Filter {
                labels: reply_1.quota_bucket.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = LabelingTaskQueryServiceClient::new(channel);
            let input: tonic::Request<labeling::BrowseLabelingTasksRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_labeling_tasks(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::ExplainThrottleDecisionRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_throttle_decision_request::Details {
                contact: Some(common::ContactInfo {
                    phone: reply_1
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
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: reply_1
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
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
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ThrottleValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainThrottleDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_throttle_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ScheduleQuotaBucketReportResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run StreamQuotaBucketMetrics after validating the tenant and durably recording the command.
    async fn run_stream_quota_bucket_metrics(
        &self,
        req: ratelimiting::StreamQuotaBucketMetricsRequest,
    ) -> Result<ratelimiting::StreamQuotaBucketMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamQuotaBucketMetrics", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamQuotaBucketMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ListQuotaBucketConflictsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_quota_bucket_conflicts_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListQuotaBucketConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_quota_bucket_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::RetryEnvironmentWorkflowRequest {
            context: req.context.clone(),
            details: Some(configuration::retry_environment_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    phone: req.subscriber.clone().unwrap_or_default().phone.clone(),
                    preferred_channel: req.subscriber.clone().unwrap_or_default().preferred_channel.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::RetryEnvironmentWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.retry_environment_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::PullThrottlesRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: reply_1
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .owner
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PullThrottlesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pull_throttles(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::StreamQuotaBucketMetricsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.throttle.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<common::PersonName> = req.subscriber.clone().unwrap_or_default().name.clone();
        if contribution != <Option<common::PersonName>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .name = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListQuotaBucketReports after validating the tenant and durably recording the command.
    async fn run_list_quota_bucket_reports(
        &self,
        req: ratelimiting::ListQuotaBucketReportsRequest,
    ) -> Result<ratelimiting::ListQuotaBucketReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListQuotaBucketReports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListQuotaBucketReports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = pricing::ListPriceListsRequest {
            context: req.context.clone(),
            filter: Some(pricing::list_price_lists_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = PriceListServiceClient::new(channel);
            let input: tonic::Request<pricing::ListPriceListsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_price_lists(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ListQuotaBucketReportsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ShareQuotaBucketReport after validating the tenant and durably recording the command.
    async fn run_share_quota_bucket_report(
        &self,
        req: ratelimiting::ShareQuotaBucketReportRequest,
    ) -> Result<ratelimiting::ShareQuotaBucketReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareQuotaBucketReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareQuotaBucketReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().template_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamRateLimitMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .recipient
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                phone: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .recipient
                    .clone()
                    .unwrap_or_default()
                    .phone
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
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = RateLimitReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamRateLimitMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_rate_limit_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamRateLimitMetricsResponse::default();
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
        let outgoing_1 = ratelimiting::ListRateLimitConflictsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_rate_limit_conflicts_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.details.clone().unwrap_or_default().variables.clone(),
                owner_id: req.quota_bucket_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListRateLimitConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_rate_limit_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::CompareThrottleOptionsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ThrottleCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CompareThrottleOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_throttle_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = yards::CloneYardRequest {
            context: req.context.clone(),
            yard: Some(yards::Yard {
                audit: Some(common::AuditInfo {
                    created_at: reply_2.throttle.clone().unwrap_or_default().occurred_at.clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    display_name: reply_2
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
                        .recipient
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
        let reply_3 = async move {
            let mut client = YardServiceClient::new(channel);
            let input: tonic::Request<yards::CloneYardRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_yard(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ShareQuotaBucketReportResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
            .yard
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .phone
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .phone = contribution;
        }
        Ok(response)
    }
    /// Run CountQuotaBucketTotals after validating the tenant and durably recording the command.
    async fn run_count_quota_bucket_totals(
        &self,
        req: ratelimiting::CountQuotaBucketTotalsRequest,
    ) -> Result<ratelimiting::CountQuotaBucketTotalsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountQuotaBucketTotals", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountQuotaBucketTotals", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = ratelimiting::CountQuotaBucketTotalsResponse::default();
        Ok(response)
    }
    /// Run DeleteQuotaBucketReport after validating the tenant and durably recording the command.
    async fn run_delete_quota_bucket_report(
        &self,
        req: ratelimiting::DeleteQuotaBucketReportRequest,
    ) -> Result<ratelimiting::DeleteQuotaBucketReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DeleteQuotaBucketReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteQuotaBucketReport", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = concessions::CloneConcessionStandRequest {
            concession_stand: Some(concessions::ConcessionStand {
                audit: Some(common::AuditInfo {
                    etag: req.etag.clone(),
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
            let input: tonic::Request<concessions::CloneConcessionStandRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_concession_stand(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = audit::CountAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::count_audit_events_request::Filter {
                created_after: reply_0
                    .concession_stand
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0
                    .concession_stand
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: (match reply_0.concession_stand.clone().unwrap_or_default().locator.clone() {
                    Some(concessions::concession_stand::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AuditEventQueryServiceClient::new(channel);
            let input: tonic::Request<audit::CountAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ValidateRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::validate_rate_limit_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .concession_stand
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
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .concession_stand
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: reply_0
                        .concession_stand
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                owner: reply_0.concession_stand.clone().unwrap_or_default().owner.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ValidateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = procurement::ResolvePurchaseRequisitionsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = PurchaseRequisitionQueryServiceClient::new(channel);
            let input: tonic::Request<procurement::ResolvePurchaseRequisitionsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_purchase_requisitions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::DeleteQuotaBucketReportResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { QuotaBucketReportingService, QuotaBucketReportingHandler;
unary {
generate_quota_bucket_report (ratelimiting::GenerateQuotaBucketReportRequest) -> ratelimiting::GenerateQuotaBucketReportResponse => run_generate_quota_bucket_report;
fetch_quota_bucket_report (ratelimiting::FetchQuotaBucketReportRequest) -> ratelimiting::FetchQuotaBucketReportResponse => run_fetch_quota_bucket_report;
schedule_quota_bucket_report (ratelimiting::ScheduleQuotaBucketReportRequest) -> ratelimiting::ScheduleQuotaBucketReportResponse => run_schedule_quota_bucket_report;
list_quota_bucket_reports (ratelimiting::ListQuotaBucketReportsRequest) -> ratelimiting::ListQuotaBucketReportsResponse => run_list_quota_bucket_reports;
share_quota_bucket_report (ratelimiting::ShareQuotaBucketReportRequest) -> ratelimiting::ShareQuotaBucketReportResponse => run_share_quota_bucket_report;
count_quota_bucket_totals (ratelimiting::CountQuotaBucketTotalsRequest) -> ratelimiting::CountQuotaBucketTotalsResponse => run_count_quota_bucket_totals;
delete_quota_bucket_report (ratelimiting::DeleteQuotaBucketReportRequest) -> ratelimiting::DeleteQuotaBucketReportResponse => run_delete_quota_bucket_report;
}
server {
stream_quota_bucket_metrics (ratelimiting::StreamQuotaBucketMetricsRequest) -> ratelimiting::StreamQuotaBucketMetricsResponse, StreamQuotaBucketMetricsStream => run_stream_quota_bucket_metrics;
}
client {
}
bidi {
}
}
