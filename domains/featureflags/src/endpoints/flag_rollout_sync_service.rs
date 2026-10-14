use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_server::FlagRolloutSyncService;
use acme_proto::acme::identity::v1 as identity;

/// Keeps flag rollouts in step with the systems of record in other regions.
#[derive(Clone)]
pub struct FlagRolloutSyncApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FlagRolloutSyncApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SyncFlagRollouts after validating the tenant and durably recording the command.
    async fn run_sync_flag_rollouts(
        &self,
        req: featureflags::SyncFlagRolloutsRequest,
    ) -> Result<featureflags::SyncFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SyncFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SyncFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::SyncFlagRolloutsResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.flag_rollout.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run PushFlagRollout after validating the tenant and durably recording the command.
    async fn run_push_flag_rollout(
        &self,
        req: featureflags::PushFlagRolloutRequest,
    ) -> Result<featureflags::PushFlagRolloutResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PushFlagRollout", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PushFlagRollout", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::PushFlagRolloutResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.flag_rollout.clone().unwrap_or_default().organizer.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).organizer = contribution;
        }
        Ok(response)
    }
    /// Run PullFlagRollouts after validating the tenant and durably recording the command.
    async fn run_pull_flag_rollouts(
        &self,
        req: featureflags::PullFlagRolloutsRequest,
    ) -> Result<featureflags::PullFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PullFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PullFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::QuoteFlagEvaluationRequest {
            context: req.context.clone(),
            details: Some(featureflags::quote_flag_evaluation_request::Details {
                destination: Some(common::PostalAddress {
                    recipient: req
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<featureflags::QuoteFlagEvaluationRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = FlagEvaluationCalculationServiceClient::new(channel);
                let input: tonic::Request<featureflags::QuoteFlagEvaluationRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.quote_flag_evaluation(input).await?;
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
        let mut response = featureflags::PullFlagRolloutsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.flag_rollout.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ReconcileFlagRollouts after validating the tenant and durably recording the command.
    async fn run_reconcile_flag_rollouts(
        &self,
        req: featureflags::ReconcileFlagRolloutsRequest,
    ) -> Result<featureflags::ReconcileFlagRolloutsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReconcileFlagRollouts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReconcileFlagRollouts", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::ReconcileFlagRolloutsResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.flag_rollout.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run StreamFlagRolloutChanges after validating the tenant and durably recording the command.
    async fn run_stream_flag_rollout_changes(
        &self,
        req: featureflags::StreamFlagRolloutChangesRequest,
    ) -> Result<featureflags::StreamFlagRolloutChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFlagRolloutChanges", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFlagRolloutChanges", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(value)) => value, _ => Default::default() }).flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::StreamFlagRolloutChangesResponse::default();
        let contribution: String = (match req.frame.clone() {
            Some(featureflags::stream_flag_rollout_changes_request::Frame::Ack(value)) => value,
            _ => Default::default(),
        })
        .status
        .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::TimeWindow> = (match req.frame.clone() {
            Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .window
        .clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).window = contribution;
        }
        Ok(response)
    }
    /// Run ResolveFlagRolloutConflict after validating the tenant and durably recording the command.
    async fn run_resolve_flag_rollout_conflict(
        &self,
        req: featureflags::ResolveFlagRolloutConflictRequest,
    ) -> Result<featureflags::ResolveFlagRolloutConflictResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResolveFlagRolloutConflict", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveFlagRolloutConflict", "running workflow");
        tracing::debug!(audit_value = ?req.flag_rollout.clone().unwrap_or_default().flag_rollout_id.clone(), "recorded audit field");
        let mut response = featureflags::ResolveFlagRolloutConflictResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.flag_rollout.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.flag_rollout.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run GetFlagRolloutSyncState after validating the tenant and durably recording the command.
    async fn run_get_flag_rollout_sync_state(
        &self,
        req: featureflags::GetFlagRolloutSyncStateRequest,
    ) -> Result<featureflags::GetFlagRolloutSyncStateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetFlagRolloutSyncState", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetFlagRolloutSyncState", "running workflow");
        let mut response = featureflags::GetFlagRolloutSyncStateResponse::default();
        let contribution: String = req.flag_rollout_id.clone();
        if contribution != <String>::default() {
            response
                .flag_rollout
                .get_or_insert_with(Default::default)
                .flag_rollout_id = contribution;
        }
        Ok(response)
    }
    /// Run ListFlagRolloutConflicts after validating the tenant and durably recording the command.
    async fn run_list_flag_rollout_conflicts(
        &self,
        req: featureflags::ListFlagRolloutConflictsRequest,
    ) -> Result<featureflags::ListFlagRolloutConflictsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFlagRolloutConflicts", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFlagRolloutConflicts", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFlagRolloutConflictsResponse::default();
        Ok(response)
    }
}
#[tonic::async_trait]
impl FlagRolloutSyncService for FlagRolloutSyncApi {
    /// Syncs a flag rollout.
    async fn sync_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::SyncFlagRolloutsRequest>,
    ) -> Result<tonic::Response<featureflags::SyncFlagRolloutsResponse>, tonic::Status> {
        self.run_sync_flag_rollouts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pushs a flag rollout.
    async fn push_flag_rollout(
        &self,
        request: tonic::Request<featureflags::PushFlagRolloutRequest>,
    ) -> Result<tonic::Response<featureflags::PushFlagRolloutResponse>, tonic::Status> {
        self.run_push_flag_rollout(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pulls a flag rollout.
    async fn pull_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::PullFlagRolloutsRequest>,
    ) -> Result<tonic::Response<featureflags::PullFlagRolloutsResponse>, tonic::Status> {
        self.run_pull_flag_rollouts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reconciles a flag rollout.
    async fn reconcile_flag_rollouts(
        &self,
        request: tonic::Request<featureflags::ReconcileFlagRolloutsRequest>,
    ) -> Result<tonic::Response<featureflags::ReconcileFlagRolloutsResponse>, tonic::Status> {
        self.run_reconcile_flag_rollouts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamFlagRolloutChangesStream =
        tokio_stream::wrappers::ReceiverStream<Result<featureflags::StreamFlagRolloutChangesResponse, tonic::Status>>;
    /// Exchanges changes of flag rollouts with a peer.
    async fn stream_flag_rollout_changes(
        &self,
        request: tonic::Request<tonic::Streaming<featureflags::StreamFlagRolloutChangesRequest>>,
    ) -> Result<tonic::Response<Self::StreamFlagRolloutChangesStream>, tonic::Status> {
        let handler = self.clone();
        let mut incoming = request.into_inner();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let received = tokio::select! { _ = sender.closed() => break, value = incoming.message() => value };
                let result = match received {
                    Ok(Some(request)) => handler.run_stream_flag_rollout_changes(request).await,
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
    /// Resolves a flag rollout.
    async fn resolve_flag_rollout_conflict(
        &self,
        request: tonic::Request<featureflags::ResolveFlagRolloutConflictRequest>,
    ) -> Result<tonic::Response<featureflags::ResolveFlagRolloutConflictResponse>, tonic::Status> {
        self.run_resolve_flag_rollout_conflict(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a flag rollout.
    async fn get_flag_rollout_sync_state(
        &self,
        request: tonic::Request<featureflags::GetFlagRolloutSyncStateRequest>,
    ) -> Result<tonic::Response<featureflags::GetFlagRolloutSyncStateResponse>, tonic::Status> {
        self.run_get_flag_rollout_sync_state(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists flag rollouts a page at a time.
    async fn list_flag_rollout_conflicts(
        &self,
        request: tonic::Request<featureflags::ListFlagRolloutConflictsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFlagRolloutConflictsResponse>, tonic::Status> {
        self.run_list_flag_rollout_conflicts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
