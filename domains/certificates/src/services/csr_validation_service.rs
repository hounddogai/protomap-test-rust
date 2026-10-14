use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::certificates::v1::csr_validation_service_server::CsrValidationService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_query_service_client::ConfigSetQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_service_client::DocumentServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_client::FeatureFlagHistoryServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::prescriptions::v1 as prescriptions;
use acme_proto::acme::prescriptions::v1::prescription_service_client::PrescriptionServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;

/// Checks csrs against the rules before anything acts on them.
#[derive(Clone)]
pub struct CsrValidationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    explain_csr_decision_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<documents::BatchGetDocumentsRequest, documents::BatchGetDocumentsResponse>,
    >,
    explain_csr_decision_1_port: acme_platform::ports::Callback<
        configuration::BatchGetConfigSetsRequest,
        configuration::BatchGetConfigSetsResponse,
    >,
    explain_csr_decision_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            featureflags::ShareFlagEvaluationExportRequest,
            featureflags::ShareFlagEvaluationExportResponse,
        >,
    >,
    explain_csr_decision_3_port: acme_platform::ports::Callback<
        certificates::PreviewCertificateRequest,
        certificates::PreviewCertificateResponse,
    >,
    preview_csr_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            certificates::PreviewCertificateRequest,
            certificates::PreviewCertificateResponse,
        >,
    >,
    score_csr_0_port:
        acme_platform::ports::Callback<documents::CreateDocumentRequest, documents::CreateDocumentResponse>,
    score_csr_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            certificates::CalculateCertificateRequest,
            certificates::CalculateCertificateResponse,
        >,
    >,
}
impl CsrValidationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            certificates::PreviewCertificateRequest,
            certificates::PreviewCertificateResponse,
        > = {
            let client = CertificateValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::PreviewCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::PreviewCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_certificate(request).await?.into_inner())
})
})
        };
        let preview_csr_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            documents::BatchGetDocumentsRequest,
            documents::BatchGetDocumentsResponse,
        > = {
            let client = DocumentServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: documents::BatchGetDocumentsRequest| -> acme_platform::ports::RpcFuture<documents::BatchGetDocumentsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.batch_get_documents(request).await?.into_inner())
})
})
        };
        let explain_csr_decision_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::BatchGetConfigSetsRequest,
            configuration::BatchGetConfigSetsResponse,
        > = {
            let client = ConfigSetServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::BatchGetConfigSetsRequest| -> acme_platform::ports::RpcFuture<configuration::BatchGetConfigSetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.batch_get_config_sets(request).await?.into_inner())
})
})
        };
        let explain_csr_decision_1_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::ShareFlagEvaluationExportRequest,
            featureflags::ShareFlagEvaluationExportResponse,
        > = {
            let client = FlagEvaluationExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ShareFlagEvaluationExportRequest| -> acme_platform::ports::RpcFuture<featureflags::ShareFlagEvaluationExportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_flag_evaluation_export(request).await?.into_inner())
})
})
        };
        let explain_csr_decision_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            certificates::PreviewCertificateRequest,
            certificates::PreviewCertificateResponse,
        > = {
            let client = CertificateValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::PreviewCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::PreviewCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_certificate(request).await?.into_inner())
})
})
        };
        let explain_csr_decision_3_port = callback;
        let callback: acme_platform::ports::Callback<
            documents::CreateDocumentRequest,
            documents::CreateDocumentResponse,
        > = {
            let client = DocumentServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: documents::CreateDocumentRequest| -> acme_platform::ports::RpcFuture<documents::CreateDocumentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.create_document(request).await?.into_inner())
})
})
        };
        let score_csr_0_port = callback;
        let callback: acme_platform::ports::Callback<
            certificates::CalculateCertificateRequest,
            certificates::CalculateCertificateResponse,
        > = {
            let client = CertificateCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CalculateCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::CalculateCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_certificate(request).await?.into_inner())
})
})
        };
        let score_csr_1_port = std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            explain_csr_decision_0_port,
            explain_csr_decision_1_port,
            explain_csr_decision_2_port,
            explain_csr_decision_3_port,
            preview_csr_1_port,
            score_csr_0_port,
            score_csr_1_port,
        }
    }
    /// Run ValidateCsr after validating the tenant and durably recording the command.
    async fn run_validate_csr(
        &self,
        req: certificates::ValidateCsrRequest,
    ) -> Result<certificates::ValidateCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_at: req
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
                    display_name: req
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
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<certificates::EstimateCertificateRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::EstimateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = prescriptions::BatchCreatePrescriptionsRequest {
            context: req.context.clone(),
            chunk: Some(prescriptions::batch_create_prescriptions_request::Chunk::Record(
                prescriptions::Prescription {
                    settlement: Some(prescriptions::prescription::Settlement::Card(
                        prescriptions::PrescriptionCardSettlement {
                            network: (match reply_0.certificate.clone().unwrap_or_default().settlement.clone() {
                                Some(certificates::certificate::Settlement::Card(value)) => value,
                                _ => Default::default(),
                            })
                            .network
                            .clone(),
                            ..Default::default()
                        },
                    )),
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
                        created_by: reply_0
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
                    organizer: Some(identity::UserRef {
                        email: req
                            .csr
                            .clone()
                            .unwrap_or_default()
                            .requester
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    }),
                    participants: vec![featureflags::FlagRolloutParticipant {
                        contact: Some(common::ContactInfo {
                            email: req
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .assignee
                                .clone()
                                .unwrap_or_default()
                                .email
                                .clone(),
                            name: req
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .requester
                                .clone()
                                .unwrap_or_default()
                                .name
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
        let mut response = certificates::ValidateCsrResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: String = req.csr.clone().unwrap_or_default().subject.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run PreviewCsr after validating the tenant and durably recording the command.
    async fn run_preview_csr(
        &self,
        req: certificates::PreviewCsrRequest,
    ) -> Result<certificates::PreviewCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CalculateCertificateRequest {
            certificate: Some(certificates::Certificate {
                counterparty: Some(certificates::CsrRef {
                    display_name: (match req.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
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
        let reply_0 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CalculateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::PreviewCertificateRequest {
            context: req.context.clone(),
            details: Some(certificates::preview_certificate_request::Details {
                contact: Some(common::ContactInfo {
                    phone: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
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
        let reply_1 = acme_platform::ports::execute(self.preview_csr_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = prescriptions::CreatePrescriptionRequest {
            context: req.context.clone(),
            prescription: Some(prescriptions::Prescription {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: req
                        .csr
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
                settlement: Some(prescriptions::prescription::Settlement::Card(
                    prescriptions::PrescriptionCardSettlement {
                        last4: (match reply_0.certificate.clone().unwrap_or_default().settlement.clone() {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = PrescriptionServiceClient::new(channel);
            let input: tonic::Request<prescriptions::CreatePrescriptionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_prescription(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::PreviewCsrResponse::default();
        let contribution: String = reply_0
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
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.csr.clone().unwrap_or_default().assignee.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.csr.get_or_insert_with(Default::default).assignee = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeCsr after validating the tenant and durably recording the command.
    async fn run_normalize_csr(
        &self,
        req: certificates::NormalizeCsrRequest,
    ) -> Result<certificates::NormalizeCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::NormalizeConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            etag: req
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .assignee
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: req
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
                    }],
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
            let input: tonic::Request<configuration::NormalizeConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = certificates::GetCsrRatesRequest {
            context: req.context.clone(),
            csr_id: req.csr_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CsrCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::GetCsrRatesRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_csr_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::NormalizeCsrResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: String = req.csr.clone().unwrap_or_default().subject.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run ExplainCsrDecision after validating the tenant and durably recording the command.
    async fn run_explain_csr_decision(
        &self,
        req: certificates::ExplainCsrDecisionRequest,
    ) -> Result<certificates::ExplainCsrDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExplainCsrDecision", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainCsrDecision", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = documents::BatchGetDocumentsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.explain_csr_decision_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::BatchGetConfigSetsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = (self.explain_csr_decision_1_port)(outgoing_1).await?;
        let outgoing_2 = featureflags::ShareFlagEvaluationExportRequest {
            context: req.context.clone(),
            details: Some(featureflags::share_flag_evaluation_export_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .csr
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
            }),
            ..Default::default()
        };
        let reply_2 =
            self.explain_csr_decision_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let outgoing_3 = certificates::PreviewCertificateRequest {
            certificate: Some(certificates::Certificate {
                counterparty: Some(certificates::CsrRef {
                    display_name: req
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
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::preview_certificate_request::Details {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.explain_csr_decision_3_port.clone());
        let reply_3 = acme_platform::ports::execute(&port, outgoing_3).await?;
        let mut response = certificates::ExplainCsrDecisionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: String = reply_2
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_3.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.csr.clone().unwrap_or_default().requester.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.csr.get_or_insert_with(Default::default).requester = contribution;
        }
        Ok(response)
    }
    /// Run ScoreCsr after validating the tenant and durably recording the command.
    async fn run_score_csr(
        &self,
        req: certificates::ScoreCsrRequest,
    ) -> Result<certificates::ScoreCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = documents::CreateDocumentRequest {
            context: req.context.clone(),
            document: Some(documents::Document {
                audit: Some(common::AuditInfo {
                    updated_at: req
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.score_csr_0_port)(outgoing_0).await?;
        let outgoing_1 = certificates::CalculateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_by: reply_0
                        .document
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        holder_name: (match reply_0.document.clone().unwrap_or_default().settlement.clone() {
                            Some(documents::document::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .holder_name
                        .clone(),
                        network: (match reply_0.document.clone().unwrap_or_default().settlement.clone() {
                            Some(documents::document::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .network
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.score_csr_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let jobs_2 = req
            .csr
            .clone()
            .unwrap_or_default()
            .comments
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_2 = featureflags::AnnotateFeatureFlagVersionRequest {
                        context: req.context.clone(),
                        feature_flag: Some(featureflags::FeatureFlag {
                            audit: Some(common::AuditInfo {
                                created_at: item.created_at.clone(),
                                updated_by: req
                                    .csr
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
                                email: item.author_email.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = FeatureFlagHistoryServiceClient::new(channel);
                        let input: tonic::Request<featureflags::AnnotateFeatureFlagVersionRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.annotate_feature_flag_version(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let mut response = certificates::ScoreCsrResponse::default();
        let contribution: String = reply_0
            .document
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::Money> = reply_1.certificate.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: i32 = req.csr.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.csr.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run LintCsrs after validating the tenant and durably recording the command.
    async fn run_lint_csrs(
        &self,
        req: certificates::LintCsrsRequest,
    ) -> Result<certificates::LintCsrsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintCsrs", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintCsrs", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let command_0 = LintCsrsDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::EstimateRateLimitRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::BrowseConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::browse_config_sets_request::Filter {
                created_after: reply_0
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.csr.clone().unwrap_or_default().custom_fields.clone(),
                owner_id: req
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
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.configuration.v1.BrowseConfigSetsRequest")?;
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
        let jobs_2 = req
            .csr
            .clone()
            .unwrap_or_default()
            .comments
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_2 = certificates::QuoteCertificateRequest {
                        certificate: Some(certificates::Certificate {
                            audit: Some(common::AuditInfo {
                                created_at: item.created_at.clone(),
                                ..Default::default()
                            }),
                            counterparty: Some(certificates::CsrRef {
                                display_name: req
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
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CertificateCalculationServiceClient::new(channel);
                        let input: tonic::Request<certificates::QuoteCertificateRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.quote_certificate(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let mut response = certificates::LintCsrsResponse::default();
        let contribution: i32 = reply_0.rate_limit.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.csr.get_or_insert_with(Default::default).priority = contribution;
        }
        let contribution: String = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .requester
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.csr.clone().unwrap_or_default().custom_fields.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .custom_fields
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run GetCsrRules after validating the tenant and durably recording the command.
    async fn run_get_csr_rules(
        &self,
        req: certificates::GetCsrRulesRequest,
    ) -> Result<certificates::GetCsrRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCsrRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCsrRules", "running workflow");
        let outgoing_0 = featureflags::RevertFeatureFlagRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
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
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<featureflags::RevertFeatureFlagRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::RevertFeatureFlagRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.revert_feature_flag(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::GetCsrRulesResponse::default();
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .assignee
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by LintCsrs for EstimateRateLimit.
struct LintCsrsDispatch0 {
    context: Option<common::RequestContext>,
    rate_limit_owner_display_name: String,
}
impl From<certificates::LintCsrsRequest> for LintCsrsDispatch0 {
    fn from(req: certificates::LintCsrsRequest) -> Self {
        Self {
            context: req.context.clone(),
            rate_limit_owner_display_name: req
                .csr
                .clone()
                .unwrap_or_default()
                .assignee
                .clone()
                .unwrap_or_default()
                .display_name
                .clone(),
        }
    }
}
impl From<LintCsrsDispatch0> for ratelimiting::EstimateRateLimitRequest {
    fn from(value: LintCsrsDispatch0) -> Self {
        ratelimiting::EstimateRateLimitRequest {
            context: value.context,
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
                    display_name: value.rate_limit_owner_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CsrValidationService for CsrValidationApi {
    /// Validates a csr.
    async fn validate_csr(
        &self,
        request: tonic::Request<certificates::ValidateCsrRequest>,
    ) -> Result<tonic::Response<certificates::ValidateCsrResponse>, tonic::Status> {
        self.run_validate_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a csr.
    async fn preview_csr(
        &self,
        request: tonic::Request<certificates::PreviewCsrRequest>,
    ) -> Result<tonic::Response<certificates::PreviewCsrResponse>, tonic::Status> {
        self.run_preview_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a csr.
    async fn normalize_csr(
        &self,
        request: tonic::Request<certificates::NormalizeCsrRequest>,
    ) -> Result<tonic::Response<certificates::NormalizeCsrResponse>, tonic::Status> {
        self.run_normalize_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a csr.
    async fn explain_csr_decision(
        &self,
        request: tonic::Request<certificates::ExplainCsrDecisionRequest>,
    ) -> Result<tonic::Response<certificates::ExplainCsrDecisionResponse>, tonic::Status> {
        self.run_explain_csr_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a csr.
    async fn score_csr(
        &self,
        request: tonic::Request<certificates::ScoreCsrRequest>,
    ) -> Result<tonic::Response<certificates::ScoreCsrResponse>, tonic::Status> {
        self.run_score_csr(request.into_inner()).await.map(tonic::Response::new)
    }
    /// Lints a csr.
    async fn lint_csrs(
        &self,
        request: tonic::Request<certificates::LintCsrsRequest>,
    ) -> Result<tonic::Response<certificates::LintCsrsResponse>, tonic::Status> {
        self.run_lint_csrs(request.into_inner()).await.map(tonic::Response::new)
    }
    /// Returns a csr.
    async fn get_csr_rules(
        &self,
        request: tonic::Request<certificates::GetCsrRulesRequest>,
    ) -> Result<tonic::Response<certificates::GetCsrRulesResponse>, tonic::Status> {
        self.run_get_csr_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
