use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::concessions::v1::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_server::EnvironmentCalculationService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;
use acme_proto::acme::notifications::v1::notification_validation_service_client::NotificationValidationServiceClient;
use acme_proto::acme::prescriptions::v1 as prescriptions;
use acme_proto::acme::prescriptions::v1::prescription_service_client::PrescriptionServiceClient;

/// Computes the figures of an environment, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct EnvironmentCalculationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    calculate_environment_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<configuration::ScoreConfigSetRequest, configuration::ScoreConfigSetResponse>,
    >,
    compare_environment_options_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::SimulateConfigChangeRequest,
            configuration::SimulateConfigChangeResponse,
        >,
    >,
    estimate_environment_0_port: acme_platform::ports::Callback<
        notifications::ExplainNotificationDecisionRequest,
        notifications::ExplainNotificationDecisionResponse,
    >,
}
impl EnvironmentCalculationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::ScoreConfigSetRequest,
            configuration::ScoreConfigSetResponse,
        > = {
            let client = ConfigSetValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ScoreConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::ScoreConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_config_set(request).await?.into_inner())
})
})
        };
        let calculate_environment_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            notifications::ExplainNotificationDecisionRequest,
            notifications::ExplainNotificationDecisionResponse,
        > = {
            let client = NotificationValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: notifications::ExplainNotificationDecisionRequest| -> acme_platform::ports::RpcFuture<notifications::ExplainNotificationDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_notification_decision(request).await?.into_inner())
})
})
        };
        let estimate_environment_0_port = callback;
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
        let compare_environment_options_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            calculate_environment_0_port,
            compare_environment_options_0_port,
            estimate_environment_0_port,
        }
    }
    /// Run CalculateEnvironment after validating the tenant and durably recording the command.
    async fn run_calculate_environment(
        &self,
        req: configuration::CalculateEnvironmentRequest,
    ) -> Result<configuration::CalculateEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ScoreConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            postal_code: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .postal_code
                                .clone(),
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
                    }],
                    location: Some(common::PostalAddress {
                        locality: (match req.environment.clone().unwrap_or_default().locator.clone() {
                            Some(configuration::environment::Locator::Address(value)) => value,
                            _ => Default::default(),
                        })
                        .locality
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
        let reply_0 = acme_platform::ports::execute(self.calculate_environment_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = prescriptions::BatchCreatePrescriptionsRequest {
            context: req.context.clone(),
            chunk: Some(prescriptions::batch_create_prescriptions_request::Chunk::Record(
                prescriptions::Prescription {
                    billing_address: Some(common::PostalAddress {
                        recipient: (match req.environment.clone().unwrap_or_default().locator.clone() {
                            Some(configuration::environment::Locator::Address(value)) => value,
                            _ => Default::default(),
                        })
                        .recipient
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
            let mut client = PrescriptionServiceClient::new(channel);
            let input: tonic::Request<prescriptions::BatchCreatePrescriptionsRequest> =
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
            let response = client.batch_create_prescriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    audit: Some(common::AuditInfo {
                        etag: reply_0
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .etag
                            .clone(),
                        updated_at: reply_0
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
                    location: Some(common::PostalAddress {
                        locality: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .destination
                            .clone()
                            .unwrap_or_default()
                            .locality
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
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
        let mut response = configuration::CalculateEnvironmentResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: String = req.environment.clone().unwrap_or_default().code.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).code = contribution;
        }
        Ok(response)
    }
    /// Run EstimateEnvironment after validating the tenant and durably recording the command.
    async fn run_estimate_environment(
        &self,
        req: configuration::EstimateEnvironmentRequest,
    ) -> Result<configuration::EstimateEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = notifications::ExplainNotificationDecisionRequest {
            context: req.context.clone(),
            notification: Some(notifications::Notification {
                assignee: Some(identity::UserRef {
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
                origin: Some(notifications::notification::Origin::Customer(
                    notifications::NotificationTemplateRef {
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
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.estimate_environment_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = configuration::ValidateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            recipient: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .recipient
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
                            email: reply_0
                                .notification
                                .clone()
                                .unwrap_or_default()
                                .requester
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
        let outgoing_1 = acme_platform::ports::RequestBuilder::<configuration::ValidateConfigSetRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ValidateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::EstimateEnvironmentResponse::default();
        let contribution: String = reply_0
            .notification
            .clone()
            .unwrap_or_default()
            .requester
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
        let contribution: String = reply_0
            .notification
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run QuoteEnvironment after validating the tenant and durably recording the command.
    async fn run_quote_environment(
        &self,
        req: configuration::QuoteEnvironmentRequest,
    ) -> Result<configuration::QuoteEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ValidateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.ValidateConfigSetRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ValidateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = concessions::SimulateConcessionStandRequest {
            concession_stand: Some(concessions::ConcessionStand {
                locator: Some(concessions::concession_stand::Locator::Address(common::PostalAddress {
                    country_code: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<concessions::SimulateConcessionStandRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConcessionStandCalculationServiceClient::new(channel);
                let input: tonic::Request<concessions::SimulateConcessionStandRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.simulate_concession_stand(input).await?;
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
        let mut response = configuration::QuoteEnvironmentResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .config_set
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
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_1
            .concession_stand
            .clone()
            .unwrap_or_default()
            .category_path
            .clone();
        if contribution != <Vec<String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .category_path
                .extend(contribution);
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
    /// Run CompareEnvironmentOptions after validating the tenant and durably recording the command.
    async fn run_compare_environment_options(
        &self,
        req: configuration::CompareEnvironmentOptionsRequest,
    ) -> Result<configuration::CompareEnvironmentOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareEnvironmentOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareEnvironmentOptions", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SimulateConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::simulate_config_change_request::Details {
                destination: Some(common::PostalAddress {
                    locality: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .locality
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.compare_environment_options_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = configuration::CompareEnvironmentOptionsResponse::default();
        let contribution: String = reply_0
            .config_change
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_0.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateEnvironments after validating the tenant and durably recording the command.
    async fn run_recalculate_environments(
        &self,
        req: configuration::RecalculateEnvironmentsRequest,
    ) -> Result<configuration::RecalculateEnvironmentsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecalculateEnvironments", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateEnvironments", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ValidateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
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
                            region: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .region
                            .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    location: Some(common::PostalAddress {
                        locality: (match req.environment.clone().unwrap_or_default().locator.clone() {
                            Some(configuration::environment::Locator::Address(value)) => value,
                            _ => Default::default(),
                        })
                        .locality
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
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ValidateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = notifications::NormalizeNotificationRequest {
            context: req.context.clone(),
            details: Some(notifications::normalize_notification_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .environment
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
            }),
            notification: Some(notifications::Notification {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: reply_0
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
                requester: Some(common::ContactInfo {
                    email: reply_0
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = NotificationValidationServiceClient::new(channel);
            let input: tonic::Request<notifications::NormalizeNotificationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_notification(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::RecalculateEnvironmentsResponse::default();
        let contribution: String = reply_0
            .config_set
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
        let contribution: String = reply_1
            .notification
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run SimulateEnvironment after validating the tenant and durably recording the command.
    async fn run_simulate_environment(
        &self,
        req: configuration::SimulateEnvironmentRequest,
    ) -> Result<configuration::SimulateEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = notifications::CreateNotificationRequest {
            context: req.context.clone(),
            notification: Some(notifications::Notification {
                assignee: Some(identity::UserRef {
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
                requester: Some(common::ContactInfo {
                    email: req
                        .environment
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = NotificationServiceClient::new(channel);
            let input: tonic::Request<notifications::CreateNotificationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_notification(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::SimulateEnvironmentResponse::default();
        let contribution: String = reply_0
            .notification
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_0
            .notification
            .clone()
            .unwrap_or_default()
            .requester
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
    /// Run GetEnvironmentRates after validating the tenant and durably recording the command.
    async fn run_get_environment_rates(
        &self,
        req: configuration::GetEnvironmentRatesRequest,
    ) -> Result<configuration::GetEnvironmentRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetEnvironmentRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetEnvironmentRates", "running workflow");
        let command_0 = GetEnvironmentRatesDispatch0::from(req.clone());
        let outgoing_0: configuration::NormalizeConfigSetRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::NormalizeConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::GetEnvironmentRatesResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by GetEnvironmentRates for NormalizeConfigSet.
struct GetEnvironmentRatesDispatch0 {
    context: Option<common::RequestContext>,
    config_set_children_children_manager_user_id: String,
}
impl From<configuration::GetEnvironmentRatesRequest> for GetEnvironmentRatesDispatch0 {
    fn from(req: configuration::GetEnvironmentRatesRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_children_manager_user_id: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
        }
    }
}
impl From<GetEnvironmentRatesDispatch0> for configuration::NormalizeConfigSetRequest {
    fn from(value: GetEnvironmentRatesDispatch0) -> Self {
        configuration::NormalizeConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            user_id: value.config_set_children_children_manager_user_id,
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
impl EnvironmentCalculationService for EnvironmentCalculationHandler {
    /// Calculates an environment.
    async fn calculate_environment(
        &self,
        request: tonic::Request<configuration::CalculateEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::CalculateEnvironmentResponse>, tonic::Status> {
        self.run_calculate_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates an environment.
    async fn estimate_environment(
        &self,
        request: tonic::Request<configuration::EstimateEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::EstimateEnvironmentResponse>, tonic::Status> {
        self.run_estimate_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes an environment.
    async fn quote_environment(
        &self,
        request: tonic::Request<configuration::QuoteEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::QuoteEnvironmentResponse>, tonic::Status> {
        self.run_quote_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares an environment.
    async fn compare_environment_options(
        &self,
        request: tonic::Request<configuration::CompareEnvironmentOptionsRequest>,
    ) -> Result<tonic::Response<configuration::CompareEnvironmentOptionsResponse>, tonic::Status> {
        self.run_compare_environment_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates an environment.
    async fn recalculate_environments(
        &self,
        request: tonic::Request<configuration::RecalculateEnvironmentsRequest>,
    ) -> Result<tonic::Response<configuration::RecalculateEnvironmentsResponse>, tonic::Status> {
        self.run_recalculate_environments(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates an environment.
    async fn simulate_environment(
        &self,
        request: tonic::Request<configuration::SimulateEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::SimulateEnvironmentResponse>, tonic::Status> {
        self.run_simulate_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an environment.
    async fn get_environment_rates(
        &self,
        request: tonic::Request<configuration::GetEnvironmentRatesRequest>,
    ) -> Result<tonic::Response<configuration::GetEnvironmentRatesResponse>, tonic::Status> {
        self.run_get_environment_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
