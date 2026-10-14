use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_query_service_client::DocumentQueryServiceClient;
use acme_proto::acme::documents::v1::document_validation_service_client::DocumentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient;
use acme_proto::acme::loyalty::v1 as loyalty;
use acme_proto::acme::loyalty::v1::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_client::ThrottleImportServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_server::ThrottleReportingService;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_query_service_client::SearchIndexQueryServiceClient;

/// Builds and schedules reports about throttles.
#[derive(Clone)]
pub struct ThrottleReportingController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    fetch_throttle_report_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<ratelimiting::QuoteQuotaBucketRequest, ratelimiting::QuoteQuotaBucketResponse>,
    >,
    generate_throttle_report_0_port:
        acme_platform::ports::Callback<search::ResolveSearchIndexesRequest, search::ResolveSearchIndexesResponse>,
    generate_throttle_report_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::ListRateLimitReportsRequest,
            ratelimiting::ListRateLimitReportsResponse,
        >,
    >,
    generate_throttle_report_2_port: acme_platform::ports::Callback<
        ratelimiting::GenerateQuotaBucketReportRequest,
        ratelimiting::GenerateQuotaBucketReportResponse,
    >,
}
impl ThrottleReportingController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            search::ResolveSearchIndexesRequest,
            search::ResolveSearchIndexesResponse,
        > = {
            let client = SearchIndexQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: search::ResolveSearchIndexesRequest| -> acme_platform::ports::RpcFuture<search::ResolveSearchIndexesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_search_indexes(request).await?.into_inner())
})
})
        };
        let generate_throttle_report_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::ListRateLimitReportsRequest,
            ratelimiting::ListRateLimitReportsResponse,
        > = {
            let client = RateLimitReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ListRateLimitReportsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ListRateLimitReportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_rate_limit_reports(request).await?.into_inner())
})
})
        };
        let generate_throttle_report_1_port =
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
        let generate_throttle_report_2_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::QuoteQuotaBucketRequest,
            ratelimiting::QuoteQuotaBucketResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::QuoteQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::QuoteQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_quota_bucket(request).await?.into_inner())
})
})
        };
        let fetch_throttle_report_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            fetch_throttle_report_0_port,
            generate_throttle_report_0_port,
            generate_throttle_report_1_port,
            generate_throttle_report_2_port,
        }
    }
    /// Run GenerateThrottleReport after validating the tenant and durably recording the command.
    async fn run_generate_throttle_report(
        &self,
        req: ratelimiting::GenerateThrottleReportRequest,
    ) -> Result<ratelimiting::GenerateThrottleReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GenerateThrottleReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateThrottleReport", "running workflow");
        tracing::debug!(audit_value = ?req.throttle_id.clone(), "recorded audit field");
        let outgoing_0 = search::ResolveSearchIndexesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = (self.generate_throttle_report_0_port)(outgoing_0).await?;
        let outgoing_1 = ratelimiting::ListRateLimitReportsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_rate_limit_reports_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.annotations.clone(),
                owner_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            self.generate_throttle_report_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = ratelimiting::GenerateQuotaBucketReportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::generate_quota_bucket_report_request::Details {
                filters: req.details.clone().unwrap_or_default().filters.clone(),
                period: req.details.clone().unwrap_or_default().period.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.generate_throttle_report_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let outgoing_3 = featureflags::RepairFlagEvaluationRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                correlation_id: reply_2.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_3 = acme_platform::ports::RequestBuilder::<featureflags::RepairFlagEvaluationRequest>::default()
            .map(|draft| *draft = outgoing_3)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FlagEvaluationAdminServiceClient::new(channel);
            let input: tonic::Request<featureflags::RepairFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.repair_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::GenerateThrottleReportResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run FetchThrottleReport after validating the tenant and durably recording the command.
    async fn run_fetch_throttle_report(
        &self,
        req: ratelimiting::FetchThrottleReportRequest,
    ) -> Result<ratelimiting::FetchThrottleReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FetchThrottleReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchThrottleReport", "running workflow");
        let outgoing_0 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    updated_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
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
                funding: Some(ratelimiting::quota_bucket::Funding::ExternalAccountId(
                    req.throttle_id.clone(),
                )),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.fetch_throttle_report_0_port.as_ref(), outgoing_0).await?;
        let mut response = ratelimiting::FetchThrottleReportResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleThrottleReport after validating the tenant and durably recording the command.
    async fn run_schedule_throttle_report(
        &self,
        req: ratelimiting::ScheduleThrottleReportRequest,
    ) -> Result<ratelimiting::ScheduleThrottleReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScheduleThrottleReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleThrottleReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ValidateThrottleImportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            throttle: Some(ratelimiting::Throttle {
                throttle_id: req.throttle_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.ratelimiting.v1.ValidateThrottleImportRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleImportServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ValidateThrottleImportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_throttle_import(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ScheduleThrottleReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run StreamThrottleMetrics after validating the tenant and durably recording the command.
    async fn run_stream_throttle_metrics(
        &self,
        req: ratelimiting::StreamThrottleMetricsRequest,
    ) -> Result<ratelimiting::StreamThrottleMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StreamThrottleMetrics", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamThrottleMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = documents::CheckDocumentEligibilityRequest {
            context: req.context.clone(),
            details: Some(documents::check_document_eligibility_request::Details {
                contact: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = DocumentValidationServiceClient::new(channel);
            let input: tonic::Request<documents::CheckDocumentEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_document_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::StreamQuotaBucketMetricsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::stream_quota_bucket_metrics_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
                name: req.subscriber.clone().unwrap_or_default().name.clone(),
                preferred_channel: req.subscriber.clone().unwrap_or_default().preferred_channel.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
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
        let mut response = ratelimiting::StreamThrottleMetricsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run ListThrottleReports after validating the tenant and durably recording the command.
    async fn run_list_throttle_reports(
        &self,
        req: ratelimiting::ListThrottleReportsRequest,
    ) -> Result<ratelimiting::ListThrottleReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListThrottleReports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListThrottleReports", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().owner_id.clone(), "recorded audit field");
        let command_0 = ListThrottleReportsDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::ListRateLimitConflictsRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListRateLimitConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_rate_limit_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ListThrottleReportsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            if response.throttles.is_empty() {
                response.throttles.push(Default::default());
            }
            response.throttles[0].correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ShareThrottleReport after validating the tenant and durably recording the command.
    async fn run_share_throttle_report(
        &self,
        req: ratelimiting::ShareThrottleReportRequest,
    ) -> Result<ratelimiting::ShareThrottleReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareThrottleReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareThrottleReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let outgoing_0 = documents::FindDocumentsRequest {
            context: req.context.clone(),
            filter: Some(documents::find_documents_request::Filter {
                labels: req.details.clone().unwrap_or_default().variables.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<documents::FindDocumentsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = DocumentQueryServiceClient::new(channel);
                let input: tonic::Request<documents::FindDocumentsRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.find_documents(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_0)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_0 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let outgoing_1 = ratelimiting::PushQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = ratelimiting::ShareThrottleReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i64 = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response
                .throttle
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.throttle_id.clone();
        if contribution != <String>::default() {
            response.throttle.get_or_insert_with(Default::default).throttle_id = contribution;
        }
        Ok(response)
    }
    /// Run CountThrottleTotals after validating the tenant and durably recording the command.
    async fn run_count_throttle_totals(
        &self,
        req: ratelimiting::CountThrottleTotalsRequest,
    ) -> Result<ratelimiting::CountThrottleTotalsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountThrottleTotals", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountThrottleTotals", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::GenerateQuotaBucketReportRequest {
            annotations: (req.filter.clone().unwrap_or_default().labels.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(ratelimiting::generate_quota_bucket_report_request::Details {
                filters: (req.context.clone().unwrap_or_default().baggage.clone())
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
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::GenerateQuotaBucketReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_quota_bucket_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CountThrottleTotalsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run DeleteThrottleReport after validating the tenant and durably recording the command.
    async fn run_delete_throttle_report(
        &self,
        req: ratelimiting::DeleteThrottleReportRequest,
    ) -> Result<ratelimiting::DeleteThrottleReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteThrottleReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteThrottleReport", "running workflow");
        tracing::debug!(audit_value = ?req.reason.clone(), "recorded audit field");
        let outgoing_0 = loyalty::BrowseLoyaltyMembersRequest {
            context: req.context.clone(),
            filter: Some(loyalty::browse_loyalty_members_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.throttle_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = LoyaltyMemberQueryServiceClient::new(channel);
            let input: tonic::Request<loyalty::BrowseLoyaltyMembersRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_loyalty_members(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ListQuotaBucketReportsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_quota_bucket_reports_request::Filter {
                labels: (reply_0.result.clone().unwrap_or_default().details.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req.throttle_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListQuotaBucketReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_quota_bucket_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::DeleteThrottleReportResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        Ok(response)
    }
}
/// The domain command prepared by ListThrottleReports for ListRateLimitConflicts.
struct ListThrottleReportsDispatch0 {
    context: Option<common::RequestContext>,
    filter_query: String,
    page_page_token: String,
    page_page_size: i32,
}
impl From<ratelimiting::ListThrottleReportsRequest> for ListThrottleReportsDispatch0 {
    fn from(req: ratelimiting::ListThrottleReportsRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_query: req.filter.clone().unwrap_or_default().query.clone(),
            page_page_token: req.page.clone().unwrap_or_default().page_token.clone(),
            page_page_size: req.page.clone().unwrap_or_default().page_size.clone(),
        }
    }
}
impl From<ListThrottleReportsDispatch0> for ratelimiting::ListRateLimitConflictsRequest {
    fn from(value: ListThrottleReportsDispatch0) -> Self {
        ratelimiting::ListRateLimitConflictsRequest {
            context: value.context,
            filter: Some(ratelimiting::list_rate_limit_conflicts_request::Filter {
                query: value.filter_query,
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_size: value.page_page_size,
                page_token: value.page_page_token,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ThrottleReportingService for ThrottleReportingController {
    /// Generates a throttle.
    async fn generate_throttle_report(
        &self,
        request: tonic::Request<ratelimiting::GenerateThrottleReportRequest>,
    ) -> Result<tonic::Response<ratelimiting::GenerateThrottleReportResponse>, tonic::Status> {
        self.run_generate_throttle_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a throttle.
    async fn fetch_throttle_report(
        &self,
        request: tonic::Request<ratelimiting::FetchThrottleReportRequest>,
    ) -> Result<tonic::Response<ratelimiting::FetchThrottleReportResponse>, tonic::Status> {
        self.run_fetch_throttle_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a throttle.
    async fn schedule_throttle_report(
        &self,
        request: tonic::Request<ratelimiting::ScheduleThrottleReportRequest>,
    ) -> Result<tonic::Response<ratelimiting::ScheduleThrottleReportResponse>, tonic::Status> {
        self.run_schedule_throttle_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamThrottleMetricsStream =
        tokio_stream::wrappers::ReceiverStream<Result<ratelimiting::StreamThrottleMetricsResponse, tonic::Status>>;
    /// Streams the changes of throttles as they happen.
    async fn stream_throttle_metrics(
        &self,
        request: tonic::Request<ratelimiting::StreamThrottleMetricsRequest>,
    ) -> Result<tonic::Response<Self::StreamThrottleMetricsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_throttle_metrics(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Lists throttles a page at a time.
    async fn list_throttle_reports(
        &self,
        request: tonic::Request<ratelimiting::ListThrottleReportsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListThrottleReportsResponse>, tonic::Status> {
        self.run_list_throttle_reports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Shares a throttle.
    async fn share_throttle_report(
        &self,
        request: tonic::Request<ratelimiting::ShareThrottleReportRequest>,
    ) -> Result<tonic::Response<ratelimiting::ShareThrottleReportResponse>, tonic::Status> {
        self.run_share_throttle_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts throttles by status.
    async fn count_throttle_totals(
        &self,
        request: tonic::Request<ratelimiting::CountThrottleTotalsRequest>,
    ) -> Result<tonic::Response<ratelimiting::CountThrottleTotalsResponse>, tonic::Status> {
        self.run_count_throttle_totals(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a throttle.
    async fn delete_throttle_report(
        &self,
        request: tonic::Request<ratelimiting::DeleteThrottleReportRequest>,
    ) -> Result<tonic::Response<ratelimiting::DeleteThrottleReportResponse>, tonic::Status> {
        self.run_delete_throttle_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
