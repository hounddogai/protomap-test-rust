use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::authorization::v1::permission_check_notification_service_client::PermissionCheckNotificationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_query_service_client::ConfigSetQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_server::EnvironmentReportingService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_export_service_client::FeatureFlagExportServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_client::FlagRolloutQueryServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::helpdesk::v1 as helpdesk;
use acme_proto::acme::helpdesk::v1::ticket_service_client::TicketServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_client::CryptoKeyQueryServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_query_service_client::KeyRingQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::seating::v1 as seating;
use acme_proto::acme::seating::v1::seat_map_service_client::SeatMapServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_validation_service_client::AccessGrantValidationServiceClient;

/// Builds and schedules reports about environments.
#[derive(Clone)]
pub struct EnvironmentReportingApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    generate_environment_report_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::CompareConfigSetOptionsRequest,
            configuration::CompareConfigSetOptionsResponse,
        >,
    >,
    list_environment_reports_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::ShareFeatureFlagExportRequest,
            featureflags::ShareFeatureFlagExportResponse,
        >,
    >,
    schedule_environment_report_0_port: acme_platform::ports::Callback<
        configuration::PreviewConfigChangeRequest,
        configuration::PreviewConfigChangeResponse,
    >,
    stream_environment_metrics_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ScoreConfigChangeRequest,
            configuration::ScoreConfigChangeResponse,
        >,
    >,
}
impl EnvironmentReportingApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::CompareConfigSetOptionsRequest,
            configuration::CompareConfigSetOptionsResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CompareConfigSetOptionsRequest| -> acme_platform::ports::RpcFuture<configuration::CompareConfigSetOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_config_set_options(request).await?.into_inner())
})
})
        };
        let generate_environment_report_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::PreviewConfigChangeRequest,
            configuration::PreviewConfigChangeResponse,
        > = {
            let client = ConfigChangeValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::PreviewConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::PreviewConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_config_change(request).await?.into_inner())
})
})
        };
        let schedule_environment_report_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::ScoreConfigChangeRequest,
            configuration::ScoreConfigChangeResponse,
        > = {
            let client = ConfigChangeValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ScoreConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::ScoreConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_config_change(request).await?.into_inner())
})
})
        };
        let stream_environment_metrics_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::ShareFeatureFlagExportRequest,
            featureflags::ShareFeatureFlagExportResponse,
        > = {
            let client = FeatureFlagExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ShareFeatureFlagExportRequest| -> acme_platform::ports::RpcFuture<featureflags::ShareFeatureFlagExportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_feature_flag_export(request).await?.into_inner())
})
})
        };
        let list_environment_reports_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            generate_environment_report_1_port,
            list_environment_reports_0_port,
            schedule_environment_report_0_port,
            stream_environment_metrics_2_port,
        }
    }
    /// Run GenerateEnvironmentReport after validating the tenant and durably recording the command.
    async fn run_generate_environment_report(
        &self,
        req: configuration::GenerateEnvironmentReportRequest,
    ) -> Result<configuration::GenerateEnvironmentReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GenerateEnvironmentReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateEnvironmentReport", "running workflow");
        tracing::debug!(audit_value = ?req.environment_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ExportKeyRingsRequest {
            context: req.context.clone(),
            format: req.details.clone().unwrap_or_default().format.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = KeyRingQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ExportKeyRingsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_key_rings(input).await?;
            let mut frames = response.into_inner();
            let mut last = keymanagement::ExportKeyRingsResponse::default();
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
        let outgoing_1 = configuration::CompareConfigSetOptionsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            email: req
                                .context
                                .clone()
                                .unwrap_or_default()
                                .actor
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            acme_platform::ports::execute(self.generate_environment_report_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = seating::BatchCreateSeatMapsRequest {
            context: req.context.clone(),
            chunk: Some(seating::batch_create_seat_maps_request::Chunk::Record(
                seating::SeatMap {
                    children: vec![seating::SeatMap {
                        children: vec![seating::SeatMap {
                            audit: Some(common::AuditInfo {
                                created_at: reply_1
                                    .config_set
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .created_at
                                    .clone(),
                                etag: reply_1
                                    .config_set
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .etag
                                    .clone(),
                                ..Default::default()
                            }),
                            location: Some(common::PostalAddress {
                                locality: reply_1
                                    .config_set
                                    .clone()
                                    .unwrap_or_default()
                                    .location
                                    .clone()
                                    .unwrap_or_default()
                                    .locality
                                    .clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = SeatMapServiceClient::new(channel);
            let input: tonic::Request<seating::BatchCreateSeatMapsRequest> =
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
            let response = client.batch_create_seat_maps(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    audit: Some(common::AuditInfo {
                        created_by: reply_1
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        ..Default::default()
                    }),
                    organizer: Some(identity::UserRef {
                        email: reply_1
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .manager
                            .clone()
                            .unwrap_or_default()
                            .email
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
        let reply_3 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_flag_rollout_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagRolloutChangesResponse::default();
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
        let mut response = configuration::GenerateEnvironmentReportResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run FetchEnvironmentReport after validating the tenant and durably recording the command.
    async fn run_fetch_environment_report(
        &self,
        req: configuration::FetchEnvironmentReportRequest,
    ) -> Result<configuration::FetchEnvironmentReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FetchEnvironmentReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchEnvironmentReport", "running workflow");
        let outgoing_0 = keymanagement::BrowseCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::browse_crypto_keys_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                owner_id: req.environment_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::BrowseCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::FetchEnvironmentReportResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleEnvironmentReport after validating the tenant and durably recording the command.
    async fn run_schedule_environment_report(
        &self,
        req: configuration::ScheduleEnvironmentReportRequest,
    ) -> Result<configuration::ScheduleEnvironmentReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleEnvironmentReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleEnvironmentReport", "running workflow");
        tracing::debug!(audit_value = ?req.environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::PreviewConfigChangeRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(configuration::preview_config_change_request::Details {
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
        let port = acme_platform::ports::opaque(self.schedule_environment_report_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = featureflags::ReconcileFeatureFlagsRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                action: Some(featureflags::feature_flag::Action::Block(
                    featureflags::FeatureFlagBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
                    display_name: reply_0
                        .config_change
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
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<featureflags::ReconcileFeatureFlagsRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ReconcileFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = ScheduleEnvironmentReportDispatch2::from(req.clone());
        command_2.key_ring_children_children_audit_etag = reply_1
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        let outgoing_2: keymanagement::CompareKeyRingOptionsRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = KeyRingCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::CompareKeyRingOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_key_ring_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::FindFlagRolloutsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::find_flag_rollouts_request::Filter {
                region: reply_2
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .location
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.featureflags.v1.FindFlagRolloutsRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FlagRolloutQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::FindFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_flag_rollouts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ScheduleEnvironmentReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
    /// Run StreamEnvironmentMetrics after validating the tenant and durably recording the command.
    async fn run_stream_environment_metrics(
        &self,
        req: configuration::StreamEnvironmentMetricsRequest,
    ) -> Result<configuration::StreamEnvironmentMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamEnvironmentMetrics", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamEnvironmentMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = authorization::WatchAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::watch_access_policies_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = AccessPolicyServiceClient::new(channel);
            let input: tonic::Request<authorization::WatchAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_access_policies(input).await?;
            let mut frames = response.into_inner();
            let mut last = authorization::WatchAccessPoliciesResponse::default();
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
        let outgoing_1 = configuration::CountConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_sets_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::CountConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::ScoreConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                attributes: req.filter.clone().unwrap_or_default().labels.clone(),
                correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                measurements: vec![configuration::ConfigChangeMeasurement {
                    observed_at: req.since.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::score_config_change_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 =
            acme_platform::ports::execute(self.stream_environment_metrics_2_port.as_ref(), outgoing_2).await?;
        let mut response = configuration::StreamEnvironmentMetricsResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListEnvironmentReports after validating the tenant and durably recording the command.
    async fn run_list_environment_reports(
        &self,
        req: configuration::ListEnvironmentReportsRequest,
    ) -> Result<configuration::ListEnvironmentReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListEnvironmentReports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListEnvironmentReports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ShareFeatureFlagExportRequest {
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
            context: req.context.clone(),
            details: Some(featureflags::share_feature_flag_export_request::Details {
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
                variables: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.list_environment_reports_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = configuration::ListEnvironmentReportsResponse::default();
        let contribution: String = reply_0.feature_flag.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response.environments.is_empty() {
                response.environments.push(Default::default());
            }
            response.environments[0].name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.environments.is_empty() {
                response.environments.push(Default::default());
            }
            response.environments[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        Ok(response)
    }
    /// Run ShareEnvironmentReport after validating the tenant and durably recording the command.
    async fn run_share_environment_report(
        &self,
        req: configuration::ShareEnvironmentReportRequest,
    ) -> Result<configuration::ShareEnvironmentReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareEnvironmentReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareEnvironmentReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::LintRateLimitsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::lint_rate_limits_request::Details {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ratelimiting::LintRateLimitsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = RateLimitValidationServiceClient::new(channel);
                let input: tonic::Request<ratelimiting::LintRateLimitsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.lint_rate_limits(input).await?;
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
        let outgoing_1 = secrets::ExplainAccessGrantDecisionRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                action: Some(secrets::access_grant::Action::Block(secrets::AccessGrantBlockAction {
                    reason: (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                        Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                        _ => Default::default(),
                    })
                    .reason
                    .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = AccessGrantValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::ExplainAccessGrantDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_access_grant_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = configuration::ShareEnvironmentReportResponse::default();
        let contribution: String = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
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
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
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
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run CountEnvironmentTotals after validating the tenant and durably recording the command.
    async fn run_count_environment_totals(
        &self,
        req: configuration::CountEnvironmentTotalsRequest,
    ) -> Result<configuration::CountEnvironmentTotalsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountEnvironmentTotals", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountEnvironmentTotals", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = helpdesk::UpdateTicketRequest {
            context: req.context.clone(),
            ticket: Some(helpdesk::Ticket {
                assignee: Some(identity::UserRef {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
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
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.helpdesk.v1.UpdateTicketRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = TicketServiceClient::new(channel);
            let input: tonic::Request<helpdesk::UpdateTicketRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_ticket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::CountEnvironmentTotalsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run DeleteEnvironmentReport after validating the tenant and durably recording the command.
    async fn run_delete_environment_report(
        &self,
        req: configuration::DeleteEnvironmentReportRequest,
    ) -> Result<configuration::DeleteEnvironmentReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DeleteEnvironmentReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteEnvironmentReport", "running workflow");
        tracing::debug!(audit_value = ?req.environment_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::UnsubscribePermissionCheckUpdatesRequest {
            context: req.context.clone(),
            details: Some(authorization::unsubscribe_permission_check_updates_request::Details {
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) =
            tokio::sync::mpsc::channel::<authorization::UnsubscribePermissionCheckUpdatesRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = PermissionCheckNotificationServiceClient::new(channel);
                let input: tonic::Request<authorization::UnsubscribePermissionCheckUpdatesRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.unsubscribe_permission_check_updates(input).await?;
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
        let outgoing_1 = configuration::CreateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        children: vec![configuration::ConfigSet {
                            attributes: reply_0.permission_check.clone().unwrap_or_default().attributes.clone(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        display_name: reply_0
                            .permission_check
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::CreateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = configuration::DeleteEnvironmentReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
/// The domain command prepared by ScheduleEnvironmentReport for CompareKeyRingOptions.
struct ScheduleEnvironmentReportDispatch2 {
    context: Option<common::RequestContext>,
    key_ring_children_children_audit_etag: String,
}
impl From<configuration::ScheduleEnvironmentReportRequest> for ScheduleEnvironmentReportDispatch2 {
    fn from(req: configuration::ScheduleEnvironmentReportRequest) -> Self {
        Self {
            context: req.context.clone(),
            key_ring_children_children_audit_etag: Default::default(),
        }
    }
}
impl From<ScheduleEnvironmentReportDispatch2> for keymanagement::CompareKeyRingOptionsRequest {
    fn from(value: ScheduleEnvironmentReportDispatch2) -> Self {
        keymanagement::CompareKeyRingOptionsRequest {
            context: value.context,
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            etag: value.key_ring_children_children_audit_etag,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl EnvironmentReportingService for EnvironmentReportingApi {
    /// Generates an environment.
    async fn generate_environment_report(
        &self,
        request: tonic::Request<configuration::GenerateEnvironmentReportRequest>,
    ) -> Result<tonic::Response<configuration::GenerateEnvironmentReportResponse>, tonic::Status> {
        self.run_generate_environment_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an environment.
    async fn fetch_environment_report(
        &self,
        request: tonic::Request<configuration::FetchEnvironmentReportRequest>,
    ) -> Result<tonic::Response<configuration::FetchEnvironmentReportResponse>, tonic::Status> {
        self.run_fetch_environment_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules an environment.
    async fn schedule_environment_report(
        &self,
        request: tonic::Request<configuration::ScheduleEnvironmentReportRequest>,
    ) -> Result<tonic::Response<configuration::ScheduleEnvironmentReportResponse>, tonic::Status> {
        self.run_schedule_environment_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamEnvironmentMetricsStream =
        tokio_stream::wrappers::ReceiverStream<Result<configuration::StreamEnvironmentMetricsResponse, tonic::Status>>;
    /// Streams the changes of environments as they happen.
    async fn stream_environment_metrics(
        &self,
        request: tonic::Request<configuration::StreamEnvironmentMetricsRequest>,
    ) -> Result<tonic::Response<Self::StreamEnvironmentMetricsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_environment_metrics(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Lists environments a page at a time.
    async fn list_environment_reports(
        &self,
        request: tonic::Request<configuration::ListEnvironmentReportsRequest>,
    ) -> Result<tonic::Response<configuration::ListEnvironmentReportsResponse>, tonic::Status> {
        self.run_list_environment_reports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Shares an environment.
    async fn share_environment_report(
        &self,
        request: tonic::Request<configuration::ShareEnvironmentReportRequest>,
    ) -> Result<tonic::Response<configuration::ShareEnvironmentReportResponse>, tonic::Status> {
        self.run_share_environment_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts environments by status.
    async fn count_environment_totals(
        &self,
        request: tonic::Request<configuration::CountEnvironmentTotalsRequest>,
    ) -> Result<tonic::Response<configuration::CountEnvironmentTotalsResponse>, tonic::Status> {
        self.run_count_environment_totals(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes an environment.
    async fn delete_environment_report(
        &self,
        request: tonic::Request<configuration::DeleteEnvironmentReportRequest>,
    ) -> Result<tonic::Response<configuration::DeleteEnvironmentReportResponse>, tonic::Status> {
        self.run_delete_environment_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
