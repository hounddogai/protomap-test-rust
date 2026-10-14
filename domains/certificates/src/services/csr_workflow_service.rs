use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_query_service_client::AccessPolicyQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_client::CertificateApprovalServiceClient;
use acme_proto::acme::certificates::v1::certificate_service_client::CertificateServiceClient;
use acme_proto::acme::certificates::v1::csr_approval_service_client::CsrApprovalServiceClient;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::certificates::v1::csr_workflow_service_server::CsrWorkflowService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::concessions::v1::concession_stand_service_client::ConcessionStandServiceClient;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_approval_service_client::EnvironmentApprovalServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_reporting_service_client::RateLimitReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_workflow_service_client::ThrottleWorkflowServiceClient;

/// Runs the long-lived workflow that takes a csr from start to finish.
#[derive(Clone)]
pub struct CsrWorkflowEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
}
impl CsrWorkflowEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run StartCsrWorkflow after validating the tenant and durably recording the command.
    async fn run_start_csr_workflow(
        &self,
        req: certificates::StartCsrWorkflowRequest,
    ) -> Result<certificates::StartCsrWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StartCsrWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartCsrWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().subject.clone(), "recorded audit field");
        let outgoing_0 = certificates::GetCsrApprovalStatusRequest {
            context: req.context.clone(),
            csr_id: req.csr.clone().unwrap_or_default().csr_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CsrApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::GetCsrApprovalStatusRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_csr_approval_status(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::DelegateCertificateApprovalRequest {
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
        let reply_1 = async move {
            let mut client = CertificateApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::DelegateCertificateApprovalRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.delegate_certificate_approval(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = concessions::BatchCreateConcessionStandsRequest {
            context: req.context.clone(),
            chunk: Some(concessions::batch_create_concession_stands_request::Chunk::Header(
                concessions::batch_create_concession_stands_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
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
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify
                            .clone()
                            .unwrap_or_default()
                            .name
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConcessionStandServiceClient::new(channel);
            let input: tonic::Request<concessions::BatchCreateConcessionStandsRequest> =
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
            let response = client.batch_create_concession_stands(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                    measurements: vec![featureflags::FlagEvaluationMeasurement {
                        observed_at: reply_1.certificate.clone().unwrap_or_default().due_at.clone(),
                        ..Default::default()
                    }],
                    subject: Some(featureflags::FlagRolloutRef {
                        display_name: reply_1
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .counterparty
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        starts_at: req
                            .csr
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
        let mut response = certificates::StartCsrWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: i32 = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .priority
            .clone();
        if contribution != <i32>::default() {
            response.csr.get_or_insert_with(Default::default).priority = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run AdvanceCsrWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_csr_workflow(
        &self,
        req: certificates::AdvanceCsrWorkflowRequest,
    ) -> Result<certificates::AdvanceCsrWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("AdvanceCsrWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceCsrWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let jobs_0 = req
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
                    let outgoing_0 = certificates::ListCertificatesRequest {
                        context: req.context.clone(),
                        filter: Some(certificates::list_certificates_request::Filter {
                            created_after: item.created_at.clone(),
                            labels: req.csr.clone().unwrap_or_default().custom_fields.clone(),
                            owner_id: item.comment_id.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CertificateServiceClient::new(channel);
                        let input: tonic::Request<certificates::ListCertificatesRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.list_certificates(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = featureflags::PreviewFlagRolloutRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: (match req.csr.clone().unwrap_or_default().origin.clone() {
                            Some(certificates::csr::Origin::Customer(value)) => value,
                            _ => Default::default(),
                        })
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagRolloutValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::PreviewFlagRolloutRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_flag_rollout(input).await?;
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
                    let outgoing_2 = certificates::SubmitCsrRequest {
                        context: req.context.clone(),
                        csr: Some(certificates::Csr {
                            comments: vec![certificates::CsrComment {
                                author_email: item.author_email.clone(),
                                created_at: item.created_at.clone(),
                                visibility: item.visibility.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CsrApprovalServiceClient::new(channel);
                        let input: tonic::Request<certificates::SubmitCsrRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.submit_csr(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let mut response = certificates::AdvanceCsrWorkflowResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.csr.get_or_insert_with(Default::default).comments.is_empty() {
                response
                    .csr
                    .get_or_insert_with(Default::default)
                    .comments
                    .push(Default::default());
            }
            response.csr.get_or_insert_with(Default::default).comments[0].created_at = contribution;
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
    /// Run CancelCsrWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_csr_workflow(
        &self,
        req: certificates::CancelCsrWorkflowRequest,
    ) -> Result<certificates::CancelCsrWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CancelCsrWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelCsrWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ListRateLimitReportsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_rate_limit_reports_request::Filter {
                created_after: req
                    .csr
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.details.clone().unwrap_or_default().parameters.clone(),
                owner_id: (match req.csr.clone().unwrap_or_default().origin.clone() {
                    Some(certificates::csr::Origin::Customer(value)) => value,
                    _ => Default::default(),
                })
                .user_id
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListRateLimitReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_rate_limit_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = notifications::BatchGetNotificationsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = NotificationServiceClient::new(channel);
            let input: tonic::Request<notifications::BatchGetNotificationsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_notifications(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::CancelCsrWorkflowResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
    /// Run RetryCsrWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_csr_workflow(
        &self,
        req: certificates::RetryCsrWorkflowRequest,
    ) -> Result<certificates::RetryCsrWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryCsrWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryCsrWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let jobs_0 = req
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
                    let outgoing_0 = certificates::CompareCsrOptionsRequest {
                        context: req.context.clone(),
                        csr: Some(certificates::Csr {
                            comments: vec![certificates::CsrComment {
                                body: item.body.clone(),
                                created_at: item.created_at.clone(),
                                visibility: item.visibility.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CsrCalculationServiceClient::new(channel);
                        let input: tonic::Request<certificates::CompareCsrOptionsRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.compare_csr_options(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = authorization::BrowseAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::browse_access_policies_request::Filter {
                created_after: reply_0
                    .csr
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_0.csr.clone().unwrap_or_default().custom_fields.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessPolicyQueryServiceClient::new(channel);
            let input: tonic::Request<authorization::BrowseAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::RetryCsrWorkflowResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
    /// Run PauseCsrWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_csr_workflow(
        &self,
        req: certificates::PauseCsrWorkflowRequest,
    ) -> Result<certificates::PauseCsrWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PauseCsrWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseCsrWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::PreviewFlagRolloutRequest {
            context: req.context.clone(),
            details: Some(featureflags::preview_flag_rollout_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .email
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
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: (match req.csr.clone().unwrap_or_default().origin.clone() {
                            Some(certificates::csr::Origin::Customer(value)) => value,
                            _ => Default::default(),
                        })
                        .email
                        .clone(),
                        phone: req
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::PreviewFlagRolloutRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_flag_rollout(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ResumeThrottleWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::resume_throttle_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ResumeThrottleWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_throttle_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::PauseCsrWorkflowResponse::default();
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .organizer
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
                .csr
                .get_or_insert_with(Default::default)
                .assignee
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
    /// Run ResumeCsrWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_csr_workflow(
        &self,
        req: certificates::ResumeCsrWorkflowRequest,
    ) -> Result<certificates::ResumeCsrWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResumeCsrWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeCsrWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CountConfigSetTotalsRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_set_totals_request::Filter {
                created_after: req
                    .csr
                    .clone()
                    .unwrap_or_default()
                    .sla
                    .clone()
                    .unwrap_or_default()
                    .due_at
                    .clone(),
                labels: req.details.clone().unwrap_or_default().parameters.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::CountConfigSetTotalsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_config_set_totals(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ResumeCsrWorkflowResponse::default();
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
    /// Run ListCsrWorkflows after validating the tenant and durably recording the command.
    async fn run_list_csr_workflows(
        &self,
        req: certificates::ListCsrWorkflowsRequest,
    ) -> Result<certificates::ListCsrWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListCsrWorkflows", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListCsrWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::SubscribeFlagEvaluationUpdatesRequest {
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
            context: req.context.clone(),
            details: Some(featureflags::subscribe_flag_evaluation_updates_request::Details {
                recipient: Some(common::ContactInfo {
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
                variables: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::SubscribeFlagEvaluationUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_flag_evaluation_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ListEnvironmentReportsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_environment_reports_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ListEnvironmentReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_environment_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ListCsrWorkflowsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0
            .flag_evaluation
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.page_info.clone().unwrap_or_default().next_page_token.clone();
        if contribution != <String>::default() {
            response.page_info.get_or_insert_with(Default::default).next_page_token = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run SignalCsrWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_csr_workflow(
        &self,
        req: certificates::SignalCsrWorkflowRequest,
    ) -> Result<certificates::SignalCsrWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SignalCsrWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalCsrWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::EscalateEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: req
                        .csr
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
            let mut client = EnvironmentApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::EscalateEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.escalate_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ScheduleThrottleExpiryRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::schedule_throttle_expiry_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    phone: req
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
            throttle: Some(ratelimiting::Throttle {
                measurements: vec![ratelimiting::ThrottleMeasurement {
                    unit: reply_0
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleLifecycleServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ScheduleThrottleExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_throttle_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::SignalCsrWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
                .csr
                .get_or_insert_with(Default::default)
                .assignee
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
}
acme_platform::endpoints! { CsrWorkflowService, CsrWorkflowEndpoint;
unary {
start_csr_workflow (certificates::StartCsrWorkflowRequest) -> certificates::StartCsrWorkflowResponse => run_start_csr_workflow;
advance_csr_workflow (certificates::AdvanceCsrWorkflowRequest) -> certificates::AdvanceCsrWorkflowResponse => run_advance_csr_workflow;
cancel_csr_workflow (certificates::CancelCsrWorkflowRequest) -> certificates::CancelCsrWorkflowResponse => run_cancel_csr_workflow;
retry_csr_workflow (certificates::RetryCsrWorkflowRequest) -> certificates::RetryCsrWorkflowResponse => run_retry_csr_workflow;
pause_csr_workflow (certificates::PauseCsrWorkflowRequest) -> certificates::PauseCsrWorkflowResponse => run_pause_csr_workflow;
resume_csr_workflow (certificates::ResumeCsrWorkflowRequest) -> certificates::ResumeCsrWorkflowResponse => run_resume_csr_workflow;
list_csr_workflows (certificates::ListCsrWorkflowsRequest) -> certificates::ListCsrWorkflowsResponse => run_list_csr_workflows;
signal_csr_workflow (certificates::SignalCsrWorkflowRequest) -> certificates::SignalCsrWorkflowResponse => run_signal_csr_workflow;
}
server {
}
client {
}
bidi {
}
}
