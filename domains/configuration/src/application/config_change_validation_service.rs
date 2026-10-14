use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::appointments::v1::appointment_service_client::AppointmentServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_notification_service_client::AccessPolicyNotificationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_change_validation_service_server::ConfigChangeValidationService;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_validation_service_client::DocumentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;

/// Checks config changes against the rules before anything acts on them.
#[derive(Clone)]
pub struct ConfigChangeValidationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    check_config_change_eligibility_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::GetConfigSetRulesRequest,
                configuration::GetConfigSetRulesResponse,
            >,
    >,
    get_config_change_rules_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::SimulateEnvironmentRequest,
            configuration::SimulateEnvironmentResponse,
        >,
    >,
    lint_config_changes_0_port:
        acme_platform::ports::Callback<appointments::CreateAppointmentRequest, appointments::CreateAppointmentResponse>,
    score_config_change_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::SimulateEnvironmentRequest,
            configuration::SimulateEnvironmentResponse,
        >,
    >,
    score_config_change_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::CompareEnvironmentOptionsRequest,
                configuration::CompareEnvironmentOptionsResponse,
            >,
    >,
    validate_config_change_0_port: acme_platform::ports::Callback<
        configuration::SimulateConfigSetRequest,
        configuration::SimulateConfigSetResponse,
    >,
    validate_config_change_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::EstimateConfigSetRequest,
            configuration::EstimateConfigSetResponse,
        >,
    >,
    validate_config_change_2_port: acme_platform::ports::Callback<
        configuration::SimulateConfigChangeRequest,
        configuration::SimulateConfigChangeResponse,
    >,
}
impl ConfigChangeValidationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::SimulateConfigSetRequest,
            configuration::SimulateConfigSetResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SimulateConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::SimulateConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_config_set(request).await?.into_inner())
})
})
        };
        let validate_config_change_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::EstimateConfigSetRequest,
            configuration::EstimateConfigSetResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::EstimateConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::EstimateConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_config_set(request).await?.into_inner())
})
})
        };
        let validate_config_change_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::SimulateConfigChangeRequest,
            configuration::SimulateConfigChangeResponse,
        > = {
            let client = ConfigChangeCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SimulateConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::SimulateConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_config_change(request).await?.into_inner())
})
})
        };
        let validate_config_change_2_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::GetConfigSetRulesRequest,
            configuration::GetConfigSetRulesResponse,
        > = {
            let client = ConfigSetValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::GetConfigSetRulesRequest| -> acme_platform::ports::RpcFuture<configuration::GetConfigSetRulesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.get_config_set_rules(request).await?.into_inner())
})
})
        };
        let check_config_change_eligibility_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::SimulateEnvironmentRequest,
            configuration::SimulateEnvironmentResponse,
        > = {
            let client = EnvironmentCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SimulateEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::SimulateEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_environment(request).await?.into_inner())
})
})
        };
        let score_config_change_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::CompareEnvironmentOptionsRequest,
            configuration::CompareEnvironmentOptionsResponse,
        > = {
            let client = EnvironmentCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CompareEnvironmentOptionsRequest| -> acme_platform::ports::RpcFuture<configuration::CompareEnvironmentOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_environment_options(request).await?.into_inner())
})
})
        };
        let score_config_change_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            appointments::CreateAppointmentRequest,
            appointments::CreateAppointmentResponse,
        > = {
            let client = AppointmentServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: appointments::CreateAppointmentRequest| -> acme_platform::ports::RpcFuture<appointments::CreateAppointmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.create_appointment(request).await?.into_inner())
})
})
        };
        let lint_config_changes_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::SimulateEnvironmentRequest,
            configuration::SimulateEnvironmentResponse,
        > = {
            let client = EnvironmentCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SimulateEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::SimulateEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_environment(request).await?.into_inner())
})
})
        };
        let get_config_change_rules_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            check_config_change_eligibility_0_port,
            get_config_change_rules_0_port,
            lint_config_changes_0_port,
            score_config_change_1_port,
            score_config_change_2_port,
            validate_config_change_0_port,
            validate_config_change_1_port,
            validate_config_change_2_port,
        }
    }
    /// Run ValidateConfigChange after validating the tenant and durably recording the command.
    async fn run_validate_config_change(
        &self,
        req: configuration::ValidateConfigChangeRequest,
    ) -> Result<configuration::ValidateConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        children: vec![configuration::ConfigSet {
                            attributes: req.config_change.clone().unwrap_or_default().attributes.clone(),
                            ..Default::default()
                        }],
                        config_set_id: req
                            .config_change
                            .clone()
                            .unwrap_or_default()
                            .subject
                            .clone()
                            .unwrap_or_default()
                            .config_set_id
                            .clone(),
                        manager: Some(identity::UserRef {
                            display_name: req
                                .config_change
                                .clone()
                                .unwrap_or_default()
                                .subject
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .contact
                                .clone()
                                .unwrap_or_default()
                                .email
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
        let reply_0 = (self.validate_config_change_0_port)(outgoing_0).await?;
        let outgoing_1 = configuration::EstimateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            region: reply_0
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.validate_config_change_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = configuration::SimulateConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    config_set_id: req
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .config_set_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::simulate_config_change_request::Details {
                destination: Some(common::PostalAddress {
                    lines: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    recipient: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    region: reply_0
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
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.validate_config_change_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let outgoing_3 = notifications::BatchCreateNotificationsRequest {
            context: req.context.clone(),
            chunk: Some(notifications::batch_create_notifications_request::Chunk::Record(
                notifications::Notification {
                    assignee: Some(identity::UserRef {
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
                    audit: Some(common::AuditInfo {
                        updated_at: reply_0
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        updated_by: reply_1
                            .config_set
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
        let reply_3 = async move {
            let mut client = NotificationServiceClient::new(channel);
            let input: tonic::Request<notifications::BatchCreateNotificationsRequest> =
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
            let response = client.batch_create_notifications(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    owner: Some(identity::UserRef {
                        user_id: reply_0
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_4);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_feature_flag_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFeatureFlagChangesResponse::default();
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
        let mut response = configuration::ValidateConfigChangeResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_set.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        let contribution: String = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
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
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2
            .config_change
            .clone()
            .unwrap_or_default()
            .config_change_id
            .clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_change.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run PreviewConfigChange after validating the tenant and durably recording the command.
    async fn run_preview_config_change(
        &self,
        req: configuration::PreviewConfigChangeRequest,
    ) -> Result<configuration::PreviewConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::RecalculateFlagRolloutsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        phone: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
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
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = FlagRolloutCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::RecalculateFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_flag_rollouts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = configuration::ScoreConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_at: reply_0
                                .flag_rollout
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            country_code: reply_0
                                .flag_rollout
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            recipient: reply_0
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
                        manager: Some(identity::UserRef {
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .contact
                                .clone()
                                .unwrap_or_default()
                                .email
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ScoreConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::PreviewConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
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
        let contribution: Option<prost_types::Timestamp> =
            req.config_change.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.config_change.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run CheckConfigChangeEligibility after validating the tenant and durably recording the command.
    async fn run_check_config_change_eligibility(
        &self,
        req: configuration::CheckConfigChangeEligibilityRequest,
    ) -> Result<configuration::CheckConfigChangeEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckConfigChangeEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckConfigChangeEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::GetConfigSetRulesRequest {
            config_set_id: req
                .config_change
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .config_set_id
                .clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.check_config_change_eligibility_0_port.as_ref(), outgoing_0).await?;
        let mut response = configuration::CheckConfigChangeEligibilityResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
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
        let contribution: Option<configuration::ConfigSetRef> =
            req.config_change.clone().unwrap_or_default().subject.clone();
        if contribution != <Option<configuration::ConfigSetRef>>::default() {
            response.config_change.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeConfigChange after validating the tenant and durably recording the command.
    async fn run_normalize_config_change(
        &self,
        req: configuration::NormalizeConfigChangeRequest,
    ) -> Result<configuration::NormalizeConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let command_0 = NormalizeConfigChangeDispatch0::from(req.clone());
        let outgoing_0: authorization::SubscribeAccessPolicyUpdatesRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessPolicyNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::SubscribeAccessPolicyUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_access_policy_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::PreviewConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        created_at: reply_0
                            .access_policy
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        created_by: reply_0
                            .access_policy
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        ..Default::default()
                    }),
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_at: reply_0
                                .access_policy
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
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
        let outgoing_2 = configuration::QuoteEnvironmentRequest {
            context: req.context.clone(),
            details: Some(configuration::quote_environment_request::Details {
                destination: Some(common::PostalAddress {
                    region: reply_1
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
            }),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
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
                })),
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .access_policy
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::QuoteEnvironmentRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = EnvironmentCalculationServiceClient::new(channel);
                let input: tonic::Request<configuration::QuoteEnvironmentRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.quote_environment(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_2)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_2 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = configuration::NormalizeConfigChangeResponse::default();
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
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: String = req.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ExplainConfigChangeDecision after validating the tenant and durably recording the command.
    async fn run_explain_config_change_decision(
        &self,
        req: configuration::ExplainConfigChangeDecisionRequest,
    ) -> Result<configuration::ExplainConfigChangeDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainConfigChangeDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainConfigChangeDecision", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let jobs_0 = req
            .config_change
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = documents::PreviewDocumentRequest {
                        context: req.context.clone(),
                        details: Some(documents::preview_document_request::Details {
                            contact: Some(common::ContactInfo {
                                email: req
                                    .details
                                    .clone()
                                    .unwrap_or_default()
                                    .contact
                                    .clone()
                                    .unwrap_or_default()
                                    .email
                                    .clone(),
                                phone: req
                                    .details
                                    .clone()
                                    .unwrap_or_default()
                                    .contact
                                    .clone()
                                    .unwrap_or_default()
                                    .phone
                                    .clone(),
                                preferred_channel: req
                                    .details
                                    .clone()
                                    .unwrap_or_default()
                                    .contact
                                    .clone()
                                    .unwrap_or_default()
                                    .preferred_channel
                                    .clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        document: Some(documents::Document {
                            audit: Some(common::AuditInfo {
                                created_at: item.observed_at.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = DocumentValidationServiceClient::new(channel);
                        let input: tonic::Request<documents::PreviewDocumentRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.preview_document(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = configuration::ExplainConfigChangeDecisionResponse::default();
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: String = req.config_change.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run ScoreConfigChange after validating the tenant and durably recording the command.
    async fn run_score_config_change(
        &self,
        req: configuration::ScoreConfigChangeRequest,
    ) -> Result<configuration::ScoreConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::QuoteEnvironmentRequest {
            context: req.context.clone(),
            details: Some(configuration::quote_environment_request::Details {
                budget: req.details.clone().unwrap_or_default().budget.clone(),
                destination: Some(common::PostalAddress {
                    recipient: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    lines: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    region: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .region
                        .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::QuoteEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::SimulateEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    recipient: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
                    display_name: reply_0
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .owner
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
        let reply_1 = acme_platform::ports::execute(self.score_config_change_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::CompareEnvironmentOptionsRequest {
            context: req.context.clone(),
            details: Some(configuration::compare_environment_options_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
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
                dimensions: Some(configuration::EnvironmentDimensions {
                    width: reply_1
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .width
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.score_config_change_2_port.as_ref(), outgoing_2).await?;
        let mut response = configuration::ScoreConfigChangeResponse::default();
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
        let contribution: String = reply_1
            .environment
            .clone()
            .unwrap_or_default()
            .dimensions
            .clone()
            .unwrap_or_default()
            .unit
            .clone();
        if contribution != <String>::default() {
            if response
                .config_change
                .get_or_insert_with(Default::default)
                .measurements
                .is_empty()
            {
                response
                    .config_change
                    .get_or_insert_with(Default::default)
                    .measurements
                    .push(Default::default());
            }
            response.config_change.get_or_insert_with(Default::default).measurements[0].unit = contribution;
        }
        let contribution: String = reply_2
            .environment
            .clone()
            .unwrap_or_default()
            .dimensions
            .clone()
            .unwrap_or_default()
            .unit
            .clone();
        if contribution != <String>::default() {
            if response
                .config_change
                .get_or_insert_with(Default::default)
                .measurements
                .is_empty()
            {
                response
                    .config_change
                    .get_or_insert_with(Default::default)
                    .measurements
                    .push(Default::default());
            }
            response.config_change.get_or_insert_with(Default::default).measurements[0].unit = contribution;
        }
        let contribution: String = reply_2
            .environment
            .clone()
            .unwrap_or_default()
            .owner
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
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: String = req.config_change.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run LintConfigChanges after validating the tenant and durably recording the command.
    async fn run_lint_config_changes(
        &self,
        req: configuration::LintConfigChangesRequest,
    ) -> Result<configuration::LintConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintConfigChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintConfigChanges", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = appointments::CreateAppointmentRequest {
            appointment: Some(appointments::Appointment {
                organizer: Some(identity::UserRef {
                    display_name: req
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
                participants: vec![appointments::AppointmentParticipant {
                    contact: Some(common::ContactInfo {
                        name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        phone: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .phone
                            .clone(),
                        preferred_channel: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .preferred_channel
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
        let port = acme_platform::ports::opaque(self.lint_config_changes_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = configuration::CheckConfigSetEligibilityRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            country_code: reply_0
                                .appointment
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            region: reply_0
                                .appointment
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .region
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
                                .config_change
                                .clone()
                                .unwrap_or_default()
                                .subject
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            user_id: reply_0
                                .appointment
                                .clone()
                                .unwrap_or_default()
                                .organizer
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
            acme_platform::ports::RequestBuilder::<configuration::CheckConfigSetEligibilityRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::CheckConfigSetEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_config_set_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::LintConfigChangesResponse::default();
        let contribution: String = reply_0
            .appointment
            .clone()
            .unwrap_or_default()
            .organizer
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.config_set.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: String = req.config_change.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run GetConfigChangeRules after validating the tenant and durably recording the command.
    async fn run_get_config_change_rules(
        &self,
        req: configuration::GetConfigChangeRulesRequest,
    ) -> Result<configuration::GetConfigChangeRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetConfigChangeRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetConfigChangeRules", "running workflow");
        let outgoing_0 = configuration::SimulateEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    created_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    configuration::EnvironmentComponent {
                        component_id: req.config_change_id.clone(),
                        ..Default::default()
                    },
                )]),
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
            }),
            ..Default::default()
        };
        let reply_0 =
            self.get_config_change_rules_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = configuration::GetConfigChangeRulesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
}
/// The domain command prepared by NormalizeConfigChange for SubscribeAccessPolicyUpdates.
struct NormalizeConfigChangeDispatch0 {
    context: Option<common::RequestContext>,
    details_recipient_preferred_channel: String,
}
impl From<configuration::NormalizeConfigChangeRequest> for NormalizeConfigChangeDispatch0 {
    fn from(req: configuration::NormalizeConfigChangeRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_recipient_preferred_channel: req
                .details
                .clone()
                .unwrap_or_default()
                .contact
                .clone()
                .unwrap_or_default()
                .preferred_channel
                .clone(),
        }
    }
}
impl From<NormalizeConfigChangeDispatch0> for authorization::SubscribeAccessPolicyUpdatesRequest {
    fn from(value: NormalizeConfigChangeDispatch0) -> Self {
        authorization::SubscribeAccessPolicyUpdatesRequest {
            context: value.context,
            details: Some(authorization::subscribe_access_policy_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    preferred_channel: value.details_recipient_preferred_channel,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigChangeValidationService for ConfigChangeValidationController {
    /// Validates a config change.
    async fn validate_config_change(
        &self,
        request: tonic::Request<configuration::ValidateConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::ValidateConfigChangeResponse>, tonic::Status> {
        self.run_validate_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a config change.
    async fn preview_config_change(
        &self,
        request: tonic::Request<configuration::PreviewConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::PreviewConfigChangeResponse>, tonic::Status> {
        self.run_preview_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a config change.
    async fn check_config_change_eligibility(
        &self,
        request: tonic::Request<configuration::CheckConfigChangeEligibilityRequest>,
    ) -> Result<tonic::Response<configuration::CheckConfigChangeEligibilityResponse>, tonic::Status> {
        self.run_check_config_change_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a config change.
    async fn normalize_config_change(
        &self,
        request: tonic::Request<configuration::NormalizeConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::NormalizeConfigChangeResponse>, tonic::Status> {
        self.run_normalize_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a config change.
    async fn explain_config_change_decision(
        &self,
        request: tonic::Request<configuration::ExplainConfigChangeDecisionRequest>,
    ) -> Result<tonic::Response<configuration::ExplainConfigChangeDecisionResponse>, tonic::Status> {
        self.run_explain_config_change_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a config change.
    async fn score_config_change(
        &self,
        request: tonic::Request<configuration::ScoreConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::ScoreConfigChangeResponse>, tonic::Status> {
        self.run_score_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a config change.
    async fn lint_config_changes(
        &self,
        request: tonic::Request<configuration::LintConfigChangesRequest>,
    ) -> Result<tonic::Response<configuration::LintConfigChangesResponse>, tonic::Status> {
        self.run_lint_config_changes(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config change.
    async fn get_config_change_rules(
        &self,
        request: tonic::Request<configuration::GetConfigChangeRulesRequest>,
    ) -> Result<tonic::Response<configuration::GetConfigChangeRulesResponse>, tonic::Status> {
        self.run_get_config_change_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
