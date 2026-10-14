use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_client::CertificateApprovalServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_approval_service_server::CertificateAuthorityApprovalService;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::chat::v1::chat_room_service_client::ChatRoomServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_query_service_client::DocumentQueryServiceClient;
use acme_proto::acme::documents::v1::document_service_client::DocumentServiceClient;
use acme_proto::acme::drivers::v1 as drivers;
use acme_proto::acme::drivers::v1::driver_service_client::DriverServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_validation_service_client::SearchIndexValidationServiceClient;

/// Routes certificate authorities through approval, with escalation and delegation.
#[derive(Clone)]
pub struct CertificateAuthorityApprovalController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    delegate_certificate_authority_approval_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            certificates::CheckCertificateEligibilityRequest,
            certificates::CheckCertificateEligibilityResponse,
        >,
    >,
    submit_certificate_authority_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            certificates::SuggestCertificatesRequest,
            certificates::SuggestCertificatesResponse,
        >,
    >,
    submit_certificate_authority_3_port:
        std::sync::Arc<dyn acme_platform::ports::RpcPort<chat::CloneChatRoomRequest, chat::CloneChatRoomResponse>>,
}
impl CertificateAuthorityApprovalController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            certificates::SuggestCertificatesRequest,
            certificates::SuggestCertificatesResponse,
        > = {
            let client = CertificateQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::SuggestCertificatesRequest| -> acme_platform::ports::RpcFuture<certificates::SuggestCertificatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.suggest_certificates(request).await?.into_inner())
})
})
        };
        let submit_certificate_authority_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<chat::CloneChatRoomRequest, chat::CloneChatRoomResponse> = {
            let client = ChatRoomServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: chat::CloneChatRoomRequest| -> acme_platform::ports::RpcFuture<chat::CloneChatRoomResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_chat_room(request).await?.into_inner())
})
})
        };
        let submit_certificate_authority_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::CheckCertificateEligibilityRequest,
            certificates::CheckCertificateEligibilityResponse,
        > = {
            let client = CertificateValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CheckCertificateEligibilityRequest| -> acme_platform::ports::RpcFuture<certificates::CheckCertificateEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_certificate_eligibility(request).await?.into_inner())
})
})
        };
        let delegate_certificate_authority_approval_0_port =
            std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            delegate_certificate_authority_approval_0_port,
            submit_certificate_authority_2_port,
            submit_certificate_authority_3_port,
        }
    }
    /// Run SubmitCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_submit_certificate_authority(
        &self,
        req: certificates::SubmitCertificateAuthorityRequest,
    ) -> Result<certificates::SubmitCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SubmitCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
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
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
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
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CsrCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::EstimateCsrRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = documents::SummarizeDocumentsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(documents::summarize_documents_request::Details {
                filters: reply_0.result.clone().unwrap_or_default().details.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = DocumentQueryServiceClient::new(channel);
            let input: tonic::Request<documents::SummarizeDocumentsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_documents(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = certificates::SuggestCertificatesRequest {
            context: req.context.clone(),
            filter: Some(certificates::suggest_certificates_request::Filter {
                created_after: req
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
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
        let reply_2 =
            acme_platform::ports::execute(self.submit_certificate_authority_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = chat::CloneChatRoomRequest {
            chat_room: Some(chat::ChatRoom {
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
        let reply_3 =
            acme_platform::ports::execute(self.submit_certificate_authority_3_port.as_ref(), outgoing_3).await?;
        let outgoing_4 = documents::BatchCreateDocumentsRequest {
            context: req.context.clone(),
            chunk: Some(documents::batch_create_documents_request::Chunk::Record(
                documents::Document {
                    audit: Some(common::AuditInfo {
                        updated_at: reply_3
                            .chat_room
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = DocumentServiceClient::new(channel);
            let input: tonic::Request<documents::BatchCreateDocumentsRequest> =
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
            let response = client.batch_create_documents(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    attributes: req.certificate_authority.clone().unwrap_or_default().attributes.clone(),
                    correlation_id: reply_3.result.clone().unwrap_or_default().correlation_id.clone(),
                    measurements: vec![featureflags::FlagEvaluationMeasurement {
                        unit: reply_3
                            .chat_room
                            .clone()
                            .unwrap_or_default()
                            .dimensions
                            .clone()
                            .unwrap_or_default()
                            .unit
                            .clone(),
                        ..Default::default()
                    }],
                    subject: Some(featureflags::FlagRolloutRef {
                        display_name: reply_3
                            .chat_room
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_5 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_5);
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
        let mut response = certificates::SubmitCertificateAuthorityResponse::default();
        let contribution: String = reply_0
            .csr
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
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
                .created_by = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3
            .chat_room
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
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
                .user_id = contribution;
        }
        let contribution: String = reply_3.chat_room.clone().unwrap_or_default().name.clone();
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
                .name = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ApproveCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_approve_certificate_authority(
        &self,
        req: certificates::ApproveCertificateAuthorityRequest,
    ) -> Result<certificates::ApproveCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ApproveCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::ExplainCsrDecisionRequest {
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
                audit: Some(common::AuditInfo {
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
                comments: vec![certificates::CsrComment {
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
                }],
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
        let outgoing_1 = drivers::CreateDriverRequest {
            context: req.context.clone(),
            driver: Some(drivers::Driver {
                audit: Some(common::AuditInfo {
                    etag: reply_0
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = DriverServiceClient::new(channel);
            let input: tonic::Request<drivers::CreateDriverRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_driver(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ApproveCertificateAuthorityResponse::default();
        let contribution: String = reply_0
            .csr
            .clone()
            .unwrap_or_default()
            .requester
            .clone()
            .unwrap_or_default()
            .email
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
                .email = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: Vec<String> = reply_1
            .driver
            .clone()
            .unwrap_or_default()
            .primary_address
            .clone()
            .unwrap_or_default()
            .lines
            .clone();
        if contribution != <Vec<String>>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .lines
                .extend(contribution);
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
    /// Run RejectCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_reject_certificate_authority(
        &self,
        req: certificates::RejectCertificateAuthorityRequest,
    ) -> Result<certificates::RejectCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RejectCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::PreviewCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            updated_at: req
                                .certificate_authority
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
                            recipient: req
                                .certificate_authority
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
            let mut client = CertificateAuthorityValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::PreviewCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = certificates::RejectCertificateAuthorityResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .certificate_authority
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
                .created_at = contribution;
        }
        let contribution: Option<identity::UserRef> = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .manager
            .clone();
        if contribution != <Option<identity::UserRef>>::default() {
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
                .manager = contribution;
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
    /// Run EscalateCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_escalate_certificate_authority(
        &self,
        req: certificates::EscalateCertificateAuthorityRequest,
    ) -> Result<certificates::EscalateCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("EscalateCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::ResolveCertificatesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = featureflags::RecalculateFlagRolloutsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .certificate_authority
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
                organizer: Some(identity::UserRef {
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
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CertificateQueryServiceClient::new(channel_0);
                let input: tonic::Request<certificates::ResolveCertificatesRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_certificates(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = FlagRolloutCalculationServiceClient::new(channel_1);
                let input: tonic::Request<featureflags::RecalculateFlagRolloutsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.recalculate_flag_rollouts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = ratelimiting::PushQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::push_quota_bucket_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_by: reply_1
                        .flag_rollout
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
        let reply_2 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::EscalateCertificateAuthorityResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .organizer
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
        let contribution: String = reply_1
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .organizer
            .clone()
            .unwrap_or_default()
            .user_id
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
                .user_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .quota_bucket
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
                .created_at = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_list_pending_certificate_authorities(
        &self,
        req: certificates::ListPendingCertificateAuthoritiesRequest,
    ) -> Result<certificates::ListPendingCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPendingCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::ScheduleConfigChangeExpiryRequest {
            config_change: Some(configuration::ConfigChange {
                occurred_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                source: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::schedule_config_change_expiry_request::Details {
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
                parameters: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeLifecycleServiceClient::new(channel);
            let input: tonic::Request<configuration::ScheduleConfigChangeExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_config_change_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = ListPendingCertificateAuthoritiesDispatch1::from(req.clone());
        command_1.search_index_owner_display_name = reply_0
            .config_change
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        let outgoing_1: search::PreviewSearchIndexRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SearchIndexValidationServiceClient::new(channel);
            let input: tonic::Request<search::PreviewSearchIndexRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_search_index(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ListPendingCertificateAuthoritiesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_change.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
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
            if response.certificate_authorities[0].children[0].children[0]
                .children
                .is_empty()
            {
                response.certificate_authorities[0].children[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.certificate_authorities[0].children[0].children[0].children[0]
                .attributes
                .extend(contribution);
        }
        let contribution: String = (match reply_1.search_index.clone().unwrap_or_default().locator.clone() {
            Some(search::search_index::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .region
        .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            if response.certificate_authorities[0].children.is_empty() {
                response.certificate_authorities[0].children.push(Default::default());
            }
            response.certificate_authorities[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run DelegateCertificateAuthorityApproval after validating the tenant and durably recording the command.
    async fn run_delegate_certificate_authority_approval(
        &self,
        req: certificates::DelegateCertificateAuthorityApprovalRequest,
    ) -> Result<certificates::DelegateCertificateAuthorityApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateCertificateAuthorityApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateCertificateAuthorityApproval", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CheckCertificateEligibilityRequest {
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
        let reply_0 =
            acme_platform::ports::execute(self.delegate_certificate_authority_approval_0_port.as_ref(), outgoing_0)
                .await?;
        let outgoing_1 = certificates::ApproveCertificateRequest {
            certificate: Some(certificates::Certificate {
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
                settlement: Some(certificates::certificate::Settlement::Bank(
                    certificates::CertificateBankSettlement {
                        bic: (match reply_0.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .bic
                        .clone(),
                        iban: (match reply_0.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .iban
                        .clone(),
                        ..Default::default()
                    },
                )),
                billing_address: Some(common::PostalAddress {
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::ApproveCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.approve_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::DelegateCertificateAuthorityApprovalResponse::default();
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
                .postal_code = contribution;
        }
        let contribution: String = reply_0
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run RecallCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_recall_certificate_authority(
        &self,
        req: certificates::RecallCertificateAuthorityRequest,
    ) -> Result<certificates::RecallCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecallCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SignalConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
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
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.SignalConfigSetWorkflowRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::SignalConfigSetWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_config_set_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::SummarizeCertificatesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(certificates::summarize_certificates_request::Details {
                filters: reply_0.result.clone().unwrap_or_default().details.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<certificates::SummarizeCertificatesRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CertificateQueryServiceClient::new(channel);
                let input: tonic::Request<certificates::SummarizeCertificatesRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.summarize_certificates(input).await?;
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
        let mut response = certificates::RecallCertificateAuthorityResponse::default();
        let contribution: Vec<String> = reply_0.config_set.clone().unwrap_or_default().path.clone();
        if contribution != <Vec<String>>::default() {
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
                .path
                .extend(contribution);
        }
        let contribution: Option<identity::UserRef> = reply_0.config_set.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
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
                .manager = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.certificate_authority.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
}
/// The domain command prepared by ListPendingCertificateAuthorities for PreviewSearchIndex.
struct ListPendingCertificateAuthoritiesDispatch1 {
    context: Option<common::RequestContext>,
    search_index_owner_display_name: String,
    search_index_address_region: String,
}
impl From<certificates::ListPendingCertificateAuthoritiesRequest> for ListPendingCertificateAuthoritiesDispatch1 {
    fn from(req: certificates::ListPendingCertificateAuthoritiesRequest) -> Self {
        Self {
            context: req.context.clone(),
            search_index_owner_display_name: Default::default(),
            search_index_address_region: req.filter.clone().unwrap_or_default().region.clone(),
        }
    }
}
impl From<ListPendingCertificateAuthoritiesDispatch1> for search::PreviewSearchIndexRequest {
    fn from(value: ListPendingCertificateAuthoritiesDispatch1) -> Self {
        search::PreviewSearchIndexRequest {
            context: value.context,
            search_index: Some(search::SearchIndex {
                locator: Some(search::search_index::Locator::Address(common::PostalAddress {
                    region: value.search_index_address_region,
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
                    display_name: value.search_index_owner_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CertificateAuthorityApprovalService for CertificateAuthorityApprovalController {
    /// Submits a certificate authority.
    async fn submit_certificate_authority(
        &self,
        request: tonic::Request<certificates::SubmitCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::SubmitCertificateAuthorityResponse>, tonic::Status> {
        self.run_submit_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a certificate authority.
    async fn approve_certificate_authority(
        &self,
        request: tonic::Request<certificates::ApproveCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::ApproveCertificateAuthorityResponse>, tonic::Status> {
        self.run_approve_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a certificate authority.
    async fn reject_certificate_authority(
        &self,
        request: tonic::Request<certificates::RejectCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::RejectCertificateAuthorityResponse>, tonic::Status> {
        self.run_reject_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a certificate authority.
    async fn escalate_certificate_authority(
        &self,
        request: tonic::Request<certificates::EscalateCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::EscalateCertificateAuthorityResponse>, tonic::Status> {
        self.run_escalate_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists certificate authorities a page at a time.
    async fn list_pending_certificate_authorities(
        &self,
        request: tonic::Request<certificates::ListPendingCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::ListPendingCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_list_pending_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates a certificate authority.
    async fn delegate_certificate_authority_approval(
        &self,
        request: tonic::Request<certificates::DelegateCertificateAuthorityApprovalRequest>,
    ) -> Result<tonic::Response<certificates::DelegateCertificateAuthorityApprovalResponse>, tonic::Status> {
        self.run_delegate_certificate_authority_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls a certificate authority.
    async fn recall_certificate_authority(
        &self,
        request: tonic::Request<certificates::RecallCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::RecallCertificateAuthorityResponse>, tonic::Status> {
        self.run_recall_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
