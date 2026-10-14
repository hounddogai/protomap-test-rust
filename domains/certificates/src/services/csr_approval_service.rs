use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_notification_service_client::AccessPolicyNotificationServiceClient;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::boxoffice::v1::box_office_sale_service_client::BoxOfficeSaleServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_workflow_service_client::CertificateWorkflowServiceClient;
use acme_proto::acme::certificates::v1::csr_approval_service_server::CsrApprovalService;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::certificates::v1::csr_lifecycle_service_client::CsrLifecycleServiceClient;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_approval_service_client::ConfigSetApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::customers::v1 as customers;
use acme_proto::acme::customers::v1::customer_query_service_client::CustomerQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_export_service_client::FlagRolloutExportServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::patients::v1::patient_query_service_client::PatientQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_validation_service_client::VenueValidationServiceClient;

/// Routes csrs through approval, with escalation and delegation.
#[derive(Clone)]
pub struct CsrApprovalEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    delegate_csr_approval_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<certificates::NormalizeCsrRequest, certificates::NormalizeCsrResponse>,
    >,
    escalate_csr_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                certificates::PreviewCertificateAuthorityRequest,
                certificates::PreviewCertificateAuthorityResponse,
            >,
    >,
    escalate_csr_1_port:
        acme_platform::ports::Callback<patients::ResolvePatientsRequest, patients::ResolvePatientsResponse>,
    get_csr_approval_status_0_port:
        acme_platform::ports::Callback<certificates::CalculateCsrRequest, certificates::CalculateCsrResponse>,
}
impl CsrApprovalEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            certificates::PreviewCertificateAuthorityRequest,
            certificates::PreviewCertificateAuthorityResponse,
        > = {
            let client = CertificateAuthorityValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::PreviewCertificateAuthorityRequest| -> acme_platform::ports::RpcFuture<certificates::PreviewCertificateAuthorityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_certificate_authority(request).await?.into_inner())
})
})
        };
        let escalate_csr_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            patients::ResolvePatientsRequest,
            patients::ResolvePatientsResponse,
        > = {
            let client = PatientQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: patients::ResolvePatientsRequest| -> acme_platform::ports::RpcFuture<patients::ResolvePatientsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_patients(request).await?.into_inner())
})
})
        };
        let escalate_csr_1_port = callback;
        let callback: acme_platform::ports::Callback<
            certificates::NormalizeCsrRequest,
            certificates::NormalizeCsrResponse,
        > = {
            let client = CsrValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::NormalizeCsrRequest| -> acme_platform::ports::RpcFuture<certificates::NormalizeCsrResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_csr(request).await?.into_inner())
})
})
        };
        let delegate_csr_approval_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::CalculateCsrRequest,
            certificates::CalculateCsrResponse,
        > = {
            let client = CsrCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CalculateCsrRequest| -> acme_platform::ports::RpcFuture<certificates::CalculateCsrResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_csr(request).await?.into_inner())
})
})
        };
        let get_csr_approval_status_0_port = callback;
        Self {
            channel,
            repository,
            delegate_csr_approval_1_port,
            escalate_csr_0_port,
            escalate_csr_1_port,
            get_csr_approval_status_0_port,
        }
    }
    /// Run SubmitCsr after validating the tenant and durably recording the command.
    async fn run_submit_csr(
        &self,
        req: certificates::SubmitCsrRequest,
    ) -> Result<certificates::SubmitCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ReconcileThrottlesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(ratelimiting::reconcile_throttles_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            reason: req.reason.clone(),
            throttle: Some(ratelimiting::Throttle {
                occurred_at: req
                    .csr
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                subject: Some(ratelimiting::RateLimitRef {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ReconcileThrottlesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_throttles(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::TerminateCsrRequest {
            context: req.context.clone(),
            csr_id: req.csr_id.clone(),
            details: Some(certificates::terminate_csr_request::Details {
                new_owner: Some(identity::UserRef {
                    email: (match req.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
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
            let mut client = CsrLifecycleServiceClient::new(channel);
            let input: tonic::Request<certificates::TerminateCsrRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.terminate_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = boxoffice::BatchCreateBoxOfficeSalesRequest {
            context: req.context.clone(),
            chunk: Some(boxoffice::batch_create_box_office_sales_request::Chunk::Record(
                boxoffice::BoxOfficeSale {
                    audit: Some(common::AuditInfo {
                        created_at: reply_1
                            .csr
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
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
                    counterparty: Some(boxoffice::CashDrawerRef {
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = BoxOfficeSaleServiceClient::new(channel);
            let input: tonic::Request<boxoffice::BatchCreateBoxOfficeSalesRequest> =
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
            let response = client.batch_create_box_office_sales(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::StreamQuotaBucketChangesRequest {
            frame: Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(
                ratelimiting::QuotaBucket {
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
                        ..Default::default()
                    }),
                    holder: Some(ratelimiting::RateLimitRef {
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
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketChangesRequest> =
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
            let response = client.stream_quota_bucket_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamQuotaBucketChangesResponse::default();
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
        let mut response = certificates::SubmitCsrResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = (match reply_1.csr.clone().unwrap_or_default().origin.clone() {
            Some(certificates::csr::Origin::Customer(value)) => value,
            _ => Default::default(),
        })
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
        let contribution: String = req.csr.clone().unwrap_or_default().subject.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run ApproveCsr after validating the tenant and durably recording the command.
    async fn run_approve_csr(
        &self,
        req: certificates::ApproveCsrRequest,
    ) -> Result<certificates::ApproveCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveCsr", "running workflow");
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
                    let outgoing_0 = authorization::UnsubscribeAccessPolicyUpdatesRequest {
                        context: req.context.clone(),
                        details: Some(authorization::unsubscribe_access_policy_updates_request::Details {
                            recipient: Some(common::ContactInfo {
                                email: item.author_email.clone(),
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
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AccessPolicyNotificationServiceClient::new(channel);
                        let input: tonic::Request<authorization::UnsubscribeAccessPolicyUpdatesRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.unsubscribe_access_policy_updates(input).await?;
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
                    let outgoing_1 = ratelimiting::ReconcileRateLimitsRequest {
                        context: req.context.clone(),
                        rate_limit: Some(ratelimiting::RateLimit {
                            audit: Some(common::AuditInfo {
                                created_at: item.created_at.clone(),
                                ..Default::default()
                            }),
                            action: Some(ratelimiting::rate_limit::Action::Block(
                                ratelimiting::RateLimitBlockAction {
                                    notify_email: item.author_email.clone(),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = RateLimitSyncServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::ReconcileRateLimitsRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.reconcile_rate_limits(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = featureflags::ListFlagRolloutVersionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_rollout_versions_request::Filter {
                created_after: reply_1
                    .rate_limit
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
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<featureflags::ListFlagRolloutVersionsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = FlagRolloutHistoryServiceClient::new(channel);
                let input: tonic::Request<featureflags::ListFlagRolloutVersionsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_flag_rollout_versions(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_2)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_2 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = certificates::ApproveCsrResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
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
        let contribution: Option<identity::UserRef> = req.csr.clone().unwrap_or_default().assignee.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.csr.get_or_insert_with(Default::default).assignee = contribution;
        }
        Ok(response)
    }
    /// Run RejectCsr after validating the tenant and durably recording the command.
    async fn run_reject_csr(
        &self,
        req: certificates::RejectCsrRequest,
    ) -> Result<certificates::RejectCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::RenewCsrRequest {
            context: req.context.clone(),
            csr_id: req.csr_id.clone(),
            details: Some(certificates::renew_csr_request::Details {
                new_owner: Some(identity::UserRef {
                    user_id: req
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
                notify_contact: Some(common::ContactInfo {
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
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = CsrLifecycleServiceClient::new(channel);
            let input: tonic::Request<certificates::RenewCsrRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.renew_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = certificates::ValidateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
                    display_name: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .assignee
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
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
                audit: Some(common::AuditInfo {
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
                requester: Some(common::ContactInfo {
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
            let mut client = CsrValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ValidateCsrRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFlagRolloutExportRequest {
            context: req.context.clone(),
            filter: Some(featureflags::stream_flag_rollout_export_request::Filter {
                created_after: reply_1
                    .csr
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: reply_1
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
        let channel = self.channel.clone();
        let _reply_2 = async move {
            let mut client = FlagRolloutExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutExportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_flag_rollout_export(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagRolloutExportResponse::default();
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
        let mut response = certificates::RejectCsrResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .csr
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .csr
            .clone()
            .unwrap_or_default()
            .sla
            .clone()
            .unwrap_or_default()
            .due_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .sla
                .get_or_insert_with(Default::default)
                .due_at = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run EscalateCsr after validating the tenant and durably recording the command.
    async fn run_escalate_csr(
        &self,
        req: certificates::EscalateCsrRequest,
    ) -> Result<certificates::EscalateCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::PreviewCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        manager: Some(identity::UserRef {
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
                    }],
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.escalate_csr_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = patients::ResolvePatientsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = (self.escalate_csr_1_port)(outgoing_1).await?;
        let outgoing_2 = configuration::GenerateConfigSetReportRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(configuration::generate_config_set_report_request::Details {
                filters: (reply_0
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .attributes
                    .clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigSetReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::GenerateConfigSetReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_config_set_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::EscalateCsrResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: i32 = req.csr.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.csr.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingCsrs after validating the tenant and durably recording the command.
    async fn run_list_pending_csrs(
        &self,
        req: certificates::ListPendingCsrsRequest,
    ) -> Result<certificates::ListPendingCsrsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListPendingCsrs", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingCsrs", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let command_0 = ListPendingCsrsDispatch0::from(req.clone());
        let outgoing_0: configuration::RejectConfigSetRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::RejectConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reject_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ListPendingCsrsResponse::default();
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
            if response.csrs.is_empty() {
                response.csrs.push(Default::default());
            }
            response.csrs[0].audit.get_or_insert_with(Default::default).etag = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run DelegateCsrApproval after validating the tenant and durably recording the command.
    async fn run_delegate_csr_approval(
        &self,
        req: certificates::DelegateCsrApprovalRequest,
    ) -> Result<certificates::DelegateCsrApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DelegateCsrApproval", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateCsrApproval", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = venues::NormalizeVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
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
                owner: Some(identity::UserRef {
                    email: (match req.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
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
            let mut client = VenueValidationServiceClient::new(channel);
            let input: tonic::Request<venues::NormalizeVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::NormalizeCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
                    user_id: (match req.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .user_id
                    .clone(),
                    ..Default::default()
                }),
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.delegate_csr_approval_1_port.as_ref(), outgoing_1).await?;
        let mut response = certificates::DelegateCsrApprovalResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .venue
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .csr
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RecallCsr after validating the tenant and durably recording the command.
    async fn run_recall_csr(
        &self,
        req: certificates::RecallCsrRequest,
    ) -> Result<certificates::RecallCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = customers::SummarizeCustomersRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.customers.v1.SummarizeCustomersRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CustomerQueryServiceClient::new(channel);
            let input: tonic::Request<customers::SummarizeCustomersRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_customers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::CheckCertificateAuthorityEligibilityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
                            user_id: req
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) =
            tokio::sync::mpsc::channel::<certificates::CheckCertificateAuthorityEligibilityRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CertificateAuthorityValidationServiceClient::new(channel);
                let input: tonic::Request<certificates::CheckCertificateAuthorityEligibilityRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.check_certificate_authority_eligibility(input).await?;
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
        let mut response = certificates::RecallCsrResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
            response
                .csr
                .get_or_insert_with(Default::default)
                .assignee
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run GetCsrApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_csr_approval_status(
        &self,
        req: certificates::GetCsrApprovalStatusRequest,
    ) -> Result<certificates::GetCsrApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCsrApprovalStatus", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCsrApprovalStatus", "running workflow");
        let outgoing_0 = certificates::CalculateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                csr_id: req.csr_id.clone(),
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
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
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.get_csr_approval_status_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = certificates::ValidateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
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
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            updated_by: reply_0
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
                        manager: Some(identity::UserRef {
                            display_name: reply_0
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .assignee
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                                Some(certificates::csr::Origin::Customer(value)) => value,
                                _ => Default::default(),
                            })
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
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<certificates::ValidateCertificateAuthorityRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateAuthorityValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ValidateCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::GetCsrApprovalStatusResponse::default();
        let contribution: String = reply_0
            .csr
            .clone()
            .unwrap_or_default()
            .requester
            .clone()
            .unwrap_or_default()
            .preferred_channel
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .requester
                .get_or_insert_with(Default::default)
                .preferred_channel = contribution;
        }
        let contribution: String = (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
            Some(certificates::csr::Origin::Customer(value)) => value,
            _ => Default::default(),
        })
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
        let contribution: Option<prost_types::Timestamp> = reply_1
            .certificate_authority
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
        Ok(response)
    }
    /// Run CommentCsrApproval after validating the tenant and durably recording the command.
    async fn run_comment_csr_approval(
        &self,
        req: certificates::CommentCsrApprovalRequest,
    ) -> Result<certificates::CommentCsrApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CommentCsrApproval", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CommentCsrApproval", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::SubmitCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
            let mut client = CertificateAuthorityApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::SubmitCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.submit_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = certificates::ResumeCertificateWorkflowRequest {
            certificate: Some(certificates::Certificate {
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
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::resume_certificate_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: (match req.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
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
            let input: tonic::Request<certificates::ResumeCertificateWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_certificate_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::CommentCsrApprovalResponse::default();
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
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .manager
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: i32 = req.csr.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.csr.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ListPendingCsrs for RejectConfigSet.
struct ListPendingCsrsDispatch0 {
    context: Option<common::RequestContext>,
    config_set_children_location_region: String,
    details_approver_email: String,
    config_set_children_children_manager_user_id: String,
    config_set_children_children_audit_created_at: Option<prost_types::Timestamp>,
}
impl From<certificates::ListPendingCsrsRequest> for ListPendingCsrsDispatch0 {
    fn from(req: certificates::ListPendingCsrsRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_location_region: req.filter.clone().unwrap_or_default().region.clone(),
            details_approver_email: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            config_set_children_children_manager_user_id: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            config_set_children_children_audit_created_at: req.filter.clone().unwrap_or_default().created_after.clone(),
        }
    }
}
impl From<ListPendingCsrsDispatch0> for configuration::RejectConfigSetRequest {
    fn from(value: ListPendingCsrsDispatch0) -> Self {
        configuration::RejectConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: value.config_set_children_children_audit_created_at,
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            user_id: value.config_set_children_children_manager_user_id,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    location: Some(common::PostalAddress {
                        region: value.config_set_children_location_region,
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: value.context,
            details: Some(configuration::reject_config_set_request::Details {
                approver: Some(identity::UserRef {
                    email: value.details_approver_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CsrApprovalService for CsrApprovalEndpoint {
    /// Submits a csr.
    async fn submit_csr(
        &self,
        request: tonic::Request<certificates::SubmitCsrRequest>,
    ) -> Result<tonic::Response<certificates::SubmitCsrResponse>, tonic::Status> {
        self.run_submit_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a csr.
    async fn approve_csr(
        &self,
        request: tonic::Request<certificates::ApproveCsrRequest>,
    ) -> Result<tonic::Response<certificates::ApproveCsrResponse>, tonic::Status> {
        self.run_approve_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a csr.
    async fn reject_csr(
        &self,
        request: tonic::Request<certificates::RejectCsrRequest>,
    ) -> Result<tonic::Response<certificates::RejectCsrResponse>, tonic::Status> {
        self.run_reject_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a csr.
    async fn escalate_csr(
        &self,
        request: tonic::Request<certificates::EscalateCsrRequest>,
    ) -> Result<tonic::Response<certificates::EscalateCsrResponse>, tonic::Status> {
        self.run_escalate_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists csrs a page at a time.
    async fn list_pending_csrs(
        &self,
        request: tonic::Request<certificates::ListPendingCsrsRequest>,
    ) -> Result<tonic::Response<certificates::ListPendingCsrsResponse>, tonic::Status> {
        self.run_list_pending_csrs(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates a csr.
    async fn delegate_csr_approval(
        &self,
        request: tonic::Request<certificates::DelegateCsrApprovalRequest>,
    ) -> Result<tonic::Response<certificates::DelegateCsrApprovalResponse>, tonic::Status> {
        self.run_delegate_csr_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls a csr.
    async fn recall_csr(
        &self,
        request: tonic::Request<certificates::RecallCsrRequest>,
    ) -> Result<tonic::Response<certificates::RecallCsrResponse>, tonic::Status> {
        self.run_recall_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a csr.
    async fn get_csr_approval_status(
        &self,
        request: tonic::Request<certificates::GetCsrApprovalStatusRequest>,
    ) -> Result<tonic::Response<certificates::GetCsrApprovalStatusResponse>, tonic::Status> {
        self.run_get_csr_approval_status(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Comments a csr.
    async fn comment_csr_approval(
        &self,
        request: tonic::Request<certificates::CommentCsrApprovalRequest>,
    ) -> Result<tonic::Response<certificates::CommentCsrApprovalResponse>, tonic::Status> {
        self.run_comment_csr_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
