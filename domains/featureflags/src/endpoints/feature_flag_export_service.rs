use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_export_service_server::FeatureFlagExportService;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;

/// Exports feature flags to files and other systems.
#[derive(Clone)]
pub struct FeatureFlagExportApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FeatureFlagExportApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StreamFeatureFlagExport after validating the tenant and durably recording the command.
    async fn run_stream_feature_flag_export(
        &self,
        req: featureflags::StreamFeatureFlagExportRequest,
    ) -> Result<featureflags::StreamFeatureFlagExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFeatureFlagExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFeatureFlagExport", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let response = featureflags::StreamFeatureFlagExportResponse::default();
        Ok(response)
    }
    /// Run GetFeatureFlagExport after validating the tenant and durably recording the command.
    async fn run_get_feature_flag_export(
        &self,
        req: featureflags::GetFeatureFlagExportRequest,
    ) -> Result<featureflags::GetFeatureFlagExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFeatureFlagExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFeatureFlagExport", "running workflow");
        let mut response = featureflags::GetFeatureFlagExportResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        Ok(response)
    }
    /// Run CancelFeatureFlagExport after validating the tenant and durably recording the command.
    async fn run_cancel_feature_flag_export(
        &self,
        req: featureflags::CancelFeatureFlagExportRequest,
    ) -> Result<featureflags::CancelFeatureFlagExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelFeatureFlagExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelFeatureFlagExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let mut response = featureflags::CancelFeatureFlagExportResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ListFeatureFlagExports after validating the tenant and durably recording the command.
    async fn run_list_feature_flag_exports(
        &self,
        req: featureflags::ListFeatureFlagExportsRequest,
    ) -> Result<featureflags::ListFeatureFlagExportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListFeatureFlagExports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFeatureFlagExports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFeatureFlagExportsResponse::default();
        Ok(response)
    }
    /// Run RetryFeatureFlagExport after validating the tenant and durably recording the command.
    async fn run_retry_feature_flag_export(
        &self,
        req: featureflags::RetryFeatureFlagExportRequest,
    ) -> Result<featureflags::RetryFeatureFlagExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryFeatureFlagExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryFeatureFlagExport", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::RetryFeatureFlagExportResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.feature_flag.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ShareFeatureFlagExport after validating the tenant and durably recording the command.
    async fn run_share_feature_flag_export(
        &self,
        req: featureflags::ShareFeatureFlagExportRequest,
    ) -> Result<featureflags::ShareFeatureFlagExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareFeatureFlagExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareFeatureFlagExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = featureflags::EstimateFlagEvaluationRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                attributes: (req.details.clone().unwrap_or_default().variables.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::EstimateFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::ShareFeatureFlagExportResponse::default();
        let contribution: String = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
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
                .feature_flag
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl FeatureFlagExportService for FeatureFlagExportApi {
    type StreamFeatureFlagExportStream =
        tokio_stream::wrappers::ReceiverStream<Result<featureflags::StreamFeatureFlagExportResponse, tonic::Status>>;
    /// Streams every matching feature flag.
    async fn stream_feature_flag_export(
        &self,
        request: tonic::Request<featureflags::StreamFeatureFlagExportRequest>,
    ) -> Result<tonic::Response<Self::StreamFeatureFlagExportStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_feature_flag_export(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Returns a feature flag.
    async fn get_feature_flag_export(
        &self,
        request: tonic::Request<featureflags::GetFeatureFlagExportRequest>,
    ) -> Result<tonic::Response<featureflags::GetFeatureFlagExportResponse>, tonic::Status> {
        self.run_get_feature_flag_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a feature flag.
    async fn cancel_feature_flag_export(
        &self,
        request: tonic::Request<featureflags::CancelFeatureFlagExportRequest>,
    ) -> Result<tonic::Response<featureflags::CancelFeatureFlagExportResponse>, tonic::Status> {
        self.run_cancel_feature_flag_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists feature flags a page at a time.
    async fn list_feature_flag_exports(
        &self,
        request: tonic::Request<featureflags::ListFeatureFlagExportsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFeatureFlagExportsResponse>, tonic::Status> {
        self.run_list_feature_flag_exports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a feature flag.
    async fn retry_feature_flag_export(
        &self,
        request: tonic::Request<featureflags::RetryFeatureFlagExportRequest>,
    ) -> Result<tonic::Response<featureflags::RetryFeatureFlagExportResponse>, tonic::Status> {
        self.run_retry_feature_flag_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Shares a feature flag.
    async fn share_feature_flag_export(
        &self,
        request: tonic::Request<featureflags::ShareFeatureFlagExportRequest>,
    ) -> Result<tonic::Response<featureflags::ShareFeatureFlagExportResponse>, tonic::Status> {
        self.run_share_feature_flag_export(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
