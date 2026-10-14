use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_export_service_client::PermissionCheckExportServiceClient;
use acme_proto::acme::authorization::v1::permission_check_notification_service_client::PermissionCheckNotificationServiceClient;
use acme_proto::acme::authorization::v1::role_binding_query_service_client::RoleBindingQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient;
use acme_proto::acme::certificates::v1::certificate_workflow_service_client::CertificateWorkflowServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_client::FlagRolloutQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_ring_query_service_client::KeyRingQueryServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_workflow_service_client::KeyRingWorkflowServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_reporting_service_client::KeyRotationReportingServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_validation_service_client::KeyRotationValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_workflow_service_server::KeyRotationWorkflowService;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_validation_service_client::LedgerAccountValidationServiceClient;
use acme_proto::acme::maintenance::v1 as maintenance;
use acme_proto::acme::maintenance::v1::work_order_calculation_service_client::WorkOrderCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::throttle_approval_service_client::ThrottleApprovalServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Runs the long-lived workflow that takes a key rotation from start to finish.
#[derive(Clone)]
pub struct KeyRotationWorkflowEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
}
impl KeyRotationWorkflowEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StartKeyRotationWorkflow after validating the tenant and durably recording the command.
    async fn run_start_key_rotation_workflow(
        &self,
        req: keymanagement::StartKeyRotationWorkflowRequest,
    ) -> Result<keymanagement::StartKeyRotationWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartKeyRotationWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartKeyRotationWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::SharePermissionCheckExportRequest {
            context: req.context.clone(),
            details: Some(authorization::share_permission_check_export_request::Details {
                recipient: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = PermissionCheckExportServiceClient::new(channel);
            let input: tonic::Request<authorization::SharePermissionCheckExportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_permission_check_export(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::ExportFlagRolloutsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::export_flag_rollouts_request::Filter {
                region: req
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
        let _reply_1 = async move {
            let mut client = FlagRolloutQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExportFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_flag_rollouts(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::ExportFlagRolloutsResponse::default();
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
        let mut response = keymanagement::StartKeyRotationWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: String = req.key_rotation.clone().unwrap_or_default().title.clone();
        if contribution != <String>::default() {
            response.key_rotation.get_or_insert_with(Default::default).title = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceKeyRotationWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_key_rotation_workflow(
        &self,
        req: keymanagement::AdvanceKeyRotationWorkflowRequest,
    ) -> Result<keymanagement::AdvanceKeyRotationWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceKeyRotationWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceKeyRotationWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::StreamConfigChangeMetricsRequest {
            context: req.context.clone(),
            filter: Some(configuration::stream_config_change_metrics_request::Filter {
                region: req
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
            subscriber: Some(common::ContactInfo {
                name: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                phone: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamConfigChangeMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_config_change_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::StreamConfigChangeMetricsResponse::default();
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
        let outgoing_1 = certificates::SignalCertificateWorkflowRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    recipient: req
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
            context: req.context.clone(),
            details: Some(certificates::signal_certificate_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
            let mut client = CertificateWorkflowServiceClient::new(channel);
            let input: tonic::Request<certificates::SignalCertificateWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_certificate_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::AdvanceKeyRotationWorkflowResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: i32 = req.key_rotation.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.key_rotation.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run CancelKeyRotationWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_key_rotation_workflow(
        &self,
        req: keymanagement::CancelKeyRotationWorkflowRequest,
    ) -> Result<keymanagement::CancelKeyRotationWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelKeyRotationWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelKeyRotationWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::GenerateKeyRotationReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(keymanagement::generate_key_rotation_report_request::Details {
                filters: req.key_rotation.clone().unwrap_or_default().metadata.clone(),
                ..Default::default()
            }),
            key_rotation_id: req.key_rotation_id.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::GenerateKeyRotationReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_key_rotation_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::BrowseKeyRingsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::browse_key_rings_request::Filter {
                created_after: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .organizer
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
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
        let reply_1 = async move {
            let mut client = KeyRingQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::BrowseKeyRingsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_key_rings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_2 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_2 = authorization::ListPermissionCheckSubscriptionsRequest {
                        context: req.context.clone(),
                        filter: Some(authorization::list_permission_check_subscriptions_request::Filter {
                            owner_id: item.participant_id.clone(),
                            region: req
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
                    async move {
                        let mut client = PermissionCheckNotificationServiceClient::new(channel);
                        let input: tonic::Request<authorization::ListPermissionCheckSubscriptionsRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.list_permission_check_subscriptions(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::CancelKeyRotationWorkflowResponse::default();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: i32 = req.key_rotation.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.key_rotation.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run RetryKeyRotationWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_key_rotation_workflow(
        &self,
        req: keymanagement::RetryKeyRotationWorkflowRequest,
    ) -> Result<keymanagement::RetryKeyRotationWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryKeyRotationWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryKeyRotationWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let jobs_0 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = authorization::CountRoleBindingsRequest {
                        context: req.context.clone(),
                        filter: Some(authorization::count_role_bindings_request::Filter {
                            created_after: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            labels: req.key_rotation.clone().unwrap_or_default().metadata.clone(),
                            owner_id: item.participant_id.clone(),
                            region: req
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
                    async move {
                        let mut client = RoleBindingQueryServiceClient::new(channel);
                        let input: tonic::Request<authorization::CountRoleBindingsRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.count_role_bindings(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = keymanagement::ExplainKeyRotationDecisionRequest {
            context: req.context.clone(),
            details: Some(keymanagement::explain_key_rotation_decision_request::Details {
                contact: Some(common::ContactInfo {
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_rotation: Some(keymanagement::KeyRotation {
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .key_rotation
                            .clone()
                            .unwrap_or_default()
                            .organizer
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        phone: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify
                            .clone()
                            .unwrap_or_default()
                            .phone
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                recurrence: Some(keymanagement::key_rotation::Recurrence::Weekly(
                    keymanagement::KeyRotationWeeklyRule {
                        days: (match req.key_rotation.clone().unwrap_or_default().recurrence.clone() {
                            Some(keymanagement::key_rotation::Recurrence::Weekly(value)) => value,
                            _ => Default::default(),
                        })
                        .days
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = KeyRotationValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ExplainKeyRotationDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_key_rotation_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::RetryKeyRotationWorkflowResponse::default();
        let contribution: String = reply_1
            .key_rotation
            .clone()
            .unwrap_or_default()
            .organizer
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response
                .key_rotation
                .get_or_insert_with(Default::default)
                .participants
                .is_empty()
            {
                response
                    .key_rotation
                    .get_or_insert_with(Default::default)
                    .participants
                    .push(Default::default());
            }
            response.key_rotation.get_or_insert_with(Default::default).participants[0]
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.key_rotation.clone().unwrap_or_default().window.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.key_rotation.get_or_insert_with(Default::default).window = contribution;
        }
        Ok(response)
    }
    /// Run PauseKeyRotationWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_key_rotation_workflow(
        &self,
        req: keymanagement::PauseKeyRotationWorkflowRequest,
    ) -> Result<keymanagement::PauseKeyRotationWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PauseKeyRotationWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseKeyRotationWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let jobs_0 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = keymanagement::CancelKeyRingWorkflowRequest {
                        context: req.context.clone(),
                        key_ring: Some(keymanagement::KeyRing {
                            children: vec![keymanagement::KeyRing {
                                manager: Some(identity::UserRef {
                                    email: item.contact.clone().unwrap_or_default().email.clone(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = KeyRingWorkflowServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::CancelKeyRingWorkflowRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.cancel_key_ring_workflow(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::PauseKeyRotationWorkflowResponse::default();
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: String = req.key_rotation.clone().unwrap_or_default().title.clone();
        if contribution != <String>::default() {
            response.key_rotation.get_or_insert_with(Default::default).title = contribution;
        }
        Ok(response)
    }
    /// Run ResumeKeyRotationWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_key_rotation_workflow(
        &self,
        req: keymanagement::ResumeKeyRotationWorkflowRequest,
    ) -> Result<keymanagement::ResumeKeyRotationWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeKeyRotationWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeKeyRotationWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = venues::CloneVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                locator: Some(venues::venue::Locator::Address(common::PostalAddress {
                    lines: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    etag: req
                        .key_rotation
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::CloneVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ledger::LintLedgerAccountsRequest {
            context: req.context.clone(),
            details: Some(ledger::lint_ledger_accounts_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ledger_account: Some(ledger::LedgerAccount {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_by: reply_0
                        .venue
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = LedgerAccountValidationServiceClient::new(channel);
            let input: tonic::Request<ledger::LintLedgerAccountsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_ledger_accounts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ResumeKeyRotationWorkflowResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0
            .venue
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.key_rotation.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.key_rotation.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run InspectKeyRotationWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_key_rotation_workflow(
        &self,
        req: keymanagement::InspectKeyRotationWorkflowRequest,
    ) -> Result<keymanagement::InspectKeyRotationWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectKeyRotationWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectKeyRotationWorkflow", "running workflow");
        let outgoing_0 = keymanagement::ListKeyRingWorkflowsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::list_key_ring_workflows_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingWorkflowServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ListKeyRingWorkflowsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_key_ring_workflows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ledger::NormalizeLedgerAccountRequest {
            annotations: reply_0.result.clone().unwrap_or_default().details.clone(),
            context: req.context.clone(),
            details: Some(ledger::normalize_ledger_account_request::Details {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = LedgerAccountValidationServiceClient::new(channel);
            let input: tonic::Request<ledger::NormalizeLedgerAccountRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_ledger_account(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::InspectKeyRotationWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .ledger_account
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .preferred_channel
            .clone();
        if contribution != <String>::default() {
            if response
                .key_rotation
                .get_or_insert_with(Default::default)
                .participants
                .is_empty()
            {
                response
                    .key_rotation
                    .get_or_insert_with(Default::default)
                    .participants
                    .push(Default::default());
            }
            response.key_rotation.get_or_insert_with(Default::default).participants[0]
                .contact
                .get_or_insert_with(Default::default)
                .preferred_channel = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        Ok(response)
    }
    /// Run ListKeyRotationWorkflows after validating the tenant and durably recording the command.
    async fn run_list_key_rotation_workflows(
        &self,
        req: keymanagement::ListKeyRotationWorkflowsRequest,
    ) -> Result<keymanagement::ListKeyRotationWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListKeyRotationWorkflows", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListKeyRotationWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::RetryConfigChangeWorkflowRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    observed_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }],
                source: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::retry_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::RetryConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.retry_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ScheduleEnvironmentReportRequest {
            annotations: req.filter.clone().unwrap_or_default().labels.clone(),
            context: req.context.clone(),
            details: Some(configuration::schedule_environment_report_request::Details {
                filters: reply_0.config_change.clone().unwrap_or_default().attributes.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ScheduleEnvironmentReportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_environment_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ListKeyRotationWorkflowsResponse::default();
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
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run SignalKeyRotationWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_key_rotation_workflow(
        &self,
        req: keymanagement::SignalKeyRotationWorkflowRequest,
    ) -> Result<keymanagement::SignalKeyRotationWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalKeyRotationWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalKeyRotationWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = maintenance::RecalculateWorkOrdersRequest {
            context: req.context.clone(),
            details: Some(maintenance::recalculate_work_orders_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    locality: req
                        .key_rotation
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
            work_order: Some(maintenance::WorkOrder {
                assignee: Some(identity::UserRef {
                    display_name: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                requester: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = WorkOrderCalculationServiceClient::new(channel);
            let input: tonic::Request<maintenance::RecalculateWorkOrdersRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_work_orders(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::RecallThrottleRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
        let reply_1 = async move {
            let mut client = ThrottleApprovalServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RecallThrottleRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recall_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = certificates::ListPendingCertificateAuthoritiesRequest {
            context: req.context.clone(),
            filter: Some(certificates::list_pending_certificate_authorities_request::Filter {
                created_after: reply_0
                    .work_order
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                region: req
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
        let reply_2 = async move {
            let mut client = CertificateAuthorityApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::ListPendingCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_certificate_authorities(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::SignalKeyRotationWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.key_rotation.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.key_rotation.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { KeyRotationWorkflowService, KeyRotationWorkflowEndpoint;
unary {
start_key_rotation_workflow (keymanagement::StartKeyRotationWorkflowRequest) -> keymanagement::StartKeyRotationWorkflowResponse => run_start_key_rotation_workflow;
advance_key_rotation_workflow (keymanagement::AdvanceKeyRotationWorkflowRequest) -> keymanagement::AdvanceKeyRotationWorkflowResponse => run_advance_key_rotation_workflow;
cancel_key_rotation_workflow (keymanagement::CancelKeyRotationWorkflowRequest) -> keymanagement::CancelKeyRotationWorkflowResponse => run_cancel_key_rotation_workflow;
retry_key_rotation_workflow (keymanagement::RetryKeyRotationWorkflowRequest) -> keymanagement::RetryKeyRotationWorkflowResponse => run_retry_key_rotation_workflow;
pause_key_rotation_workflow (keymanagement::PauseKeyRotationWorkflowRequest) -> keymanagement::PauseKeyRotationWorkflowResponse => run_pause_key_rotation_workflow;
resume_key_rotation_workflow (keymanagement::ResumeKeyRotationWorkflowRequest) -> keymanagement::ResumeKeyRotationWorkflowResponse => run_resume_key_rotation_workflow;
inspect_key_rotation_workflow (keymanagement::InspectKeyRotationWorkflowRequest) -> keymanagement::InspectKeyRotationWorkflowResponse => run_inspect_key_rotation_workflow;
list_key_rotation_workflows (keymanagement::ListKeyRotationWorkflowsRequest) -> keymanagement::ListKeyRotationWorkflowsResponse => run_list_key_rotation_workflows;
signal_key_rotation_workflow (keymanagement::SignalKeyRotationWorkflowRequest) -> keymanagement::SignalKeyRotationWorkflowResponse => run_signal_key_rotation_workflow;
}
server {
}
client {
}
bidi {
}
}
