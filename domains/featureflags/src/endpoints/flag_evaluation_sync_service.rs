use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_server::FlagEvaluationSyncService;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Keeps flag evaluations in step with the systems of record in other regions.
#[derive(Clone)]
pub struct FlagEvaluationSyncApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    list_flag_evaluation_conflicts_0_port:
        acme_platform::ports::Callback<venues::FindVenuesRequest, venues::FindVenuesResponse>,
}
impl FlagEvaluationSyncApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<venues::FindVenuesRequest, venues::FindVenuesResponse> = {
            let client = VenueQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: venues::FindVenuesRequest| -> acme_platform::ports::RpcFuture<venues::FindVenuesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.find_venues(request).await?.into_inner())
})
})
        };
        let list_flag_evaluation_conflicts_0_port = callback;
        Self {
            channel,
            repository,
            list_flag_evaluation_conflicts_0_port,
        }
    }
    /// Run SyncFlagEvaluations after validating the tenant and durably recording the command.
    async fn run_sync_flag_evaluations(
        &self,
        req: featureflags::SyncFlagEvaluationsRequest,
    ) -> Result<featureflags::SyncFlagEvaluationsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SyncFlagEvaluations", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SyncFlagEvaluations", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::SyncFlagEvaluationsResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: String = req.flag_evaluation.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.flag_evaluation.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run PushFlagEvaluation after validating the tenant and durably recording the command.
    async fn run_push_flag_evaluation(
        &self,
        req: featureflags::PushFlagEvaluationRequest,
    ) -> Result<featureflags::PushFlagEvaluationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PushFlagEvaluation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PushFlagEvaluation", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::PushFlagEvaluationResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: i64 = req.flag_evaluation.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.flag_evaluation.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run PullFlagEvaluations after validating the tenant and durably recording the command.
    async fn run_pull_flag_evaluations(
        &self,
        req: featureflags::PullFlagEvaluationsRequest,
    ) -> Result<featureflags::PullFlagEvaluationsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PullFlagEvaluations", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PullFlagEvaluations", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::PullFlagEvaluationsResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.flag_evaluation.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run ReconcileFlagEvaluations after validating the tenant and durably recording the command.
    async fn run_reconcile_flag_evaluations(
        &self,
        req: featureflags::ReconcileFlagEvaluationsRequest,
    ) -> Result<featureflags::ReconcileFlagEvaluationsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ReconcileFlagEvaluations", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReconcileFlagEvaluations", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let mut response = featureflags::ReconcileFlagEvaluationsResponse::default();
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
    /// Run StreamFlagEvaluationChanges after validating the tenant and durably recording the command.
    async fn run_stream_flag_evaluation_changes(
        &self,
        req: featureflags::StreamFlagEvaluationChangesRequest,
    ) -> Result<featureflags::StreamFlagEvaluationChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFlagEvaluationChanges", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFlagEvaluationChanges", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(value)) => value, _ => Default::default() }).flag_evaluation_id.clone(), "recorded audit field");
        let command_0 = StreamFlagEvaluationChangesDispatch0::from(req.clone());
        let outgoing_0: featureflags::CompareFlagRolloutOptionsRequest = command_0.into();
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
        let mut response = featureflags::StreamFlagEvaluationChangesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: i64 = (match req.frame.clone() {
            Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .sequence
        .clone();
        if contribution != <i64>::default() {
            response.flag_evaluation.get_or_insert_with(Default::default).sequence = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> = (match req.frame.clone() {
            Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .attributes
        .clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ResolveFlagEvaluationConflict after validating the tenant and durably recording the command.
    async fn run_resolve_flag_evaluation_conflict(
        &self,
        req: featureflags::ResolveFlagEvaluationConflictRequest,
    ) -> Result<featureflags::ResolveFlagEvaluationConflictResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResolveFlagEvaluationConflict", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveFlagEvaluationConflict", "running workflow");
        tracing::debug!(audit_value = ?req.flag_evaluation.clone().unwrap_or_default().flag_evaluation_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CompareFlagRolloutOptionsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                organizer: Some(identity::UserRef {
                    display_name: req
                        .flag_evaluation
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            flag_rollout_id: req
                .flag_evaluation
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .flag_rollout_id
                .clone(),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<featureflags::CompareFlagRolloutOptionsRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
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
        let mut response = featureflags::ResolveFlagEvaluationConflictResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
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
    /// Run GetFlagEvaluationSyncState after validating the tenant and durably recording the command.
    async fn run_get_flag_evaluation_sync_state(
        &self,
        req: featureflags::GetFlagEvaluationSyncStateRequest,
    ) -> Result<featureflags::GetFlagEvaluationSyncStateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetFlagEvaluationSyncState", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagEvaluationSyncState", "running workflow");
        let mut response = featureflags::GetFlagEvaluationSyncStateResponse::default();
        let contribution: String = req.flag_evaluation_id.clone();
        if contribution != <String>::default() {
            response
                .flag_evaluation
                .get_or_insert_with(Default::default)
                .flag_evaluation_id = contribution;
        }
        Ok(response)
    }
    /// Run ListFlagEvaluationConflicts after validating the tenant and durably recording the command.
    async fn run_list_flag_evaluation_conflicts(
        &self,
        req: featureflags::ListFlagEvaluationConflictsRequest,
    ) -> Result<featureflags::ListFlagEvaluationConflictsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFlagEvaluationConflicts", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFlagEvaluationConflicts", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = venues::FindVenuesRequest {
            context: req.context.clone(),
            page: Some(common::PageRequest {
                page_token: req.page.clone().unwrap_or_default().page_token.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.list_flag_evaluation_conflicts_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = featureflags::ListFlagEvaluationConflictsResponse::default();
        let contribution: i64 = reply_0.page_info.clone().unwrap_or_default().total_size.clone();
        if contribution != <i64>::default() {
            response.page_info.get_or_insert_with(Default::default).total_size = contribution;
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
        Ok(response)
    }
}
/// The domain command prepared by StreamFlagEvaluationChanges for CompareFlagRolloutOptions.
struct StreamFlagEvaluationChangesDispatch0 {
    context: Option<common::RequestContext>,
    flag_rollout_flag_rollout_id: String,
}
impl From<featureflags::StreamFlagEvaluationChangesRequest> for StreamFlagEvaluationChangesDispatch0 {
    fn from(req: featureflags::StreamFlagEvaluationChangesRequest) -> Self {
        Self {
            context: req.context.clone(),
            flag_rollout_flag_rollout_id: (match req.frame.clone() {
                Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(value)) => value,
                _ => Default::default(),
            })
            .subject
            .clone()
            .unwrap_or_default()
            .flag_rollout_id
            .clone(),
        }
    }
}
impl From<StreamFlagEvaluationChangesDispatch0> for featureflags::CompareFlagRolloutOptionsRequest {
    fn from(value: StreamFlagEvaluationChangesDispatch0) -> Self {
        featureflags::CompareFlagRolloutOptionsRequest {
            context: value.context,
            flag_rollout: Some(featureflags::FlagRollout {
                flag_rollout_id: value.flag_rollout_flag_rollout_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl FlagEvaluationSyncService for FlagEvaluationSyncApi {
    /// Syncs a flag evaluation.
    async fn sync_flag_evaluations(
        &self,
        request: tonic::Request<featureflags::SyncFlagEvaluationsRequest>,
    ) -> Result<tonic::Response<featureflags::SyncFlagEvaluationsResponse>, tonic::Status> {
        self.run_sync_flag_evaluations(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pushs a flag evaluation.
    async fn push_flag_evaluation(
        &self,
        request: tonic::Request<featureflags::PushFlagEvaluationRequest>,
    ) -> Result<tonic::Response<featureflags::PushFlagEvaluationResponse>, tonic::Status> {
        self.run_push_flag_evaluation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pulls a flag evaluation.
    async fn pull_flag_evaluations(
        &self,
        request: tonic::Request<featureflags::PullFlagEvaluationsRequest>,
    ) -> Result<tonic::Response<featureflags::PullFlagEvaluationsResponse>, tonic::Status> {
        self.run_pull_flag_evaluations(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reconciles a flag evaluation.
    async fn reconcile_flag_evaluations(
        &self,
        request: tonic::Request<featureflags::ReconcileFlagEvaluationsRequest>,
    ) -> Result<tonic::Response<featureflags::ReconcileFlagEvaluationsResponse>, tonic::Status> {
        self.run_reconcile_flag_evaluations(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamFlagEvaluationChangesStream = tokio_stream::wrappers::ReceiverStream<
        Result<featureflags::StreamFlagEvaluationChangesResponse, tonic::Status>,
    >;
    /// Exchanges changes of flag evaluations with a peer.
    async fn stream_flag_evaluation_changes(
        &self,
        request: tonic::Request<tonic::Streaming<featureflags::StreamFlagEvaluationChangesRequest>>,
    ) -> Result<tonic::Response<Self::StreamFlagEvaluationChangesStream>, tonic::Status> {
        let handler = self.clone();
        let mut incoming = request.into_inner();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let received = tokio::select! { _ = sender.closed() => break, value = incoming.message() => value };
                let result = match received {
                    Ok(Some(request)) => handler.run_stream_flag_evaluation_changes(request).await,
                    Ok(None) => break,
                    Err(error) => Err(error),
                };
                let failed = result.is_err();
                if sender.send(result).await.is_err() || failed {
                    break;
                }
            }
        });
        Ok(tonic::Response::new(tokio_stream::wrappers::ReceiverStream::new(
            receiver,
        )))
    }
    /// Resolves a flag evaluation.
    async fn resolve_flag_evaluation_conflict(
        &self,
        request: tonic::Request<featureflags::ResolveFlagEvaluationConflictRequest>,
    ) -> Result<tonic::Response<featureflags::ResolveFlagEvaluationConflictResponse>, tonic::Status> {
        self.run_resolve_flag_evaluation_conflict(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag evaluation.
    async fn get_flag_evaluation_sync_state(
        &self,
        request: tonic::Request<featureflags::GetFlagEvaluationSyncStateRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagEvaluationSyncStateResponse>, tonic::Status> {
        self.run_get_flag_evaluation_sync_state(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag evaluations a page at a time.
    async fn list_flag_evaluation_conflicts(
        &self,
        request: tonic::Request<featureflags::ListFlagEvaluationConflictsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFlagEvaluationConflictsResponse>, tonic::Status> {
        self.run_list_flag_evaluation_conflicts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
