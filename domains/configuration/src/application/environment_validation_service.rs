use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_server::EnvironmentValidationService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::seating::v1 as seating;
use acme_proto::acme::seating::v1::seat_map_calculation_service_client::SeatMapCalculationServiceClient;

/// Checks environments against the rules before anything acts on them.
#[derive(Clone)]
pub struct EnvironmentValidationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    normalize_environment_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::CompareConfigChangeOptionsRequest,
            configuration::CompareConfigChangeOptionsResponse,
        >,
    >,
    normalize_environment_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::SimulateConfigChangeRequest,
                configuration::SimulateConfigChangeResponse,
            >,
    >,
    preview_environment_0_port: acme_platform::ports::Callback<
        configuration::NormalizeConfigChangeRequest,
        configuration::NormalizeConfigChangeResponse,
    >,
    score_environment_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<seating::QuoteSeatMapRequest, seating::QuoteSeatMapResponse>,
    >,
    score_environment_1_port: acme_platform::ports::Callback<
        configuration::EstimateEnvironmentRequest,
        configuration::EstimateEnvironmentResponse,
    >,
}
impl EnvironmentValidationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::NormalizeConfigChangeRequest,
            configuration::NormalizeConfigChangeResponse,
        > = {
            let client = ConfigChangeValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::NormalizeConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::NormalizeConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_config_change(request).await?.into_inner())
})
})
        };
        let preview_environment_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::CompareConfigChangeOptionsRequest,
            configuration::CompareConfigChangeOptionsResponse,
        > = {
            let client = ConfigChangeCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CompareConfigChangeOptionsRequest| -> acme_platform::ports::RpcFuture<configuration::CompareConfigChangeOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_config_change_options(request).await?.into_inner())
})
})
        };
        let normalize_environment_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let normalize_environment_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<seating::QuoteSeatMapRequest, seating::QuoteSeatMapResponse> = {
            let client = SeatMapCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: seating::QuoteSeatMapRequest| -> acme_platform::ports::RpcFuture<seating::QuoteSeatMapResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_seat_map(request).await?.into_inner())
})
})
        };
        let score_environment_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::EstimateEnvironmentRequest,
            configuration::EstimateEnvironmentResponse,
        > = {
            let client = EnvironmentCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::EstimateEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::EstimateEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_environment(request).await?.into_inner())
})
})
        };
        let score_environment_1_port = callback;
        Self {
            channel,
            repository,
            normalize_environment_1_port,
            normalize_environment_2_port,
            preview_environment_0_port,
            score_environment_0_port,
            score_environment_1_port,
        }
    }
    /// Run ValidateEnvironment after validating the tenant and durably recording the command.
    async fn run_validate_environment(
        &self,
        req: configuration::ValidateEnvironmentRequest,
    ) -> Result<configuration::ValidateEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::NotifyFeatureFlagWatchersRequest {
            context: req.context.clone(),
            details: Some(featureflags::notify_feature_flag_watchers_request::Details {
                recipient: Some(common::ContactInfo {
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
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.featureflags.v1.NotifyFeatureFlagWatchersRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::NotifyFeatureFlagWatchersRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.notify_feature_flag_watchers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = audit::BatchCreateAuditEventsRequest {
            context: req.context.clone(),
            chunk: Some(audit::batch_create_audit_events_request::Chunk::Header(
                audit::batch_create_audit_events_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::BatchCreateAuditEventsRequest> =
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
            let response = client.batch_create_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        created_by: req
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
                    action: Some(featureflags::feature_flag::Action::Discount(
                        featureflags::FeatureFlagDiscountAction {
                            percent: (match reply_0.feature_flag.clone().unwrap_or_default().action.clone() {
                                Some(featureflags::feature_flag::Action::Discount(value)) => value,
                                _ => Default::default(),
                            })
                            .percent
                            .clone(),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
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
        let mut response = configuration::ValidateEnvironmentResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.environment.clone().unwrap_or_default().specs.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .specs
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PreviewEnvironment after validating the tenant and durably recording the command.
    async fn run_preview_environment(
        &self,
        req: configuration::PreviewEnvironmentRequest,
    ) -> Result<configuration::PreviewEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::NormalizeConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::normalize_config_change_request::Details {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.preview_environment_0_port)(outgoing_0).await?;
        let mut response = configuration::PreviewEnvironmentResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.environment.clone().unwrap_or_default().specs.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .specs
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CheckEnvironmentEligibility after validating the tenant and durably recording the command.
    async fn run_check_environment_eligibility(
        &self,
        req: configuration::CheckEnvironmentEligibilityRequest,
    ) -> Result<configuration::CheckEnvironmentEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckEnvironmentEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckEnvironmentEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let command_0 = CheckEnvironmentEligibilityDispatch0::from(req.clone());
        let outgoing_0: configuration::PreviewConfigChangeRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::PreviewConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::CheckEnvironmentEligibilityResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeEnvironment after validating the tenant and durably recording the command.
    async fn run_normalize_environment(
        &self,
        req: configuration::NormalizeEnvironmentRequest,
    ) -> Result<configuration::NormalizeEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::QuoteConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    unit: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .unit
                        .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::quote_config_change_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .postal_code
                    .clone(),
                    recipient: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .recipient
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::QuoteConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::CompareConfigChangeOptionsRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: reply_0
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    path: reply_0
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .path
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::compare_config_change_options_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .postal_code
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.normalize_environment_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::SimulateConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: reply_1
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
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.normalize_environment_2_port.as_ref(), outgoing_2).await?;
        let mut response = configuration::NormalizeEnvironmentResponse::default();
        let contribution: String = reply_0.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
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
        let contribution: String = req.environment.clone().unwrap_or_default().code.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).code = contribution;
        }
        Ok(response)
    }
    /// Run ExplainEnvironmentDecision after validating the tenant and durably recording the command.
    async fn run_explain_environment_decision(
        &self,
        req: configuration::ExplainEnvironmentDecisionRequest,
    ) -> Result<configuration::ExplainEnvironmentDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainEnvironmentDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainEnvironmentDecision", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::PreviewConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
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
            context: req.context.clone(),
            details: Some(configuration::preview_config_change_request::Details {
                contact: Some(common::ContactInfo {
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
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.PreviewConfigChangeRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::PreviewConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::NormalizeConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            country_code: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .country_code
                            .clone(),
                            locality: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .locality
                            .clone(),
                            region: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .region
                            .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::NormalizeConfigSetRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigSetValidationServiceClient::new(channel);
                let input: tonic::Request<configuration::NormalizeConfigSetRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.normalize_config_set(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_1)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_1 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let outgoing_2 = configuration::RecalculateConfigChangesRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    unit: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .unit
                        .clone(),
                    ..Default::default()
                }],
                subject: Some(configuration::ConfigSetRef {
                    display_name: reply_0
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    path: reply_0
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .path
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::recalculate_config_changes_request::Details {
                destination: Some(common::PostalAddress {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = configuration::ExplainEnvironmentDecisionResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: Option<identity::UserRef> = req.environment.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.environment.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run ScoreEnvironment after validating the tenant and durably recording the command.
    async fn run_score_environment(
        &self,
        req: configuration::ScoreEnvironmentRequest,
    ) -> Result<configuration::ScoreEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = seating::QuoteSeatMapRequest {
            context: req.context.clone(),
            seat_map: Some(seating::SeatMap {
                children: vec![seating::SeatMap {
                    children: vec![seating::SeatMap {
                        audit: Some(common::AuditInfo {
                            updated_by: req
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
                        location: Some(common::PostalAddress {
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
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.score_environment_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = configuration::EstimateEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    recipient: reply_0
                        .seat_map
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
                        .seat_map
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    user_id: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .owner
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
        let port = acme_platform::ports::opaque(self.score_environment_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let outgoing_2 = configuration::PreviewConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: reply_0
                                .seat_map
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<configuration::PreviewConfigSetRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::PreviewConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_3 = ScoreEnvironmentDispatch3::from(req.clone());
        command_3.details_contact_email = reply_0
            .seat_map
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        command_3.config_change_subject_display_name = reply_0
            .seat_map
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        command_3.config_change_subject_config_set_id =
            reply_2.config_set.clone().unwrap_or_default().config_set_id.clone();
        command_3.config_change_attributes = reply_2.config_set.clone().unwrap_or_default().attributes.clone();
        let outgoing_3: configuration::ExplainConfigChangeDecisionRequest = command_3.into();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainConfigChangeDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_config_change_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ScoreEnvironmentResponse::default();
        let contribution: String = reply_0.seat_map.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: Vec<String> = reply_1.environment.clone().unwrap_or_default().tags.clone();
        if contribution != <Vec<String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .tags
                .extend(contribution);
        }
        let contribution: String = reply_1.environment.clone().unwrap_or_default().code.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).code = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i32 = req.environment.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.environment.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run LintEnvironments after validating the tenant and durably recording the command.
    async fn run_lint_environments(
        &self,
        req: configuration::LintEnvironmentsRequest,
    ) -> Result<configuration::LintEnvironmentsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintEnvironments", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintEnvironments", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CheckConfigSetEligibilityRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_at: req
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
                        location: Some(common::PostalAddress {
                            lines: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .lines
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
        let reply_0 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::CheckConfigSetEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_config_set_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::LintEnvironmentsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i64 = req.environment.clone().unwrap_or_default().capacity.clone();
        if contribution != <i64>::default() {
            response.environment.get_or_insert_with(Default::default).capacity = contribution;
        }
        Ok(response)
    }
    /// Run GetEnvironmentRules after validating the tenant and durably recording the command.
    async fn run_get_environment_rules(
        &self,
        req: configuration::GetEnvironmentRulesRequest,
    ) -> Result<configuration::GetEnvironmentRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetEnvironmentRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetEnvironmentRules", "running workflow");
        let mut response = configuration::GetEnvironmentRulesResponse::default();
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CheckEnvironmentEligibility for PreviewConfigChange.
struct CheckEnvironmentEligibilityDispatch0 {
    context: Option<common::RequestContext>,
    details_contact_email: String,
    config_change_measurements_unit: String,
}
impl From<configuration::CheckEnvironmentEligibilityRequest> for CheckEnvironmentEligibilityDispatch0 {
    fn from(req: configuration::CheckEnvironmentEligibilityRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_contact_email: req
                .details
                .clone()
                .unwrap_or_default()
                .contact
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            config_change_measurements_unit: req
                .environment
                .clone()
                .unwrap_or_default()
                .dimensions
                .clone()
                .unwrap_or_default()
                .unit
                .clone(),
        }
    }
}
impl From<CheckEnvironmentEligibilityDispatch0> for configuration::PreviewConfigChangeRequest {
    fn from(value: CheckEnvironmentEligibilityDispatch0) -> Self {
        configuration::PreviewConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    unit: value.config_change_measurements_unit,
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: value.context,
            details: Some(configuration::preview_config_change_request::Details {
                contact: Some(common::ContactInfo {
                    email: value.details_contact_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by ScoreEnvironment for ExplainConfigChangeDecision.
struct ScoreEnvironmentDispatch3 {
    context: Option<common::RequestContext>,
    details_contact_email: String,
    config_change_subject_display_name: String,
    config_change_subject_config_set_id: String,
    config_change_attributes: std::collections::BTreeMap<String, String>,
}
impl From<configuration::ScoreEnvironmentRequest> for ScoreEnvironmentDispatch3 {
    fn from(req: configuration::ScoreEnvironmentRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_contact_email: Default::default(),
            config_change_subject_display_name: Default::default(),
            config_change_subject_config_set_id: Default::default(),
            config_change_attributes: Default::default(),
        }
    }
}
impl From<ScoreEnvironmentDispatch3> for configuration::ExplainConfigChangeDecisionRequest {
    fn from(value: ScoreEnvironmentDispatch3) -> Self {
        configuration::ExplainConfigChangeDecisionRequest {
            config_change: Some(configuration::ConfigChange {
                attributes: value.config_change_attributes,
                subject: Some(configuration::ConfigSetRef {
                    config_set_id: value.config_change_subject_config_set_id,
                    display_name: value.config_change_subject_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            details: Some(configuration::explain_config_change_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: value.details_contact_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl EnvironmentValidationService for EnvironmentValidationHandler {
    /// Validates an environment.
    async fn validate_environment(
        &self,
        request: tonic::Request<configuration::ValidateEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::ValidateEnvironmentResponse>, tonic::Status> {
        self.run_validate_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews an environment.
    async fn preview_environment(
        &self,
        request: tonic::Request<configuration::PreviewEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::PreviewEnvironmentResponse>, tonic::Status> {
        self.run_preview_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks an environment.
    async fn check_environment_eligibility(
        &self,
        request: tonic::Request<configuration::CheckEnvironmentEligibilityRequest>,
    ) -> Result<tonic::Response<configuration::CheckEnvironmentEligibilityResponse>, tonic::Status> {
        self.run_check_environment_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes an environment.
    async fn normalize_environment(
        &self,
        request: tonic::Request<configuration::NormalizeEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::NormalizeEnvironmentResponse>, tonic::Status> {
        self.run_normalize_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains an environment.
    async fn explain_environment_decision(
        &self,
        request: tonic::Request<configuration::ExplainEnvironmentDecisionRequest>,
    ) -> Result<tonic::Response<configuration::ExplainEnvironmentDecisionResponse>, tonic::Status> {
        self.run_explain_environment_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores an environment.
    async fn score_environment(
        &self,
        request: tonic::Request<configuration::ScoreEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::ScoreEnvironmentResponse>, tonic::Status> {
        self.run_score_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints an environment.
    async fn lint_environments(
        &self,
        request: tonic::Request<configuration::LintEnvironmentsRequest>,
    ) -> Result<tonic::Response<configuration::LintEnvironmentsResponse>, tonic::Status> {
        self.run_lint_environments(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an environment.
    async fn get_environment_rules(
        &self,
        request: tonic::Request<configuration::GetEnvironmentRulesRequest>,
    ) -> Result<tonic::Response<configuration::GetEnvironmentRulesResponse>, tonic::Status> {
        self.run_get_environment_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
