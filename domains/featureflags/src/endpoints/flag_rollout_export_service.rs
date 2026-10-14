use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_export_service_server::FlagRolloutExportService;

/// Exports flag rollouts to files and other systems.
#[derive(Clone)]
pub struct FlagRolloutExportApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagRolloutExportApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StreamFlagRolloutExport after validating the tenant and durably recording the command.
    async fn run_stream_flag_rollout_export(
        &self,
        req: featureflags::StreamFlagRolloutExportRequest,
    ) -> Result<featureflags::StreamFlagRolloutExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFlagRolloutExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFlagRolloutExport", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    subject: Some(featureflags::FlagRolloutRef {
                        starts_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_flag_evaluation_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagEvaluationChangesResponse::default();
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
        let mut response = featureflags::StreamFlagRolloutExportResponse::default();
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run GetFlagRolloutExport after validating the tenant and durably recording the command.
    async fn run_get_flag_rollout_export(
        &self,
        req: featureflags::GetFlagRolloutExportRequest,
    ) -> Result<featureflags::GetFlagRolloutExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetFlagRolloutExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagRolloutExport", "running workflow");
        let mut response = featureflags::GetFlagRolloutExportResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        Ok(response)
    }
    /// Run CancelFlagRolloutExport after validating the tenant and durably recording the command.
    async fn run_cancel_flag_rollout_export(
        &self,
        req: featureflags::CancelFlagRolloutExportRequest,
    ) -> Result<featureflags::CancelFlagRolloutExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelFlagRolloutExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelFlagRolloutExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CompareFlagRolloutOptionsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify_contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify_contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        phone: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify_contact
                            .clone()
                            .unwrap_or_default()
                            .phone
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::CompareFlagRolloutOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_flag_rollout_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::CancelFlagRolloutExportResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ListFlagRolloutExports after validating the tenant and durably recording the command.
    async fn run_list_flag_rollout_exports(
        &self,
        req: featureflags::ListFlagRolloutExportsRequest,
    ) -> Result<featureflags::ListFlagRolloutExportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListFlagRolloutExports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFlagRolloutExports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFlagRolloutExportsResponse::default();
        Ok(response)
    }
    /// Run ScheduleFlagRolloutExport after validating the tenant and durably recording the command.
    async fn run_schedule_flag_rollout_export(
        &self,
        req: featureflags::ScheduleFlagRolloutExportRequest,
    ) -> Result<featureflags::ScheduleFlagRolloutExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleFlagRolloutExport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleFlagRolloutExport", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::ScheduleFlagRolloutExportResponse::default();
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run RetryFlagRolloutExport after validating the tenant and durably recording the command.
    async fn run_retry_flag_rollout_export(
        &self,
        req: featureflags::RetryFlagRolloutExportRequest,
    ) -> Result<featureflags::RetryFlagRolloutExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryFlagRolloutExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryFlagRolloutExport", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::RetryFlagRolloutExportResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: i32 = req.flag_rollout.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ShareFlagRolloutExport after validating the tenant and durably recording the command.
    async fn run_share_flag_rollout_export(
        &self,
        req: featureflags::ShareFlagRolloutExportRequest,
    ) -> Result<featureflags::ShareFlagRolloutExportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareFlagRolloutExport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareFlagRolloutExport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let mut response = featureflags::ShareFlagRolloutExportResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
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
                .flag_rollout
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { FlagRolloutExportService, FlagRolloutExportApi;
unary {
get_flag_rollout_export (featureflags::GetFlagRolloutExportRequest) -> featureflags::GetFlagRolloutExportResponse => run_get_flag_rollout_export;
cancel_flag_rollout_export (featureflags::CancelFlagRolloutExportRequest) -> featureflags::CancelFlagRolloutExportResponse => run_cancel_flag_rollout_export;
list_flag_rollout_exports (featureflags::ListFlagRolloutExportsRequest) -> featureflags::ListFlagRolloutExportsResponse => run_list_flag_rollout_exports;
schedule_flag_rollout_export (featureflags::ScheduleFlagRolloutExportRequest) -> featureflags::ScheduleFlagRolloutExportResponse => run_schedule_flag_rollout_export;
retry_flag_rollout_export (featureflags::RetryFlagRolloutExportRequest) -> featureflags::RetryFlagRolloutExportResponse => run_retry_flag_rollout_export;
share_flag_rollout_export (featureflags::ShareFlagRolloutExportRequest) -> featureflags::ShareFlagRolloutExportResponse => run_share_flag_rollout_export;
}
server {
stream_flag_rollout_export (featureflags::StreamFlagRolloutExportRequest) -> featureflags::StreamFlagRolloutExportResponse, StreamFlagRolloutExportStream => run_stream_flag_rollout_export;
}
client {
}
bidi {
}
}
