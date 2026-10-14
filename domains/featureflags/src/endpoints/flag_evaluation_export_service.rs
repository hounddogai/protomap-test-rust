use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_export_service_server::FlagEvaluationExportService;

/// Exports flag evaluations to files and other systems.
#[derive(Clone)]
pub struct FlagEvaluationExportEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagEvaluationExportEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StreamFlagEvaluationExport after validating the tenant and durably recording the command.
    async fn run_stream_flag_evaluation_export(
        &self,
        req: featureflags::StreamFlagEvaluationExportRequest,
    ) -> Result<featureflags::StreamFlagEvaluationExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFlagEvaluationExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFlagEvaluationExport", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let response = featureflags::StreamFlagEvaluationExportResponse::default();
        Ok(response)
    }
    /// Run GetFlagEvaluationExport after validating the tenant and durably recording the command.
    async fn run_get_flag_evaluation_export(
        &self,
        req: featureflags::GetFlagEvaluationExportRequest,
    ) -> Result<featureflags::GetFlagEvaluationExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetFlagEvaluationExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagEvaluationExport", "running workflow");
        let mut response = featureflags::GetFlagEvaluationExportResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
    /// Run CancelFlagEvaluationExport after validating the tenant and durably recording the command.
    async fn run_cancel_flag_evaluation_export(
        &self,
        req: featureflags::CancelFlagEvaluationExportRequest,
    ) -> Result<featureflags::CancelFlagEvaluationExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelFlagEvaluationExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelFlagEvaluationExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::CancelFlagEvaluationExportResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ListFlagEvaluationExports after validating the tenant and durably recording the command.
    async fn run_list_flag_evaluation_exports(
        &self,
        req: featureflags::ListFlagEvaluationExportsRequest,
    ) -> Result<featureflags::ListFlagEvaluationExportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFlagEvaluationExports", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFlagEvaluationExports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFlagEvaluationExportsResponse::default();
        Ok(response)
    }
    /// Run ScheduleFlagEvaluationExport after validating the tenant and durably recording the command.
    async fn run_schedule_flag_evaluation_export(
        &self,
        req: featureflags::ScheduleFlagEvaluationExportRequest,
    ) -> Result<featureflags::ScheduleFlagEvaluationExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleFlagEvaluationExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleFlagEvaluationExport", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::ScheduleFlagEvaluationExportResponse::default();
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run RetryFlagEvaluationExport after validating the tenant and durably recording the command.
    async fn run_retry_flag_evaluation_export(
        &self,
        req: featureflags::RetryFlagEvaluationExportRequest,
    ) -> Result<featureflags::RetryFlagEvaluationExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryFlagEvaluationExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryFlagEvaluationExport", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::RetryFlagEvaluationExportResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ShareFlagEvaluationExport after validating the tenant and durably recording the command.
    async fn run_share_flag_evaluation_export(
        &self,
        req: featureflags::ShareFlagEvaluationExportRequest,
    ) -> Result<featureflags::ShareFlagEvaluationExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ShareFlagEvaluationExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareFlagEvaluationExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let mut response = featureflags::ShareFlagEvaluationExportResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FlagEvaluationExportService for FlagEvaluationExportEndpoint {
    type StreamFlagEvaluationExportStream =
        tokio_stream::wrappers::ReceiverStream<Result<featureflags::StreamFlagEvaluationExportResponse, tonic::Status>>;
    /// Streams every matching flag evaluation.
    async fn stream_flag_evaluation_export(
        &self,
        request: tonic::Request<featureflags::StreamFlagEvaluationExportRequest>,
    ) -> Result<tonic::Response<Self::StreamFlagEvaluationExportStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(
                async move { handler.run_stream_flag_evaluation_export(request).await },
            );
        Ok(tonic::Response::new(stream))
    }
    /// Returns a flag evaluation.
    async fn get_flag_evaluation_export(
        &self,
        request: tonic::Request<featureflags::GetFlagEvaluationExportRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagEvaluationExportResponse>, tonic::Status> {
        self.run_get_flag_evaluation_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a flag evaluation.
    async fn cancel_flag_evaluation_export(
        &self,
        request: tonic::Request<featureflags::CancelFlagEvaluationExportRequest>,
    ) -> Result<tonic::Response<featureflags::CancelFlagEvaluationExportResponse>, tonic::Status> {
        self.run_cancel_flag_evaluation_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag evaluations a page at a time.
    async fn list_flag_evaluation_exports(
        &self,
        request: tonic::Request<featureflags::ListFlagEvaluationExportsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFlagEvaluationExportsResponse>, tonic::Status> {
        self.run_list_flag_evaluation_exports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a flag evaluation.
    async fn schedule_flag_evaluation_export(
        &self,
        request: tonic::Request<featureflags::ScheduleFlagEvaluationExportRequest>,
    ) -> Result<tonic::Response<featureflags::ScheduleFlagEvaluationExportResponse>, tonic::Status> {
        self.run_schedule_flag_evaluation_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a flag evaluation.
    async fn retry_flag_evaluation_export(
        &self,
        request: tonic::Request<featureflags::RetryFlagEvaluationExportRequest>,
    ) -> Result<tonic::Response<featureflags::RetryFlagEvaluationExportResponse>, tonic::Status> {
        self.run_retry_flag_evaluation_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Shares a flag evaluation.
    async fn share_flag_evaluation_export(
        &self,
        request: tonic::Request<featureflags::ShareFlagEvaluationExportRequest>,
    ) -> Result<tonic::Response<featureflags::ShareFlagEvaluationExportResponse>, tonic::Status> {
        self.run_share_flag_evaluation_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
