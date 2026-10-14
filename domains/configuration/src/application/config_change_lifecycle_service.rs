use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::anomalies::v1::anomaly_query_service_client::AnomalyQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_history_service_client::PermissionCheckHistoryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_server::ConfigChangeLifecycleService;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::dispatch::v1::dispatch_order_service_client::DispatchOrderServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_service_client::CryptoKeyServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_query_service_client::SecretQueryServiceClient;

/// Moves a config change between the states of its lifecycle.
#[derive(Clone)]
pub struct ConfigChangeLifecycleApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    activate_config_change_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::SyncEnvironmentsRequest,
                configuration::SyncEnvironmentsResponse,
            >,
    >,
    activate_config_change_1_port: acme_platform::ports::Callback<
        configuration::ListEnvironmentConflictsRequest,
        configuration::ListEnvironmentConflictsResponse,
    >,
    renew_config_change_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<ratelimiting::PreviewRateLimitRequest, ratelimiting::PreviewRateLimitResponse>,
    >,
    renew_config_change_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::SearchConfigSetsRequest,
                configuration::SearchConfigSetsResponse,
            >,
    >,
    renew_config_change_2_port: acme_platform::ports::Callback<
        configuration::ListConfigChangeReportsRequest,
        configuration::ListConfigChangeReportsResponse,
    >,
    suspend_config_change_0_port:
        acme_platform::ports::Callback<secrets::SummarizeSecretsRequest, secrets::SummarizeSecretsResponse>,
}
impl ConfigChangeLifecycleApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::SyncEnvironmentsRequest,
            configuration::SyncEnvironmentsResponse,
        > = {
            let client = EnvironmentSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SyncEnvironmentsRequest| -> acme_platform::ports::RpcFuture<configuration::SyncEnvironmentsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.sync_environments(request).await?.into_inner())
})
})
        };
        let activate_config_change_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ListEnvironmentConflictsRequest,
            configuration::ListEnvironmentConflictsResponse,
        > = {
            let client = EnvironmentSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ListEnvironmentConflictsRequest| -> acme_platform::ports::RpcFuture<configuration::ListEnvironmentConflictsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_environment_conflicts(request).await?.into_inner())
})
})
        };
        let activate_config_change_1_port = callback;
        let callback: acme_platform::ports::Callback<
            secrets::SummarizeSecretsRequest,
            secrets::SummarizeSecretsResponse,
        > = {
            let client = SecretQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::SummarizeSecretsRequest| -> acme_platform::ports::RpcFuture<secrets::SummarizeSecretsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.summarize_secrets(request).await?.into_inner())
})
})
        };
        let suspend_config_change_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::PreviewRateLimitRequest,
            ratelimiting::PreviewRateLimitResponse,
        > = {
            let client = RateLimitValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::PreviewRateLimitRequest| -> acme_platform::ports::RpcFuture<ratelimiting::PreviewRateLimitResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_rate_limit(request).await?.into_inner())
})
})
        };
        let renew_config_change_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::SearchConfigSetsRequest,
            configuration::SearchConfigSetsResponse,
        > = {
            let client = ConfigSetServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SearchConfigSetsRequest| -> acme_platform::ports::RpcFuture<configuration::SearchConfigSetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.search_config_sets(request).await?.into_inner())
})
})
        };
        let renew_config_change_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ListConfigChangeReportsRequest,
            configuration::ListConfigChangeReportsResponse,
        > = {
            let client = ConfigChangeReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ListConfigChangeReportsRequest| -> acme_platform::ports::RpcFuture<configuration::ListConfigChangeReportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_config_change_reports(request).await?.into_inner())
})
})
        };
        let renew_config_change_2_port = callback;
        Self {
            channel,
            repository,
            activate_config_change_0_port,
            activate_config_change_1_port,
            renew_config_change_0_port,
            renew_config_change_1_port,
            renew_config_change_2_port,
            suspend_config_change_0_port,
        }
    }
    /// Run ActivateConfigChange after validating the tenant and durably recording the command.
    async fn run_activate_config_change(
        &self,
        req: configuration::ActivateConfigChangeRequest,
    ) -> Result<configuration::ActivateConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ActivateConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ActivateConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = configuration::SyncEnvironmentsRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let reply_0 = acme_platform::ports::execute(self.activate_config_change_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::ListEnvironmentConflictsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_environment_conflicts_request::Filter {
                created_after: reply_0
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: reply_0
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = (self.activate_config_change_1_port)(outgoing_1).await?;
        let outgoing_2 = dispatch::BatchCreateDispatchOrdersRequest {
            context: req.context.clone(),
            chunk: Some(dispatch::batch_create_dispatch_orders_request::Chunk::Record(
                dispatch::DispatchOrder {
                    audit: Some(common::AuditInfo {
                        updated_by: reply_0
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_by
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = DispatchOrderServiceClient::new(channel);
            let input: tonic::Request<dispatch::BatchCreateDispatchOrdersRequest> =
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
            let response = client.batch_create_dispatch_orders(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    audit: Some(common::AuditInfo {
                        updated_at: reply_0
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        ..Default::default()
                    }),
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
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
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
        let mut response = configuration::ActivateConfigChangeResponse::default();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run SuspendConfigChange after validating the tenant and durably recording the command.
    async fn run_suspend_config_change(
        &self,
        req: configuration::SuspendConfigChangeRequest,
    ) -> Result<configuration::SuspendConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuspendConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuspendConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = secrets::SummarizeSecretsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.suspend_config_change_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = configuration::NormalizeConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            display_name: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .new_owner
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .notify_contact
                                .clone()
                                .unwrap_or_default()
                                .email
                                .clone(),
                            user_id: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .new_owner
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
        let outgoing_1 = acme_platform::ports::RequestBuilder::<configuration::NormalizeConfigSetRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::NormalizeConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::SuspendConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ReinstateConfigChange after validating the tenant and durably recording the command.
    async fn run_reinstate_config_change(
        &self,
        req: configuration::ReinstateConfigChangeRequest,
    ) -> Result<configuration::ReinstateConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReinstateConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReinstateConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ExplainKeyRingDecisionRequest {
            context: req.context.clone(),
            details: Some(keymanagement::explain_key_ring_decision_request::Details {
                contact: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    manager: Some(identity::UserRef {
                        display_name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .new_owner
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .new_owner
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::ExplainKeyRingDecisionRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = KeyRingValidationServiceClient::new(channel);
                let input: tonic::Request<keymanagement::ExplainKeyRingDecisionRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_key_ring_decision(input).await?;
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
        let outgoing_1 = ratelimiting::CheckRateLimitEligibilityRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::check_rate_limit_eligibility_request::Details {
                contact: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CheckRateLimitEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_rate_limit_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = authorization::AnnotatePermissionCheckVersionRequest {
            context: req.context.clone(),
            details: Some(authorization::annotate_permission_check_version_request::Details {
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            permission_check: Some(authorization::PermissionCheck {
                correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                subject: Some(authorization::AccessPolicyRef {
                    display_name: reply_0
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: reply_1.rate_limit.clone().unwrap_or_default().version.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = PermissionCheckHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::AnnotatePermissionCheckVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.annotate_permission_check_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = anomalies::BrowseAnomaliesRequest {
            context: req.context.clone(),
            filter: Some(anomalies::browse_anomalies_request::Filter {
                created_after: reply_1
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                region: reply_0
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
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = AnomalyQueryServiceClient::new(channel);
            let input: tonic::Request<anomalies::BrowseAnomaliesRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_anomalies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ReinstateConfigChangeResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .new_owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run TerminateConfigChange after validating the tenant and durably recording the command.
    async fn run_terminate_config_change(
        &self,
        req: configuration::TerminateConfigChangeRequest,
    ) -> Result<configuration::TerminateConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TerminateConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TerminateConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let command_0 = TerminateConfigChangeDispatch0::from(req.clone());
        let outgoing_0: keymanagement::CreateCryptoKeyRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyServiceClient::new(channel);
            let input: tonic::Request<keymanagement::CreateCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::PreviewConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            display_name: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .new_owner
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .notify_contact
                                .clone()
                                .unwrap_or_default()
                                .email
                                .clone(),
                            user_id: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .new_owner
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.configuration.v1.PreviewConfigSetRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::PreviewConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::TerminateConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RenewConfigChange after validating the tenant and durably recording the command.
    async fn run_renew_config_change(
        &self,
        req: configuration::RenewConfigChangeRequest,
    ) -> Result<configuration::RenewConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RenewConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RenewConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PreviewRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::preview_rate_limit_request::Details {
                contact: Some(common::ContactInfo {
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let reply_0 = acme_platform::ports::execute(self.renew_config_change_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::SearchConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::search_config_sets_request::Filter {
                created_after: reply_0.outcome.clone().unwrap_or_default().effective_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.renew_config_change_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::ListConfigChangeReportsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_config_change_reports_request::Filter {
                created_after: reply_0
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_1.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = (self.renew_config_change_2_port)(outgoing_2).await?;
        let mut response = configuration::RenewConfigChangeResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ExpireConfigChange after validating the tenant and durably recording the command.
    async fn run_expire_config_change(
        &self,
        req: configuration::ExpireConfigChangeRequest,
    ) -> Result<configuration::ExpireConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExpireConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExpireConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = configuration::ResolveEnvironmentConflictRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<configuration::ResolveEnvironmentConflictRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::ResolveEnvironmentConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_environment_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = ExpireConfigChangeDispatch1::from(req.clone());
        command_1.filter_region = (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .region
        .clone();
        command_1.filter_created_after = reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        command_1.filter_labels = reply_0.environment.clone().unwrap_or_default().specs.clone();
        let outgoing_1: configuration::BrowseConfigChangesRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::BrowseConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ExpireConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run GetConfigChangeLifecycle after validating the tenant and durably recording the command.
    async fn run_get_config_change_lifecycle(
        &self,
        req: configuration::GetConfigChangeLifecycleRequest,
    ) -> Result<configuration::GetConfigChangeLifecycleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetConfigChangeLifecycle", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetConfigChangeLifecycle", "running workflow");
        let outgoing_0 = configuration::ShareConfigChangeReportRequest {
            config_change_id: req.config_change_id.clone(),
            context: req.context.clone(),
            details: Some(configuration::share_config_change_report_request::Details {
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
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.ShareConfigChangeReportRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ShareConfigChangeReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_config_change_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::GetConfigChangeLifecycleResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_change.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleConfigChangeExpiry after validating the tenant and durably recording the command.
    async fn run_schedule_config_change_expiry(
        &self,
        req: configuration::ScheduleConfigChangeExpiryRequest,
    ) -> Result<configuration::ScheduleConfigChangeExpiryResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleConfigChangeExpiry", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleConfigChangeExpiry", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().kind.clone(), "recorded audit field");
        let outgoing_0 = secrets::ExportSecretsRequest {
            context: req.context.clone(),
            filter: Some(secrets::export_secrets_request::Filter {
                created_after: req.config_change.clone().unwrap_or_default().occurred_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req.config_change.clone().unwrap_or_default().config_change_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = SecretQueryServiceClient::new(channel);
            let input: tonic::Request<secrets::ExportSecretsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_secrets(input).await?;
            let mut frames = response.into_inner();
            let mut last = secrets::ExportSecretsResponse::default();
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
        let mut response = configuration::ScheduleConfigChangeExpiryResponse::default();
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.config_change.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.config_change.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by TerminateConfigChange for CreateCryptoKey.
struct TerminateConfigChangeDispatch0 {
    context: Option<common::RequestContext>,
    crypto_key_contact_email: String,
    crypto_key_contact_preferred_channel: String,
    crypto_key_contact_name: Option<common::PersonName>,
    crypto_key_contact_phone: String,
}
impl From<configuration::TerminateConfigChangeRequest> for TerminateConfigChangeDispatch0 {
    fn from(req: configuration::TerminateConfigChangeRequest) -> Self {
        Self {
            context: req.context.clone(),
            crypto_key_contact_email: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            crypto_key_contact_preferred_channel: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .preferred_channel
                .clone(),
            crypto_key_contact_name: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .name
                .clone(),
            crypto_key_contact_phone: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .phone
                .clone(),
        }
    }
}
impl From<TerminateConfigChangeDispatch0> for keymanagement::CreateCryptoKeyRequest {
    fn from(value: TerminateConfigChangeDispatch0) -> Self {
        keymanagement::CreateCryptoKeyRequest {
            context: value.context,
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: value.crypto_key_contact_email,
                    name: value.crypto_key_contact_name,
                    phone: value.crypto_key_contact_phone,
                    preferred_channel: value.crypto_key_contact_preferred_channel,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by ExpireConfigChange for BrowseConfigChanges.
struct ExpireConfigChangeDispatch1 {
    context: Option<common::RequestContext>,
    filter_region: String,
    filter_created_after: Option<prost_types::Timestamp>,
    filter_owner_id: String,
    filter_labels: std::collections::BTreeMap<String, String>,
}
impl From<configuration::ExpireConfigChangeRequest> for ExpireConfigChangeDispatch1 {
    fn from(req: configuration::ExpireConfigChangeRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_region: Default::default(),
            filter_created_after: Default::default(),
            filter_owner_id: req
                .details
                .clone()
                .unwrap_or_default()
                .new_owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            filter_labels: Default::default(),
        }
    }
}
impl From<ExpireConfigChangeDispatch1> for configuration::BrowseConfigChangesRequest {
    fn from(value: ExpireConfigChangeDispatch1) -> Self {
        configuration::BrowseConfigChangesRequest {
            context: value.context,
            filter: Some(configuration::browse_config_changes_request::Filter {
                created_after: value.filter_created_after,
                labels: value.filter_labels,
                owner_id: value.filter_owner_id,
                region: value.filter_region,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigChangeLifecycleService for ConfigChangeLifecycleApi {
    /// Activates a config change.
    async fn activate_config_change(
        &self,
        request: tonic::Request<configuration::ActivateConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::ActivateConfigChangeResponse>, tonic::Status> {
        self.run_activate_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Suspends a config change.
    async fn suspend_config_change(
        &self,
        request: tonic::Request<configuration::SuspendConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::SuspendConfigChangeResponse>, tonic::Status> {
        self.run_suspend_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reinstates a config change.
    async fn reinstate_config_change(
        &self,
        request: tonic::Request<configuration::ReinstateConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::ReinstateConfigChangeResponse>, tonic::Status> {
        self.run_reinstate_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Terminates a config change.
    async fn terminate_config_change(
        &self,
        request: tonic::Request<configuration::TerminateConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::TerminateConfigChangeResponse>, tonic::Status> {
        self.run_terminate_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Renews a config change.
    async fn renew_config_change(
        &self,
        request: tonic::Request<configuration::RenewConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::RenewConfigChangeResponse>, tonic::Status> {
        self.run_renew_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Expires a config change.
    async fn expire_config_change(
        &self,
        request: tonic::Request<configuration::ExpireConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::ExpireConfigChangeResponse>, tonic::Status> {
        self.run_expire_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config change.
    async fn get_config_change_lifecycle(
        &self,
        request: tonic::Request<configuration::GetConfigChangeLifecycleRequest>,
    ) -> Result<tonic::Response<configuration::GetConfigChangeLifecycleResponse>, tonic::Status> {
        self.run_get_config_change_lifecycle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a config change.
    async fn schedule_config_change_expiry(
        &self,
        request: tonic::Request<configuration::ScheduleConfigChangeExpiryRequest>,
    ) -> Result<tonic::Response<configuration::ScheduleConfigChangeExpiryResponse>, tonic::Status> {
        self.run_schedule_config_change_expiry(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
