use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_server::FeatureFlagSyncService;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Keeps feature flags in step with the systems of record in other regions.
#[derive(Clone)]
pub struct FeatureFlagSyncEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
}
impl FeatureFlagSyncEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::FeatureflagsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SyncFeatureFlags after validating the tenant and durably recording the command.
    async fn run_sync_feature_flags(
        &self,
        req: featureflags::SyncFeatureFlagsRequest,
    ) -> Result<featureflags::SyncFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SyncFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SyncFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::SyncFeatureFlagsResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req.feature_flag.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run PushFeatureFlag after validating the tenant and durably recording the command.
    async fn run_push_feature_flag(
        &self,
        req: featureflags::PushFeatureFlagRequest,
    ) -> Result<featureflags::PushFeatureFlagResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PushFeatureFlag", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PushFeatureFlag", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::PushFeatureFlagResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.feature_flag.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.feature_flag.get_or_insert_with(Default::default).effective = contribution;
        }
        Ok(response)
    }
    /// Run PullFeatureFlags after validating the tenant and durably recording the command.
    async fn run_pull_feature_flags(
        &self,
        req: featureflags::PullFeatureFlagsRequest,
    ) -> Result<featureflags::PullFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PullFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PullFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::PullFeatureFlagsResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: String = req.feature_flag.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run ReconcileFeatureFlags after validating the tenant and durably recording the command.
    async fn run_reconcile_feature_flags(
        &self,
        req: featureflags::ReconcileFeatureFlagsRequest,
    ) -> Result<featureflags::ReconcileFeatureFlagsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReconcileFeatureFlags", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReconcileFeatureFlags", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let command_0 = ReconcileFeatureFlagsDispatch0::from(req.clone());
        let outgoing_0: venues::SearchVenuesRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::SearchVenuesRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::ReconcileFeatureFlagsResponse::default();
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
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: i32 = req.feature_flag.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.feature_flag.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run StreamFeatureFlagChanges after validating the tenant and durably recording the command.
    async fn run_stream_feature_flag_changes(
        &self,
        req: featureflags::StreamFeatureFlagChangesRequest,
    ) -> Result<featureflags::StreamFeatureFlagChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamFeatureFlagChanges", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamFeatureFlagChanges", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(featureflags::stream_feature_flag_changes_request::Frame::Change(value)) => value, _ => Default::default() }).feature_flag_id.clone(), "recorded audit field");
        let outgoing_0 = venues::SearchVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::search_venues_request::Filter {
                created_after: (match req.frame.clone() {
                    Some(featureflags::stream_feature_flag_changes_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
                labels: (match req.frame.clone() {
                    Some(featureflags::stream_feature_flag_changes_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .parameters
                .clone(),
                owner_id: (match req.frame.clone() {
                    Some(featureflags::stream_feature_flag_changes_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::SearchVenuesRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = featureflags::StreamFeatureFlagChangesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = (match req.frame.clone() {
            Some(featureflags::stream_feature_flag_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .description
        .clone();
        if contribution != <String>::default() {
            response.feature_flag.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: i64 = (match req.frame.clone() {
            Some(featureflags::stream_feature_flag_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .version
        .clone();
        if contribution != <i64>::default() {
            response.feature_flag.get_or_insert_with(Default::default).version = contribution;
        }
        Ok(response)
    }
    /// Run ResolveFeatureFlagConflict after validating the tenant and durably recording the command.
    async fn run_resolve_feature_flag_conflict(
        &self,
        req: featureflags::ResolveFeatureFlagConflictRequest,
    ) -> Result<featureflags::ResolveFeatureFlagConflictResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResolveFeatureFlagConflict", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveFeatureFlagConflict", "running workflow");
        tracing::debug!(audit_value = ?req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(), "recorded audit field");
        let mut response = featureflags::ResolveFeatureFlagConflictResponse::default();
        let contribution: String = req.feature_flag_id.clone();
        if contribution != <String>::default() {
            response
                .feature_flag
                .get_or_insert_with(Default::default)
                .feature_flag_id = contribution;
        }
        let contribution: i32 = req.feature_flag.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.feature_flag.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run ListFeatureFlagConflicts after validating the tenant and durably recording the command.
    async fn run_list_feature_flag_conflicts(
        &self,
        req: featureflags::ListFeatureFlagConflictsRequest,
    ) -> Result<featureflags::ListFeatureFlagConflictsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListFeatureFlagConflicts", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListFeatureFlagConflicts", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = featureflags::ListFeatureFlagConflictsResponse::default();
        Ok(response)
    }
}
/// The domain command prepared by ReconcileFeatureFlags for SearchVenues.
struct ReconcileFeatureFlagsDispatch0 {
    context: Option<common::RequestContext>,
    filter_created_after: Option<prost_types::Timestamp>,
    filter_owner_id: String,
}
impl From<featureflags::ReconcileFeatureFlagsRequest> for ReconcileFeatureFlagsDispatch0 {
    fn from(req: featureflags::ReconcileFeatureFlagsRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_created_after: req
                .feature_flag
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
            filter_owner_id: req.feature_flag.clone().unwrap_or_default().feature_flag_id.clone(),
        }
    }
}
impl From<ReconcileFeatureFlagsDispatch0> for venues::SearchVenuesRequest {
    fn from(value: ReconcileFeatureFlagsDispatch0) -> Self {
        venues::SearchVenuesRequest {
            context: value.context,
            filter: Some(venues::search_venues_request::Filter {
                created_after: value.filter_created_after,
                owner_id: value.filter_owner_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl FeatureFlagSyncService for FeatureFlagSyncEndpoint {
    /// Syncs a feature flag.
    async fn sync_feature_flags(
        &self,
        request: tonic::Request<featureflags::SyncFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::SyncFeatureFlagsResponse>, tonic::Status> {
        self.run_sync_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pushs a feature flag.
    async fn push_feature_flag(
        &self,
        request: tonic::Request<featureflags::PushFeatureFlagRequest>,
    ) -> Result<tonic::Response<featureflags::PushFeatureFlagResponse>, tonic::Status> {
        self.run_push_feature_flag(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pulls a feature flag.
    async fn pull_feature_flags(
        &self,
        request: tonic::Request<featureflags::PullFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::PullFeatureFlagsResponse>, tonic::Status> {
        self.run_pull_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reconciles a feature flag.
    async fn reconcile_feature_flags(
        &self,
        request: tonic::Request<featureflags::ReconcileFeatureFlagsRequest>,
    ) -> Result<tonic::Response<featureflags::ReconcileFeatureFlagsResponse>, tonic::Status> {
        self.run_reconcile_feature_flags(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamFeatureFlagChangesStream =
        tokio_stream::wrappers::ReceiverStream<Result<featureflags::StreamFeatureFlagChangesResponse, tonic::Status>>;
    /// Exchanges changes of feature flags with a peer.
    async fn stream_feature_flag_changes(
        &self,
        request: tonic::Request<tonic::Streaming<featureflags::StreamFeatureFlagChangesRequest>>,
    ) -> Result<tonic::Response<Self::StreamFeatureFlagChangesStream>, tonic::Status> {
        let handler = self.clone();
        let mut incoming = request.into_inner();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let received = tokio::select! { _ = sender.closed() => break, value = incoming.message() => value };
                let result = match received {
                    Ok(Some(request)) => handler.run_stream_feature_flag_changes(request).await,
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
    /// Resolves a feature flag.
    async fn resolve_feature_flag_conflict(
        &self,
        request: tonic::Request<featureflags::ResolveFeatureFlagConflictRequest>,
    ) -> Result<tonic::Response<featureflags::ResolveFeatureFlagConflictResponse>, tonic::Status> {
        self.run_resolve_feature_flag_conflict(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists feature flags a page at a time.
    async fn list_feature_flag_conflicts(
        &self,
        request: tonic::Request<featureflags::ListFeatureFlagConflictsRequest>,
    ) -> Result<tonic::Response<featureflags::ListFeatureFlagConflictsResponse>, tonic::Status> {
        self.run_list_feature_flag_conflicts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
