use acme_proto::acme::accounting::v1 as accounting;
use acme_proto::acme::accounting::v1::chart_of_accounts_service_client::ChartOfAccountsServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_export_service_client::PermissionCheckExportServiceClient;
use acme_proto::acme::authorization::v1::role_binding_export_service_client::RoleBindingExportServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::csr_workflow_service_client::CsrWorkflowServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::experiments::v1::experiment_calculation_service_client::ExperimentCalculationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_lifecycle_service_server::CryptoKeyLifecycleService;
use acme_proto::acme::keymanagement::v1::crypto_key_service_client::CryptoKeyServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::subscriptions::v1 as subscriptions;
use acme_proto::acme::subscriptions::v1::subscription_service_client::SubscriptionServiceClient;

/// Moves a crypto key between the states of its lifecycle.
#[derive(Clone)]
pub struct CryptoKeyLifecycleApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    renew_crypto_key_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                keymanagement::CalculateKeyRingRequest,
                keymanagement::CalculateKeyRingResponse,
            >,
    >,
    renew_crypto_key_3_port:
        acme_platform::ports::Callback<keymanagement::NormalizeKeyRingRequest, keymanagement::NormalizeKeyRingResponse>,
    schedule_crypto_key_expiry_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::ScheduleEnvironmentReportRequest,
            configuration::ScheduleEnvironmentReportResponse,
        >,
    >,
}
impl CryptoKeyLifecycleApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::CalculateKeyRingRequest,
            keymanagement::CalculateKeyRingResponse,
        > = {
            let client = KeyRingCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::CalculateKeyRingRequest| -> acme_platform::ports::RpcFuture<keymanagement::CalculateKeyRingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_key_ring(request).await?.into_inner())
})
})
        };
        let renew_crypto_key_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::NormalizeKeyRingRequest,
            keymanagement::NormalizeKeyRingResponse,
        > = {
            let client = KeyRingValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::NormalizeKeyRingRequest| -> acme_platform::ports::RpcFuture<keymanagement::NormalizeKeyRingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_key_ring(request).await?.into_inner())
})
})
        };
        let renew_crypto_key_3_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::ScheduleEnvironmentReportRequest,
            configuration::ScheduleEnvironmentReportResponse,
        > = {
            let client = EnvironmentReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ScheduleEnvironmentReportRequest| -> acme_platform::ports::RpcFuture<configuration::ScheduleEnvironmentReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.schedule_environment_report(request).await?.into_inner())
})
})
        };
        let schedule_crypto_key_expiry_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            renew_crypto_key_2_port,
            renew_crypto_key_3_port,
            schedule_crypto_key_expiry_0_port,
        }
    }
    /// Run ActivateCryptoKey after validating the tenant and durably recording the command.
    async fn run_activate_crypto_key(
        &self,
        req: keymanagement::ActivateCryptoKeyRequest,
    ) -> Result<keymanagement::ActivateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ActivateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ActivateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = configuration::ListPendingConfigChangesRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_pending_config_changes_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::ScoreCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
                holder: Some(keymanagement::KeyRingRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeApprovalServiceClient::new(channel_0);
                let input: tonic::Request<configuration::ListPendingConfigChangesRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_pending_config_changes(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CryptoKeyValidationServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::ScoreCryptoKeyRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.score_crypto_key(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = accounting::BatchCreateChartOfAccountsesRequest {
            context: req.context.clone(),
            chunk: Some(accounting::batch_create_chart_of_accountses_request::Chunk::Record(
                accounting::ChartOfAccounts {
                    children: vec![accounting::ChartOfAccounts {
                        children: vec![accounting::ChartOfAccounts {
                            audit: Some(common::AuditInfo {
                                updated_at: reply_1
                                    .crypto_key
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
            let mut client = ChartOfAccountsServiceClient::new(channel);
            let input: tonic::Request<accounting::BatchCreateChartOfAccountsesRequest> =
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
            let response = client.batch_create_chart_of_accountses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::StreamThrottleChangesRequest {
            frame: Some(ratelimiting::stream_throttle_changes_request::Frame::Change(
                ratelimiting::Throttle {
                    correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamThrottleChangesRequest> =
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
            let response = client.stream_throttle_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamThrottleChangesResponse::default();
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
        let mut response = keymanagement::ActivateCryptoKeyResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run SuspendCryptoKey after validating the tenant and durably recording the command.
    async fn run_suspend_crypto_key(
        &self,
        req: keymanagement::SuspendCryptoKeyRequest,
    ) -> Result<keymanagement::SuspendCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuspendCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuspendCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::EstimateKeyRotationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                organizer: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.keymanagement.v1.EstimateKeyRotationRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::EstimateKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::SimulateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
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
            details: Some(keymanagement::simulate_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    recipient: reply_0
                        .key_rotation
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::SimulateCryptoKeyRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CryptoKeyCalculationServiceClient::new(channel);
                let input: tonic::Request<keymanagement::SimulateCryptoKeyRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.simulate_crypto_key(input).await?;
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
        let mut response = keymanagement::SuspendCryptoKeyResponse::default();
        let contribution: String = reply_0
            .key_rotation
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .phone
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .phone = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ReinstateCryptoKey after validating the tenant and durably recording the command.
    async fn run_reinstate_crypto_key(
        &self,
        req: keymanagement::ReinstateCryptoKeyRequest,
    ) -> Result<keymanagement::ReinstateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReinstateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReinstateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::CalculateKeyRotationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                participants: vec![keymanagement::KeyRotationParticipant {
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
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = certificates::RetryCsrWorkflowRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
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
            details: Some(certificates::retry_csr_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = KeyRotationCalculationServiceClient::new(channel_0);
                let input: tonic::Request<keymanagement::CalculateKeyRotationRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.calculate_key_rotation(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CsrWorkflowServiceClient::new(channel_1);
                let input: tonic::Request<certificates::RetryCsrWorkflowRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.retry_csr_workflow(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = certificates::ListCsrWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(certificates::list_csr_workflows_request::Filter {
                created_after: reply_0
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.annotations.clone(),
                owner_id: reply_1.csr.clone().unwrap_or_default().csr_id.clone(),
                region: reply_0
                    .key_rotation
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
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = CsrWorkflowServiceClient::new(channel);
            let input: tonic::Request<certificates::ListCsrWorkflowsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_csr_workflows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_3 = experiments::CalculateExperimentRequest {
            context: req.context.clone(),
            experiment: Some(experiments::Experiment {
                location: Some(common::PostalAddress {
                    country_code: reply_0
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    ..Default::default()
                }),
                organizer: Some(identity::UserRef {
                    display_name: reply_1
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .assignee
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                participants: vec![experiments::ExperimentParticipant {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ExperimentCalculationServiceClient::new(channel);
            let input: tonic::Request<experiments::CalculateExperimentRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_experiment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ReinstateCryptoKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0
            .key_rotation
            .clone()
            .unwrap_or_default()
            .organizer
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1
            .csr
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RenewCryptoKey after validating the tenant and durably recording the command.
    async fn run_renew_crypto_key(
        &self,
        req: keymanagement::RenewCryptoKeyRequest,
    ) -> Result<keymanagement::RenewCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RenewCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RenewCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFeatureFlagSubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_subscriptions_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req
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
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::SyncEnvironmentsRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = FeatureFlagNotificationServiceClient::new(channel_0);
                let input: tonic::Request<featureflags::ListFeatureFlagSubscriptionsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_feature_flag_subscriptions(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = EnvironmentSyncServiceClient::new(channel_1);
                let input: tonic::Request<configuration::SyncEnvironmentsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.sync_environments(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = keymanagement::CalculateKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_at: reply_1
                                .environment
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
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
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.renew_crypto_key_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = keymanagement::NormalizeKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            updated_at: reply_2
                                .key_ring
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
                            lines: reply_2
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone(),
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
                        manager: Some(identity::UserRef {
                            display_name: reply_1
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
            ..Default::default()
        };
        let reply_3 = (self.renew_crypto_key_3_port)(outgoing_3).await?;
        let mut response = keymanagement::RenewCryptoKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_2.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_3
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ExpireCryptoKey after validating the tenant and durably recording the command.
    async fn run_expire_crypto_key(
        &self,
        req: keymanagement::ExpireCryptoKeyRequest,
    ) -> Result<keymanagement::ExpireCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExpireCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExpireCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = authorization::SharePermissionCheckExportRequest {
            context: req.context.clone(),
            details: Some(authorization::share_permission_check_export_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::EstimateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
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
                holder: Some(keymanagement::KeyRingRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = PermissionCheckExportServiceClient::new(channel_0);
                let input: tonic::Request<authorization::SharePermissionCheckExportRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.share_permission_check_export(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CryptoKeyCalculationServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::EstimateCryptoKeyRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.estimate_crypto_key(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = keymanagement::ExpireCryptoKeyResponse::default();
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .phone
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .phone = contribution;
        }
        let contribution: String = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run GetCryptoKeyLifecycle after validating the tenant and durably recording the command.
    async fn run_get_crypto_key_lifecycle(
        &self,
        req: keymanagement::GetCryptoKeyLifecycleRequest,
    ) -> Result<keymanagement::GetCryptoKeyLifecycleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCryptoKeyLifecycle", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCryptoKeyLifecycle", "running workflow");
        let outgoing_0 = subscriptions::SearchSubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(subscriptions::search_subscriptions_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                owner_id: req.crypto_key_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<subscriptions::SearchSubscriptionsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SubscriptionServiceClient::new(channel);
            let input: tonic::Request<subscriptions::SearchSubscriptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_subscriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::WatchCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::watch_crypto_keys_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: req.crypto_key_id.clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
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
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = CryptoKeyServiceClient::new(channel);
            let input: tonic::Request<keymanagement::WatchCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_crypto_keys(input).await?;
            let mut frames = response.into_inner();
            let mut last = keymanagement::WatchCryptoKeysResponse::default();
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
        let mut response = keymanagement::GetCryptoKeyLifecycleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleCryptoKeyExpiry after validating the tenant and durably recording the command.
    async fn run_schedule_crypto_key_expiry(
        &self,
        req: keymanagement::ScheduleCryptoKeyExpiryRequest,
    ) -> Result<keymanagement::ScheduleCryptoKeyExpiryResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleCryptoKeyExpiry", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleCryptoKeyExpiry", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ScheduleEnvironmentReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.schedule_crypto_key_expiry_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = authorization::StreamRoleBindingExportRequest {
            context: req.context.clone(),
            filter: Some(authorization::stream_role_binding_export_request::Filter {
                created_after: req
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = RoleBindingExportServiceClient::new(channel);
            let input: tonic::Request<authorization::StreamRoleBindingExportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_role_binding_export(input).await?;
            let mut frames = response.into_inner();
            let mut last = authorization::StreamRoleBindingExportResponse::default();
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
        let outgoing_2 = keymanagement::ScoreCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                funding: Some(keymanagement::crypto_key::Funding::Bank(
                    keymanagement::CryptoKeyBankFunding {
                        account_holder: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .account_holder
                        .clone(),
                        bic: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .bic
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<keymanagement::ScoreCryptoKeyRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CryptoKeyValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ScoreCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_3 = req
            .crypto_key
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_2 = reply_2.clone();
                async move {
                    let outgoing_3 = keymanagement::ExplainCryptoKeyDecisionRequest {
                        context: req.context.clone(),
                        crypto_key: Some(keymanagement::CryptoKey {
                            contact: Some(common::ContactInfo {
                                preferred_channel: reply_2
                                    .crypto_key
                                    .clone()
                                    .unwrap_or_default()
                                    .contact
                                    .clone()
                                    .unwrap_or_default()
                                    .preferred_channel
                                    .clone(),
                                ..Default::default()
                            }),
                            limits: vec![keymanagement::CryptoKeyLimit {
                                amount: item.amount.clone(),
                                name: item.name.clone(),
                                period: item.period.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CryptoKeyValidationServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::ExplainCryptoKeyDecisionRequest> =
                            tonic::IntoRequest::into_request(outgoing_3);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.explain_crypto_key_decision(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_3).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_3 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::ScheduleCryptoKeyExpiryResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<common::Money> = req.crypto_key.clone().unwrap_or_default().available.clone();
        if contribution != <Option<common::Money>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).available = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl CryptoKeyLifecycleService for CryptoKeyLifecycleApi {
    /// Activates a crypto key.
    async fn activate_crypto_key(
        &self,
        request: tonic::Request<keymanagement::ActivateCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::ActivateCryptoKeyResponse>, tonic::Status> {
        self.run_activate_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Suspends a crypto key.
    async fn suspend_crypto_key(
        &self,
        request: tonic::Request<keymanagement::SuspendCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::SuspendCryptoKeyResponse>, tonic::Status> {
        self.run_suspend_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reinstates a crypto key.
    async fn reinstate_crypto_key(
        &self,
        request: tonic::Request<keymanagement::ReinstateCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::ReinstateCryptoKeyResponse>, tonic::Status> {
        self.run_reinstate_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Renews a crypto key.
    async fn renew_crypto_key(
        &self,
        request: tonic::Request<keymanagement::RenewCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::RenewCryptoKeyResponse>, tonic::Status> {
        self.run_renew_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Expires a crypto key.
    async fn expire_crypto_key(
        &self,
        request: tonic::Request<keymanagement::ExpireCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::ExpireCryptoKeyResponse>, tonic::Status> {
        self.run_expire_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a crypto key.
    async fn get_crypto_key_lifecycle(
        &self,
        request: tonic::Request<keymanagement::GetCryptoKeyLifecycleRequest>,
    ) -> Result<tonic::Response<keymanagement::GetCryptoKeyLifecycleResponse>, tonic::Status> {
        self.run_get_crypto_key_lifecycle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a crypto key.
    async fn schedule_crypto_key_expiry(
        &self,
        request: tonic::Request<keymanagement::ScheduleCryptoKeyExpiryRequest>,
    ) -> Result<tonic::Response<keymanagement::ScheduleCryptoKeyExpiryResponse>, tonic::Status> {
        self.run_schedule_crypto_key_expiry(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
