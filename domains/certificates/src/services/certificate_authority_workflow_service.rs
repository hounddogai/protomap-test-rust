use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_validation_service_client::PermissionCheckValidationServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_client::CertificateApprovalServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_workflow_service_server::CertificateAuthorityWorkflowService;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_workflow_service_client::CertificateWorkflowServiceClient;
use acme_proto::acme::certificates::v1::csr_approval_service_client::CsrApprovalServiceClient;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_client::ThrottleImportServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_workflow_service_client::ThrottleWorkflowServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Runs the long-lived workflow that takes a certificate authority from start to finish.
#[derive(Clone)]
pub struct CertificateAuthorityWorkflowController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
}
impl CertificateAuthorityWorkflowController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StartCertificateAuthorityWorkflow after validating the tenant and durably recording the command.
    async fn run_start_certificate_authority_workflow(
        &self,
        req: certificates::StartCertificateAuthorityWorkflowRequest,
    ) -> Result<certificates::StartCertificateAuthorityWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartCertificateAuthorityWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartCertificateAuthorityWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EscalateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            created_by: req
                                .certificate_authority
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            country_code: req
                                .certificate_authority
                                .clone()
                                .unwrap_or_default()
                                .location
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
        let reply_0 = async move {
            let mut client = CertificateAuthorityApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::EscalateCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.escalate_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::PauseCertificateWorkflowRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_by: reply_0
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
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateWorkflowServiceClient::new(channel);
            let input: tonic::Request<certificates::PauseCertificateWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pause_certificate_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = venues::BatchCreateVenuesRequest {
            context: req.context.clone(),
            chunk: Some(venues::batch_create_venues_request::Chunk::Record(venues::Venue {
                locator: Some(venues::venue::Locator::Address(common::PostalAddress {
                    region: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .region
                        .clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    created_at: reply_1
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_by: reply_1
                        .certificate
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
                    user_id: reply_0
                        .certificate_authority
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
            })),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::BatchCreateVenuesRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::StreamRateLimitChangesRequest {
            frame: Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(
                ratelimiting::RateLimit {
                    audit: Some(common::AuditInfo {
                        created_by: req
                            .certificate_authority
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        updated_at: req
                            .certificate_authority
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        updated_by: req
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
                    owner: Some(identity::UserRef {
                        email: req
                            .certificate_authority
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
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamRateLimitChangesRequest> =
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
            let response = client.stream_rate_limit_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamRateLimitChangesResponse::default();
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
        let mut response = certificates::StartCertificateAuthorityWorkflowResponse::default();
        let contribution: String = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: Option<common::PostalAddress> = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children[0]
                    .children
                    .push(Default::default());
            }
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children[0]
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children[0]
                    .children[0]
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children[0]
                .children[0]
                .location = contribution;
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: Option<identity::UserRef> =
            req.certificate_authority.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .manager = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceCertificateAuthorityWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_certificate_authority_workflow(
        &self,
        req: certificates::AdvanceCertificateAuthorityWorkflowRequest,
    ) -> Result<certificates::AdvanceCertificateAuthorityWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceCertificateAuthorityWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceCertificateAuthorityWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ExplainFlagEvaluationDecisionRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_flag_evaluation_decision_request::Details {
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
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: req
                        .certificate_authority
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExplainFlagEvaluationDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_flag_evaluation_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::AdvanceCertificateAuthorityWorkflowResponse::default();
        let contribution: String = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: String = req.certificate_authority.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .parent_id = contribution;
        }
        Ok(response)
    }
    /// Run CancelCertificateAuthorityWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_certificate_authority_workflow(
        &self,
        req: certificates::CancelCertificateAuthorityWorkflowRequest,
    ) -> Result<certificates::CancelCertificateAuthorityWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelCertificateAuthorityWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelCertificateAuthorityWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::ExplainCsrDecisionRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                audit: Some(common::AuditInfo {
                    updated_by: req
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
                requester: Some(common::ContactInfo {
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
            details: Some(certificates::explain_csr_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .certificate_authority
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
        let reply_0 = async move {
            let mut client = CsrValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ExplainCsrDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_csr_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::MapThrottleImportColumnsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::map_throttle_import_columns_request::Details {
                contact: Some(common::ContactInfo {
                    email: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .email
                    .clone(),
                    phone: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
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
        let reply_1 = async move {
            let mut client = ThrottleImportServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::MapThrottleImportColumnsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.map_throttle_import_columns(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = certificates::EscalateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    updated_at: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                counterparty: Some(certificates::CsrRef {
                    display_name: reply_1
                        .throttle
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::EscalateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.escalate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = certificates::CompareCertificateAuthorityOptionsRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
                            etag: reply_0
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
                                .clone(),
                            updated_by: reply_2
                                .certificate
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_by
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
                                .certificate_authority
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
            let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CompareCertificateAuthorityOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_certificate_authority_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::CancelCertificateAuthorityWorkflowResponse::default();
        let contribution: String = reply_0
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
                .certificate_authority
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = reply_2
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children[0]
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_3
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .locality
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
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
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: String = req.certificate_authority.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.certificate_authority.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run RetryCertificateAuthorityWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_certificate_authority_workflow(
        &self,
        req: certificates::RetryCertificateAuthorityWorkflowRequest,
    ) -> Result<certificates::RetryCertificateAuthorityWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryCertificateAuthorityWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryCertificateAuthorityWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ResumeThrottleWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::resume_throttle_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .manager
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
            throttle: Some(ratelimiting::Throttle {
                attributes: req.certificate_authority.clone().unwrap_or_default().attributes.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ResumeThrottleWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_throttle_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::GenerateRateLimitReportRequest {
            context: req.context.clone(),
            rate_limit_id: reply_0
                .throttle
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .rate_limit_id
                .clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::GenerateRateLimitReportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_rate_limit_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = certificates::SubmitCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            etag: req
                                .certificate_authority
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
                            country_code: req
                                .certificate_authority
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateAuthorityApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::SubmitCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.submit_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::RetryCertificateAuthorityWorkflowResponse::default();
        let contribution: String = reply_0
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children[0]
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children[0]
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = reply_2
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .locality
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: String = req.certificate_authority.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.certificate_authority.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run PauseCertificateAuthorityWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_certificate_authority_workflow(
        &self,
        req: certificates::PauseCertificateAuthorityWorkflowRequest,
    ) -> Result<certificates::PauseCertificateAuthorityWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PauseCertificateAuthorityWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseCertificateAuthorityWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ListConfigSetReportsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_config_set_reports_request::Filter {
                created_after: req
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: req
                    .certificate_authority
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = certificates::ApproveCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .certificate_authority
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigSetReportingServiceClient::new(channel_0);
                let input: tonic::Request<configuration::ListConfigSetReportsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_config_set_reports(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CertificateApprovalServiceClient::new(channel_1);
                let input: tonic::Request<certificates::ApproveCertificateRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.approve_certificate(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = certificates::ScoreCertificateRequest {
            certificate: Some(certificates::Certificate {
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        last4: (match reply_1.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .last4
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ScoreCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::PauseCertificateAuthorityWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .locality
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: i32 = req.certificate_authority.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .status = contribution;
        }
        Ok(response)
    }
    /// Run ResumeCertificateAuthorityWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_certificate_authority_workflow(
        &self,
        req: certificates::ResumeCertificateAuthorityWorkflowRequest,
    ) -> Result<certificates::ResumeCertificateAuthorityWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeCertificateAuthorityWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeCertificateAuthorityWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CheckCertificateEligibilityRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    recipient: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    region: req
                        .certificate_authority
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
            context: req.context.clone(),
            details: Some(certificates::check_certificate_eligibility_request::Details {
                contact: Some(common::ContactInfo {
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = certificates::RecallCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
                    user_id: req
                        .certificate_authority
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
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CertificateValidationServiceClient::new(channel_0);
                let input: tonic::Request<certificates::CheckCertificateEligibilityRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.check_certificate_eligibility(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CsrApprovalServiceClient::new(channel_1);
                let input: tonic::Request<certificates::RecallCsrRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.recall_csr(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = certificates::ResumeCertificateAuthorityWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_1
            .csr
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_1
            .csr
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children[0]
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: String = req.certificate_authority.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.certificate_authority.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run InspectCertificateAuthorityWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_certificate_authority_workflow(
        &self,
        req: certificates::InspectCertificateAuthorityWorkflowRequest,
    ) -> Result<certificates::InspectCertificateAuthorityWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectCertificateAuthorityWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectCertificateAuthorityWorkflow", "running workflow");
        let mut response = certificates::InspectCertificateAuthorityWorkflowResponse::default();
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        Ok(response)
    }
    /// Run ListCertificateAuthorityWorkflows after validating the tenant and durably recording the command.
    async fn run_list_certificate_authority_workflows(
        &self,
        req: certificates::ListCertificateAuthorityWorkflowsRequest,
    ) -> Result<certificates::ListCertificateAuthorityWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListCertificateAuthorityWorkflows", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListCertificateAuthorityWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = authorization::ValidatePermissionCheckRequest {
            context: req.context.clone(),
            details: Some(authorization::validate_permission_check_request::Details {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = PermissionCheckValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::ValidatePermissionCheckRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_permission_check(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::RecallCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateAuthorityApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::RecallCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recall_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ListCertificateAuthorityWorkflowsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.permission_check.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            response.certificate_authorities[0].attributes.extend(contribution);
        }
        let contribution: String = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .certificate_authority_id
            .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            if response.certificate_authorities[0].children.is_empty() {
                response.certificate_authorities[0].children.push(Default::default());
            }
            if response.certificate_authorities[0].children[0].children.is_empty() {
                response.certificate_authorities[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.certificate_authorities[0].children[0].children[0].certificate_authority_id = contribution;
        }
        let contribution: String = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            if response.certificate_authorities[0].children.is_empty() {
                response.certificate_authorities[0].children.push(Default::default());
            }
            if response.certificate_authorities[0].children[0].children.is_empty() {
                response.certificate_authorities[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.certificate_authorities[0].children[0].children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run SignalCertificateAuthorityWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_certificate_authority_workflow(
        &self,
        req: certificates::SignalCertificateAuthorityWorkflowRequest,
    ) -> Result<certificates::SignalCertificateAuthorityWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalCertificateAuthorityWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalCertificateAuthorityWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = certificates::PeekCertificateAuthorityRequest {
            certificate_authority_id: req
                .certificate_authority
                .clone()
                .unwrap_or_default()
                .certificate_authority_id
                .clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::RetryThrottleImportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::retry_throttle_import_request::Details {
                notify: Some(common::ContactInfo {
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CertificateAuthorityQueryServiceClient::new(channel_0);
                let input: tonic::Request<certificates::PeekCertificateAuthorityRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.peek_certificate_authority(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ThrottleImportServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::RetryThrottleImportRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.retry_throttle_import(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = ratelimiting::ListThrottleImportsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_throttle_imports_request::Filter {
                region: reply_0
                    .certificate_authority
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
            let mut client = ThrottleImportServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListThrottleImportsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_throttle_imports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::SignalCertificateAuthorityWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: Option<common::PostalAddress> =
            req.certificate_authority.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .location = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { CertificateAuthorityWorkflowService, CertificateAuthorityWorkflowController;
unary {
start_certificate_authority_workflow (certificates::StartCertificateAuthorityWorkflowRequest) -> certificates::StartCertificateAuthorityWorkflowResponse => run_start_certificate_authority_workflow;
advance_certificate_authority_workflow (certificates::AdvanceCertificateAuthorityWorkflowRequest) -> certificates::AdvanceCertificateAuthorityWorkflowResponse => run_advance_certificate_authority_workflow;
cancel_certificate_authority_workflow (certificates::CancelCertificateAuthorityWorkflowRequest) -> certificates::CancelCertificateAuthorityWorkflowResponse => run_cancel_certificate_authority_workflow;
retry_certificate_authority_workflow (certificates::RetryCertificateAuthorityWorkflowRequest) -> certificates::RetryCertificateAuthorityWorkflowResponse => run_retry_certificate_authority_workflow;
pause_certificate_authority_workflow (certificates::PauseCertificateAuthorityWorkflowRequest) -> certificates::PauseCertificateAuthorityWorkflowResponse => run_pause_certificate_authority_workflow;
resume_certificate_authority_workflow (certificates::ResumeCertificateAuthorityWorkflowRequest) -> certificates::ResumeCertificateAuthorityWorkflowResponse => run_resume_certificate_authority_workflow;
inspect_certificate_authority_workflow (certificates::InspectCertificateAuthorityWorkflowRequest) -> certificates::InspectCertificateAuthorityWorkflowResponse => run_inspect_certificate_authority_workflow;
list_certificate_authority_workflows (certificates::ListCertificateAuthorityWorkflowsRequest) -> certificates::ListCertificateAuthorityWorkflowsResponse => run_list_certificate_authority_workflows;
signal_certificate_authority_workflow (certificates::SignalCertificateAuthorityWorkflowRequest) -> certificates::SignalCertificateAuthorityWorkflowResponse => run_signal_certificate_authority_workflow;
}
server {
}
client {
}
bidi {
}
}
