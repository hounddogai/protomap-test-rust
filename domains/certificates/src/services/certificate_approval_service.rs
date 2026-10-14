use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::catalog::v1 as catalog;
use acme_proto::acme::catalog::v1::product_calculation_service_client::ProductCalculationServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_server::CertificateApprovalService;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::customers::v1 as customers;
use acme_proto::acme::expenses::v1 as expenses;
use acme_proto::acme::expenses::v1::expense_report_service_client::ExpenseReportServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::fulfillment::v1::fulfillment_query_service_client::FulfillmentQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_service_client::UserServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_approval_service_client::RateLimitApprovalServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Routes certificates through approval, with escalation and delegation.
#[derive(Clone)]
pub struct CertificateApprovalHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
}
impl CertificateApprovalHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SubmitCertificate after validating the tenant and durably recording the command.
    async fn run_submit_certificate(
        &self,
        req: certificates::SubmitCertificateRequest,
    ) -> Result<certificates::SubmitCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CountCertificateAuthoritiesRequest {
            context: req.context.clone(),
            filter: Some(certificates::count_certificate_authorities_request::Filter {
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
            let mut client = CertificateAuthorityQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::CountCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_certificate_authorities(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ListPendingRateLimitsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_pending_rate_limits_request::Filter {
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
        let reply_1 = async move {
            let mut client = RateLimitApprovalServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListPendingRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = audit::BatchCreateAuditEventsRequest {
            context: req.context.clone(),
            chunk: Some(audit::batch_create_audit_events_request::Chunk::Record(
                audit::AuditEvent {
                    correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                    subject: Some(audit::AuditTrailRef {
                        display_name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .approver
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        number: req.certificate.clone().unwrap_or_default().number.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::BatchCreateAuditEventsRequest> =
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
            let response = client.batch_create_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        created_by: req
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        updated_at: req
                            .certificate
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
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
        let mut response = certificates::SubmitCertificateResponse::default();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run ApproveCertificate after validating the tenant and durably recording the command.
    async fn run_approve_certificate(
        &self,
        req: certificates::ApproveCertificateRequest,
    ) -> Result<certificates::ApproveCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = catalog::CompareProductOptionsRequest {
            context: req.context.clone(),
            product: Some(catalog::Product {
                audit: Some(common::AuditInfo {
                    created_by: req
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ProductCalculationServiceClient::new(channel);
            let input: tonic::Request<catalog::CompareProductOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_product_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = identity::GetUserRequest {
            context: req.context.clone(),
            user_id: req
                .details
                .clone()
                .unwrap_or_default()
                .approver
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = UserServiceClient::new(channel);
            let input: tonic::Request<identity::GetUserRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_user(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ApproveCertificateResponse::default();
        let contribution: String = reply_0
            .product
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .user
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
        let contribution: Vec<String> = reply_1
            .user
            .clone()
            .unwrap_or_default()
            .primary_address
            .clone()
            .unwrap_or_default()
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
        let contribution: Option<certificates::CsrRef> =
            req.certificate.clone().unwrap_or_default().counterparty.clone();
        if contribution != <Option<certificates::CsrRef>>::default() {
            response.certificate.get_or_insert_with(Default::default).counterparty = contribution;
        }
        Ok(response)
    }
    /// Run RejectCertificate after validating the tenant and durably recording the command.
    async fn run_reject_certificate(
        &self,
        req: certificates::RejectCertificateRequest,
    ) -> Result<certificates::RejectCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::QuoteCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
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
                        location: Some(common::PostalAddress {
                            postal_code: req
                                .certificate
                                .clone()
                                .unwrap_or_default()
                                .billing_address
                                .clone()
                                .unwrap_or_default()
                                .postal_code
                                .clone(),
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
                        manager: Some(identity::UserRef {
                            user_id: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .approver
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::QuoteCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::StreamFlagEvaluationExportRequest {
            context: req.context.clone(),
            filter: Some(featureflags::stream_flag_evaluation_export_request::Filter {
                created_after: req
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
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
        let _reply_1 = async move {
            let mut client = FlagEvaluationExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationExportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_flag_evaluation_export(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagEvaluationExportResponse::default();
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
        let outgoing_2 = venues::ExplainVenueDecisionRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::ExplainVenueDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_venue_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::RejectCertificateResponse::default();
        let contribution: String = reply_0
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
        let contribution: String = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: String = reply_2
            .venue
            .clone()
            .unwrap_or_default()
            .components
            .clone()
            .get("primary")
            .cloned()
            .unwrap_or_default()
            .description
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate
                .get_or_insert_with(Default::default)
                .lines
                .is_empty()
            {
                response
                    .certificate
                    .get_or_insert_with(Default::default)
                    .lines
                    .push(Default::default());
            }
            response.certificate.get_or_insert_with(Default::default).lines[0].description = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run EscalateCertificate after validating the tenant and durably recording the command.
    async fn run_escalate_certificate(
        &self,
        req: certificates::EscalateCertificateRequest,
    ) -> Result<certificates::EscalateCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::ResolveCertificatesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::ResolveCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ScheduleConfigChangeExpiryRequest {
            config_change: Some(configuration::ConfigChange {
                correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::schedule_config_change_expiry_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
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
            let mut client = ConfigChangeLifecycleServiceClient::new(channel);
            let input: tonic::Request<configuration::ScheduleConfigChangeExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_config_change_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = certificates::SimulateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            updated_at: req
                                .certificate
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
            let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::SimulateCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::EscalateCertificateResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
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
                .certificate
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run ListPendingCertificates after validating the tenant and durably recording the command.
    async fn run_list_pending_certificates(
        &self,
        req: certificates::ListPendingCertificatesRequest,
    ) -> Result<certificates::ListPendingCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPendingCertificates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::RecalculateCertificatesRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: req.context.clone().unwrap_or_default().deadline.clone(),
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
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::RecalculateCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::RecallRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::recall_rate_limit_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_0
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
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    etag: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_at: reply_0
                        .certificate
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RateLimitApprovalServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RecallRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recall_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ListPendingCertificatesResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.certificate.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.certificates.is_empty() {
                response.certificates.push(Default::default());
            }
            response.certificates[0].due_at = contribution;
        }
        let contribution: Option<common::Money> =
            (match reply_1.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                _ => Default::default(),
            })
            .amount
            .clone();
        if contribution != <Option<common::Money>>::default() {
            if response.certificates.is_empty() {
                response.certificates.push(Default::default());
            }
            if response.certificates[0].lines.is_empty() {
                response.certificates[0].lines.push(Default::default());
            }
            response.certificates[0].lines[0]
                .tax
                .get_or_insert_with(Default::default)
                .amount = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run DelegateCertificateApproval after validating the tenant and durably recording the command.
    async fn run_delegate_certificate_approval(
        &self,
        req: certificates::DelegateCertificateApprovalRequest,
    ) -> Result<certificates::DelegateCertificateApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateCertificateApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateCertificateApproval", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::GenerateConfigChangeReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::GenerateConfigChangeReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_config_change_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::ScoreCertificateRequest {
            certificate: Some(certificates::Certificate {
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        last4: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .last4
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
                    priority: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .counterparty
                        .clone()
                        .unwrap_or_default()
                        .priority
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
            let mut client = CertificateValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ScoreCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::DelegateCertificateApprovalResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.certificate.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).due_at = contribution;
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
    /// Run RecallCertificate after validating the tenant and durably recording the command.
    async fn run_recall_certificate(
        &self,
        req: certificates::RecallCertificateRequest,
    ) -> Result<certificates::RecallCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = expenses::CloneExpenseReportRequest {
            context: req.context.clone(),
            expense_report: Some(expenses::ExpenseReport {
                settlement: Some(expenses::expense_report::Settlement::Bank(
                    expenses::ExpenseReportBankSettlement {
                        account_holder: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .account_holder
                        .clone(),
                        bic: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .bic
                        .clone(),
                        ..Default::default()
                    },
                )),
                counterparty: Some(customers::CustomerRef {
                    display_name: req
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ExpenseReportServiceClient::new(channel);
            let input: tonic::Request<expenses::CloneExpenseReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_expense_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::QuoteCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        location: Some(common::PostalAddress {
                            postal_code: req
                                .certificate
                                .clone()
                                .unwrap_or_default()
                                .billing_address
                                .clone()
                                .unwrap_or_default()
                                .postal_code
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
            let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::QuoteCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::RecallCertificateResponse::default();
        let contribution: String = reply_0
            .expense_report
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
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
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
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.certificate.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).due_at = contribution;
        }
        Ok(response)
    }
    /// Run GetCertificateApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_certificate_approval_status(
        &self,
        req: certificates::GetCertificateApprovalStatusRequest,
    ) -> Result<certificates::GetCertificateApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetCertificateApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCertificateApprovalStatus", "running workflow");
        let outgoing_0 = certificates::NormalizeCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateAuthorityValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::NormalizeCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = fulfillment::ExportFulfillmentsRequest {
            context: req.context.clone(),
            filter: Some(fulfillment::export_fulfillments_request::Filter {
                created_after: reply_0.outcome.clone().unwrap_or_default().effective_at.clone(),
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
        let _reply_1 = async move {
            let mut client = FulfillmentQueryServiceClient::new(channel);
            let input: tonic::Request<fulfillment::ExportFulfillmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_fulfillments(input).await?;
            let mut frames = response.into_inner();
            let mut last = fulfillment::ExportFulfillmentsResponse::default();
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
        let mut response = certificates::GetCertificateApprovalStatusResponse::default();
        let contribution: Vec<String> = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
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
}
acme_platform::endpoints! { CertificateApprovalService, CertificateApprovalHandler;
unary {
submit_certificate (certificates::SubmitCertificateRequest) -> certificates::SubmitCertificateResponse => run_submit_certificate;
approve_certificate (certificates::ApproveCertificateRequest) -> certificates::ApproveCertificateResponse => run_approve_certificate;
reject_certificate (certificates::RejectCertificateRequest) -> certificates::RejectCertificateResponse => run_reject_certificate;
escalate_certificate (certificates::EscalateCertificateRequest) -> certificates::EscalateCertificateResponse => run_escalate_certificate;
list_pending_certificates (certificates::ListPendingCertificatesRequest) -> certificates::ListPendingCertificatesResponse => run_list_pending_certificates;
delegate_certificate_approval (certificates::DelegateCertificateApprovalRequest) -> certificates::DelegateCertificateApprovalResponse => run_delegate_certificate_approval;
recall_certificate (certificates::RecallCertificateRequest) -> certificates::RecallCertificateResponse => run_recall_certificate;
get_certificate_approval_status (certificates::GetCertificateApprovalStatusRequest) -> certificates::GetCertificateApprovalStatusResponse => run_get_certificate_approval_status;
}
server {
}
client {
}
bidi {
}
}
