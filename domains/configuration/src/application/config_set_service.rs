use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_query_service_client::RoleBindingQueryServiceClient;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::broadcasts::v1::broadcast_service_client::BroadcastServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_query_service_client::ConfigSetQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_server::ConfigSetService;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_validation_service_client::DocumentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::seating::v1 as seating;
use acme_proto::acme::seating::v1::seat_map_query_service_client::SeatMapQueryServiceClient;

/// Manages config sets: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct ConfigSetHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    delete_config_set_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ExplainEnvironmentDecisionRequest,
            configuration::ExplainEnvironmentDecisionResponse,
        >,
    >,
    delete_config_set_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::NormalizeEnvironmentRequest,
                configuration::NormalizeEnvironmentResponse,
            >,
    >,
    update_config_set_0_port: acme_platform::ports::Callback<
        configuration::ExplainConfigSetDecisionRequest,
        configuration::ExplainConfigSetDecisionResponse,
    >,
    watch_config_sets_0_port: acme_platform::ports::Callback<
        configuration::ExplainEnvironmentDecisionRequest,
        configuration::ExplainEnvironmentDecisionResponse,
    >,
}
impl ConfigSetHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::ExplainConfigSetDecisionRequest,
            configuration::ExplainConfigSetDecisionResponse,
        > = {
            let client = ConfigSetValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ExplainConfigSetDecisionRequest| -> acme_platform::ports::RpcFuture<configuration::ExplainConfigSetDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_config_set_decision(request).await?.into_inner())
})
})
        };
        let update_config_set_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::ExplainEnvironmentDecisionRequest,
            configuration::ExplainEnvironmentDecisionResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ExplainEnvironmentDecisionRequest| -> acme_platform::ports::RpcFuture<configuration::ExplainEnvironmentDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_environment_decision(request).await?.into_inner())
})
})
        };
        let delete_config_set_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::NormalizeEnvironmentRequest,
            configuration::NormalizeEnvironmentResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::NormalizeEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::NormalizeEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_environment(request).await?.into_inner())
})
})
        };
        let delete_config_set_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ExplainEnvironmentDecisionRequest,
            configuration::ExplainEnvironmentDecisionResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ExplainEnvironmentDecisionRequest| -> acme_platform::ports::RpcFuture<configuration::ExplainEnvironmentDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_environment_decision(request).await?.into_inner())
})
})
        };
        let watch_config_sets_0_port = callback;
        Self {
            channel,
            repository,
            delete_config_set_1_port,
            delete_config_set_2_port,
            update_config_set_0_port,
            watch_config_sets_0_port,
        }
    }
    /// Run CreateConfigSet after validating the tenant and durably recording the command.
    async fn run_create_config_set(
        &self,
        req: configuration::CreateConfigSetRequest,
    ) -> Result<configuration::CreateConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = featureflags::NormalizeFeatureFlagRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<featureflags::NormalizeFeatureFlagRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = FeatureFlagValidationServiceClient::new(channel);
                let input: tonic::Request<featureflags::NormalizeFeatureFlagRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.normalize_feature_flag(input).await?;
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
        let outgoing_1 = broadcasts::BatchCreateBroadcastsRequest {
            context: req.context.clone(),
            chunk: Some(broadcasts::batch_create_broadcasts_request::Chunk::Record(
                broadcasts::Broadcast {
                    audit: Some(common::AuditInfo {
                        updated_at: reply_0
                            .feature_flag
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        updated_by: reply_0
                            .feature_flag
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_by
                            .clone(),
                        ..Default::default()
                    }),
                    participants: vec![broadcasts::BroadcastParticipant {
                        contact: Some(common::ContactInfo {
                            email: reply_0
                                .feature_flag
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
                    }],
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = BroadcastServiceClient::new(channel);
            let input: tonic::Request<broadcasts::BatchCreateBroadcastsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_broadcasts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    attributes: req.config_set.clone().unwrap_or_default().attributes.clone(),
                    correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                    subject: Some(featureflags::FlagRolloutRef {
                        display_name: reply_0
                            .feature_flag
                            .clone()
                            .unwrap_or_default()
                            .owner
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        starts_at: req
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
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
        let reply_2 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
        let mut response = configuration::CreateConfigSetResponse::default();
        let contribution: String = reply_0.feature_flag.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_set.clone().unwrap_or_default().config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.config_set.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.config_set.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run GetConfigSet after validating the tenant and durably recording the command.
    async fn run_get_config_set(
        &self,
        req: configuration::GetConfigSetRequest,
    ) -> Result<configuration::GetConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetConfigSet", "running workflow");
        let mut response = configuration::GetConfigSetResponse::default();
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        Ok(response)
    }
    /// Run UpdateConfigSet after validating the tenant and durably recording the command.
    async fn run_update_config_set(
        &self,
        req: configuration::UpdateConfigSetRequest,
    ) -> Result<configuration::UpdateConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UpdateConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UpdateConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ExplainConfigSetDecisionRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_at: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            ..Default::default()
                        }),
                        children: vec![configuration::ConfigSet {
                            location: req.config_set.clone().unwrap_or_default().location.clone(),
                            ..Default::default()
                        }],
                        manager: Some(identity::UserRef {
                            email: req
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
                    }],
                    manager: Some(identity::UserRef {
                        user_id: req
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .manager
                            .clone()
                            .unwrap_or_default()
                            .user_id
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
        let reply_0 = (self.update_config_set_0_port)(outgoing_0).await?;
        let outgoing_1 = audit::CreateAuditEventRequest {
            audit_event: Some(audit::AuditEvent {
                attributes: (reply_0.config_set.clone().unwrap_or_default().attributes.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                measurements: vec![audit::AuditEventMeasurement {
                    observed_at: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }],
                subject: Some(audit::AuditTrailRef {
                    display_name: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::CreateAuditEventRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_audit_event(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::UpdateConfigSetResponse::default();
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
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
        let contribution: i32 = req.config_set.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.config_set.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.config_set.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.config_set.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run DeleteConfigSet after validating the tenant and durably recording the command.
    async fn run_delete_config_set(
        &self,
        req: configuration::DeleteConfigSetRequest,
    ) -> Result<configuration::DeleteConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::PreviewEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    etag: req.etag.clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::PreviewEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ExplainEnvironmentDecisionRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    configuration::EnvironmentComponent {
                        component_id: reply_0
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .components
                            .clone()
                            .get("primary")
                            .cloned()
                            .unwrap_or_default()
                            .component_id
                            .clone(),
                        description: reply_0
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .components
                            .clone()
                            .get("primary")
                            .cloned()
                            .unwrap_or_default()
                            .description
                            .clone(),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.delete_config_set_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::NormalizeEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    etag: reply_1
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                dimensions: Some(configuration::EnvironmentDimensions {
                    unit: reply_1
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .unit
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.delete_config_set_2_port.as_ref(), outgoing_2).await?;
        let mut response = configuration::DeleteConfigSetResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
        Ok(response)
    }
    /// Run BatchGetConfigSets after validating the tenant and durably recording the command.
    async fn run_batch_get_config_sets(
        &self,
        req: configuration::BatchGetConfigSetsRequest,
    ) -> Result<configuration::BatchGetConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchGetConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchGetConfigSets", "running workflow");
        let outgoing_0 = authorization::ResolveRoleBindingsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<authorization::ResolveRoleBindingsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RoleBindingQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::ResolveRoleBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_role_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::BatchGetConfigSetsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run SearchConfigSets after validating the tenant and durably recording the command.
    async fn run_search_config_sets(
        &self,
        req: configuration::SearchConfigSetsRequest,
    ) -> Result<configuration::SearchConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SearchConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SearchConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = configuration::SearchConfigSetsResponse::default();
        Ok(response)
    }
    /// Run WatchConfigSets after validating the tenant and durably recording the command.
    async fn run_watch_config_sets(
        &self,
        req: configuration::WatchConfigSetsRequest,
    ) -> Result<configuration::WatchConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("WatchConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "WatchConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = configuration::ExplainEnvironmentDecisionRequest {
            context: req.context.clone(),
            details: Some(configuration::explain_environment_decision_request::Details {
                contact: Some(common::ContactInfo {
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.watch_config_sets_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = configuration::BrowseConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::browse_config_sets_request::Filter {
                statuses: req.filter.clone().unwrap_or_default().statuses.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<configuration::BrowseConfigSetsRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::BrowseConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = WatchConfigSetsDispatch2::from(req.clone());
        command_2.config_set_children_children_location_postal_code =
            (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                Some(configuration::environment::Locator::Address(value)) => value,
                _ => Default::default(),
            })
            .postal_code
            .clone();
        let outgoing_2: configuration::ValidateConfigSetRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ValidateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::WatchConfigSetsResponse::default();
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .postal_code
        .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0]
                .manager
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run BatchCreateConfigSets after validating the tenant and durably recording the command.
    async fn run_batch_create_config_sets(
        &self,
        req: configuration::BatchCreateConfigSetsRequest,
    ) -> Result<configuration::BatchCreateConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchCreateConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateConfigSets", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(configuration::batch_create_config_sets_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = documents::CheckDocumentEligibilityRequest {
            context: req.context.clone(),
            details: Some(documents::check_document_eligibility_request::Details {
                contact: Some(common::ContactInfo {
                    phone: (match req.chunk.clone() {
                        Some(configuration::batch_create_config_sets_request::Chunk::Header(value)) => value,
                        _ => Default::default(),
                    })
                    .submitted_by
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            document: Some(documents::Document {
                audit: Some(common::AuditInfo {
                    created_at: (match req.chunk.clone() {
                        Some(configuration::batch_create_config_sets_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                    updated_by: (match req.chunk.clone() {
                        Some(configuration::batch_create_config_sets_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_by
                    .clone(),
                    ..Default::default()
                }),
                counterparty: Some(documents::DocumentFolderRef {
                    display_name: (match req.chunk.clone() {
                        Some(configuration::batch_create_config_sets_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .manager
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
        let mut response = configuration::BatchCreateConfigSetsResponse::default();
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
        Ok(response)
    }
    /// Run CloneConfigSet after validating the tenant and durably recording the command.
    async fn run_clone_config_set(
        &self,
        req: configuration::CloneConfigSetRequest,
    ) -> Result<configuration::CloneConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CloneConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CloneConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = seating::ExportSeatMapsRequest {
            context: req.context.clone(),
            filter: Some(seating::export_seat_maps_request::Filter {
                created_after: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.config_set.clone().unwrap_or_default().attributes.clone(),
                owner_id: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .config_set
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
        let _reply_0 = async move {
            let mut client = SeatMapQueryServiceClient::new(channel);
            let input: tonic::Request<seating::ExportSeatMapsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_seat_maps(input).await?;
            let mut frames = response.into_inner();
            let mut last = seating::ExportSeatMapsResponse::default();
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
        let outgoing_1 = configuration::ResolveConfigSetsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.configuration.v1.ResolveConfigSetsRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::ResolveConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::CloneConfigSetResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.config_set.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.config_set.get_or_insert_with(Default::default).location = contribution;
        }
        let contribution: i32 = req.config_set.clone().unwrap_or_default().depth.clone();
        if contribution != <i32>::default() {
            response.config_set.get_or_insert_with(Default::default).depth = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by WatchConfigSets for ValidateConfigSet.
struct WatchConfigSetsDispatch2 {
    context: Option<common::RequestContext>,
    config_set_children_children_location_postal_code: String,
}
impl From<configuration::WatchConfigSetsRequest> for WatchConfigSetsDispatch2 {
    fn from(req: configuration::WatchConfigSetsRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_children_location_postal_code: Default::default(),
        }
    }
}
impl From<WatchConfigSetsDispatch2> for configuration::ValidateConfigSetRequest {
    fn from(value: WatchConfigSetsDispatch2) -> Self {
        configuration::ValidateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            postal_code: value.config_set_children_children_location_postal_code,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigSetService for ConfigSetHandler {
    /// Creates a config set.
    async fn create_config_set(
        &self,
        request: tonic::Request<configuration::CreateConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::CreateConfigSetResponse>, tonic::Status> {
        self.run_create_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config set.
    async fn get_config_set(
        &self,
        request: tonic::Request<configuration::GetConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::GetConfigSetResponse>, tonic::Status> {
        self.run_get_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Changes a config set.
    async fn update_config_set(
        &self,
        request: tonic::Request<configuration::UpdateConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::UpdateConfigSetResponse>, tonic::Status> {
        self.run_update_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a config set.
    async fn delete_config_set(
        &self,
        request: tonic::Request<configuration::DeleteConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::DeleteConfigSetResponse>, tonic::Status> {
        self.run_delete_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several config sets at once.
    async fn batch_get_config_sets(
        &self,
        request: tonic::Request<configuration::BatchGetConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::BatchGetConfigSetsResponse>, tonic::Status> {
        self.run_batch_get_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists config sets a page at a time.
    async fn search_config_sets(
        &self,
        request: tonic::Request<configuration::SearchConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::SearchConfigSetsResponse>, tonic::Status> {
        self.run_search_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type WatchConfigSetsStream =
        tokio_stream::wrappers::ReceiverStream<Result<configuration::WatchConfigSetsResponse, tonic::Status>>;
    /// Streams the changes of config sets as they happen.
    async fn watch_config_sets(
        &self,
        request: tonic::Request<configuration::WatchConfigSetsRequest>,
    ) -> Result<tonic::Response<Self::WatchConfigSetsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_watch_config_sets(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Takes a stream of config sets and answers once at the end.
    async fn batch_create_config_sets(
        &self,
        request: tonic::Request<tonic::Streaming<configuration::BatchCreateConfigSetsRequest>>,
    ) -> Result<tonic::Response<configuration::BatchCreateConfigSetsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = configuration::BatchCreateConfigSetsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_batch_create_config_sets(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Creates a config set.
    async fn clone_config_set(
        &self,
        request: tonic::Request<configuration::CloneConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::CloneConfigSetResponse>, tonic::Status> {
        self.run_clone_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
