use acme_proto::acme::analytics::v1 as analytics;
use acme_proto::acme::analytics::v1::analytics_event_service_client::AnalyticsEventServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_client::CertificateApprovalServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_workflow_service_server::CertificateWorkflowService;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::certificates::v1::csr_lifecycle_service_client::CsrLifecycleServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::helpdesk::v1 as helpdesk;
use acme_proto::acme::helpdesk::v1::ticket_service_client::TicketServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_validation_service_client::MetricSeriesValidationServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;

/// Runs the long-lived workflow that takes a certificate from start to finish.
#[derive(Clone)]
pub struct CertificateWorkflowApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
}
impl CertificateWorkflowApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StartCertificateWorkflow after validating the tenant and durably recording the command.
    async fn run_start_certificate_workflow(
        &self,
        req: certificates::StartCertificateWorkflowRequest,
    ) -> Result<certificates::StartCertificateWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartCertificateWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartCertificateWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let jobs_0 = req
            .certificate
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = certificates::FindCertificateAuthoritiesRequest {
                        context: req.context.clone(),
                        filter: Some(certificates::find_certificate_authorities_request::Filter {
                            labels: item.attributes.clone(),
                            owner_id: item.line_id.clone(),
                            region: req
                                .certificate
                                .clone()
                                .unwrap_or_default()
                                .billing_address
                                .clone()
                                .unwrap_or_default()
                                .region
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CertificateAuthorityQueryServiceClient::new(channel);
                        let input: tonic::Request<certificates::FindCertificateAuthoritiesRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.find_certificate_authorities(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let jobs_1 = req
            .certificate
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = certificates::NormalizeCertificateAuthorityRequest {
                        certificate_authority: Some(certificates::CertificateAuthority {
                            children: vec![certificates::CertificateAuthority {
                                children: vec![certificates::CertificateAuthority {
                                    attributes: item.attributes.clone(),
                                    location: Some(common::PostalAddress {
                                        country_code: req
                                            .certificate
                                            .clone()
                                            .unwrap_or_default()
                                            .billing_address
                                            .clone()
                                            .unwrap_or_default()
                                            .country_code
                                            .clone(),
                                        recipient: req
                                            .certificate
                                            .clone()
                                            .unwrap_or_default()
                                            .billing_address
                                            .clone()
                                            .unwrap_or_default()
                                            .recipient
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
                    async move {
                        let mut client = CertificateAuthorityValidationServiceClient::new(channel);
                        let input: tonic::Request<certificates::NormalizeCertificateAuthorityRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.normalize_certificate_authority(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = featureflags::ResolveFeatureFlagConflictRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    created_by: reply_1
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: reply_1
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: reply_1
                        .certificate_authority
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ResolveFeatureFlagConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_feature_flag_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = analytics::BatchCreateAnalyticsEventsRequest {
            context: req.context.clone(),
            chunk: Some(analytics::batch_create_analytics_events_request::Chunk::Header(
                analytics::batch_create_analytics_events_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
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
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = AnalyticsEventServiceClient::new(channel);
            let input: tonic::Request<analytics::BatchCreateAnalyticsEventsRequest> =
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
            let response = client.batch_create_analytics_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = configuration::StreamEnvironmentChangesRequest {
            frame: Some(configuration::stream_environment_changes_request::Frame::Change(
                configuration::Environment {
                    locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                        postal_code: reply_1
                            .certificate_authority
                            .clone()
                            .unwrap_or_default()
                            .location
                            .clone()
                            .unwrap_or_default()
                            .postal_code
                            .clone(),
                        ..Default::default()
                    })),
                    audit: Some(common::AuditInfo {
                        created_by: reply_1
                            .certificate_authority
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        updated_by: reply_2
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
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamEnvironmentChangesRequest> =
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
            let response = client.stream_environment_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::StreamEnvironmentChangesResponse::default();
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
        let mut response = certificates::StartCertificateWorkflowResponse::default();
        let contribution: i32 = reply_2.feature_flag.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .priority = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.certificate.clone().unwrap_or_default().issued_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).issued_at = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceCertificateWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_certificate_workflow(
        &self,
        req: certificates::AdvanceCertificateWorkflowRequest,
    ) -> Result<certificates::AdvanceCertificateWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceCertificateWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceCertificateWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::TerminateCsrRequest {
            context: req.context.clone(),
            csr_id: req
                .certificate
                .clone()
                .unwrap_or_default()
                .counterparty
                .clone()
                .unwrap_or_default()
                .csr_id
                .clone(),
            details: Some(certificates::terminate_csr_request::Details {
                new_owner: Some(identity::UserRef {
                    display_name: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .counterparty
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
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
                notify_contact: Some(common::ContactInfo {
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CsrLifecycleServiceClient::new(channel);
            let input: tonic::Request<certificates::TerminateCsrRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.terminate_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .certificate
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = certificates::CheckCertificateEligibilityRequest {
                        certificate: Some(certificates::Certificate {
                            lines: vec![certificates::CertificateLine {
                                amount: item.amount.clone(),
                                tax: Some(certificates::CertificateLineTax {
                                    rate: item.tax.clone().unwrap_or_default().rate.clone(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CertificateValidationServiceClient::new(channel);
                        let input: tonic::Request<certificates::CheckCertificateEligibilityRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.check_certificate_eligibility(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = certificates::CountCertificatesRequest {
            context: req.context.clone(),
            filter: Some(certificates::count_certificates_request::Filter {
                created_after: req
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                    Some(certificates::csr::Origin::Customer(value)) => value,
                    _ => Default::default(),
                })
                .user_id
                .clone(),
                region: reply_1
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .billing_address
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
            let mut client = CertificateQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::CountCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = certificates::LintCertificateAuthoritiesRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        location: Some(common::PostalAddress {
                            country_code: reply_1
                                .certificate
                                .clone()
                                .unwrap_or_default()
                                .billing_address
                                .clone()
                                .unwrap_or_default()
                                .country_code
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
        let reply_3 = async move {
            let mut client = CertificateAuthorityValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::LintCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_certificate_authorities(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::AdvanceCertificateWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: String = req.certificate.clone().unwrap_or_default().number.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).number = contribution;
        }
        Ok(response)
    }
    /// Run RetryCertificateWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_certificate_workflow(
        &self,
        req: certificates::RetryCertificateWorkflowRequest,
    ) -> Result<certificates::RetryCertificateWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryCertificateWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryCertificateWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::SimulateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
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
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CsrCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::SimulateCsrRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::RetryCertificateWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<common::PostalAddress> =
            req.certificate.clone().unwrap_or_default().billing_address.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address = contribution;
        }
        Ok(response)
    }
    /// Run PauseCertificateWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_certificate_workflow(
        &self,
        req: certificates::PauseCertificateWorkflowRequest,
    ) -> Result<certificates::PauseCertificateWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PauseCertificateWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseCertificateWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = notifications::SearchNotificationsRequest {
            context: req.context.clone(),
            filter: Some(notifications::search_notifications_request::Filter {
                created_after: req.certificate.clone().unwrap_or_default().due_at.clone(),
                region: req
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .billing_address
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = NotificationServiceClient::new(channel);
            let input: tonic::Request<notifications::SearchNotificationsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_notifications(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::PauseCertificateWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<certificates::CsrRef> =
            req.certificate.clone().unwrap_or_default().counterparty.clone();
        if contribution != <Option<certificates::CsrRef>>::default() {
            response.certificate.get_or_insert_with(Default::default).counterparty = contribution;
        }
        Ok(response)
    }
    /// Run ResumeCertificateWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_certificate_workflow(
        &self,
        req: certificates::ResumeCertificateWorkflowRequest,
    ) -> Result<certificates::ResumeCertificateWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeCertificateWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeCertificateWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::ListPendingCertificatesRequest {
            context: req.context.clone(),
            filter: Some(certificates::list_pending_certificates_request::Filter {
                created_after: req.certificate.clone().unwrap_or_default().issued_at.clone(),
                region: req
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .billing_address
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::ListPendingCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = helpdesk::WatchTicketsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
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
        let _reply_1 = async move {
            let mut client = TicketServiceClient::new(channel);
            let input: tonic::Request<helpdesk::WatchTicketsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_tickets(input).await?;
            let mut frames = response.into_inner();
            let mut last = helpdesk::WatchTicketsResponse::default();
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
        let outgoing_2 = certificates::DelegateCertificateApprovalRequest {
            certificate: Some(certificates::Certificate {
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        holder_name: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .holder_name
                        .clone(),
                        network: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .network
                        .clone(),
                        ..Default::default()
                    },
                )),
                counterparty: Some(certificates::CsrRef {
                    csr_id: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .counterparty
                        .clone()
                        .unwrap_or_default()
                        .csr_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::DelegateCertificateApprovalRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.delegate_certificate_approval(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = certificates::NormalizeCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_by: reply_2
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    recipient: reply_2
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                }),
                counterparty: Some(certificates::CsrRef {
                    display_name: reply_2
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .counterparty
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
        let reply_3 = async move {
            let mut client = CertificateValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::NormalizeCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ResumeCertificateWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.certificate.clone().unwrap_or_default().certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: String = reply_2
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.certificate.clone().unwrap_or_default().notes.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).notes = contribution;
        }
        let contribution: i32 = req.certificate.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.certificate.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run InspectCertificateWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_certificate_workflow(
        &self,
        req: certificates::InspectCertificateWorkflowRequest,
    ) -> Result<certificates::InspectCertificateWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectCertificateWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectCertificateWorkflow", "running workflow");
        let outgoing_0 = featureflags::ValidateFlagRolloutRequest {
            context: req.context.clone(),
            details: Some(featureflags::validate_flag_rollout_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            flag_rollout: Some(featureflags::FlagRollout {
                organizer: Some(identity::UserRef {
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
                participants: vec![featureflags::FlagRolloutParticipant {
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
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ValidateFlagRolloutRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_flag_rollout(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = metrics::LintMetricSeriesesRequest {
            context: req.context.clone(),
            metric_series: Some(metrics::MetricSeries {
                locator: Some(metrics::metric_series::Locator::Address(common::PostalAddress {
                    lines: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = MetricSeriesValidationServiceClient::new(channel);
            let input: tonic::Request<metrics::LintMetricSeriesesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_metric_serieses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::InspectCertificateWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.flag_rollout.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = (match reply_1.metric_series.clone().unwrap_or_default().locator.clone() {
            Some(metrics::metric_series::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .lines
        .clone();
        if contribution != <Vec<String>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .lines
                .extend(contribution);
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        Ok(response)
    }
    /// Run ListCertificateWorkflows after validating the tenant and durably recording the command.
    async fn run_list_certificate_workflows(
        &self,
        req: certificates::ListCertificateWorkflowsRequest,
    ) -> Result<certificates::ListCertificateWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListCertificateWorkflows", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListCertificateWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::ScoreCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ScoreCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ListCertificateWorkflowsResponse::default();
        let contribution: String = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        if contribution != <String>::default() {
            if response.certificates.is_empty() {
                response.certificates.push(Default::default());
            }
            response.certificates[0]
                .billing_address
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.certificates.is_empty() {
                response.certificates.push(Default::default());
            }
            response.certificates[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        Ok(response)
    }
    /// Run SignalCertificateWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_certificate_workflow(
        &self,
        req: certificates::SignalCertificateWorkflowRequest,
    ) -> Result<certificates::SignalCertificateWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalCertificateWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalCertificateWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::SuspendCsrRequest {
            context: req.context.clone(),
            csr_id: req
                .certificate
                .clone()
                .unwrap_or_default()
                .counterparty
                .clone()
                .unwrap_or_default()
                .csr_id
                .clone(),
            details: Some(certificates::suspend_csr_request::Details {
                notify_contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CsrLifecycleServiceClient::new(channel);
            let input: tonic::Request<certificates::SuspendCsrRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suspend_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::RecallConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::recall_config_change_request::Details {
                approver: Some(identity::UserRef {
                    display_name: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .counterparty
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    user_id: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .assignee
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::RecallConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recall_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = certificates::ValidateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    recipient: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::validate_certificate_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .assignee
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
        let reply_2 = async move {
            let mut client = CertificateValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ValidateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::SignalCertificateWorkflowResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .csr
            .clone()
            .unwrap_or_default()
            .sla
            .clone()
            .unwrap_or_default()
            .due_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).due_at = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i32 = reply_2
            .certificate
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .priority
            .clone();
        if contribution != <i32>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .priority = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: String = req.certificate.clone().unwrap_or_default().notes.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).notes = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { CertificateWorkflowService, CertificateWorkflowApi;
unary {
start_certificate_workflow (certificates::StartCertificateWorkflowRequest) -> certificates::StartCertificateWorkflowResponse => run_start_certificate_workflow;
advance_certificate_workflow (certificates::AdvanceCertificateWorkflowRequest) -> certificates::AdvanceCertificateWorkflowResponse => run_advance_certificate_workflow;
retry_certificate_workflow (certificates::RetryCertificateWorkflowRequest) -> certificates::RetryCertificateWorkflowResponse => run_retry_certificate_workflow;
pause_certificate_workflow (certificates::PauseCertificateWorkflowRequest) -> certificates::PauseCertificateWorkflowResponse => run_pause_certificate_workflow;
resume_certificate_workflow (certificates::ResumeCertificateWorkflowRequest) -> certificates::ResumeCertificateWorkflowResponse => run_resume_certificate_workflow;
inspect_certificate_workflow (certificates::InspectCertificateWorkflowRequest) -> certificates::InspectCertificateWorkflowResponse => run_inspect_certificate_workflow;
list_certificate_workflows (certificates::ListCertificateWorkflowsRequest) -> certificates::ListCertificateWorkflowsResponse => run_list_certificate_workflows;
signal_certificate_workflow (certificates::SignalCertificateWorkflowRequest) -> certificates::SignalCertificateWorkflowResponse => run_signal_certificate_workflow;
}
server {
}
client {
}
bidi {
}
}
