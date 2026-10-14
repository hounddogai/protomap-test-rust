use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::configuration::v1::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient;
use acme_proto::acme::employees::v1 as employees;
use acme_proto::acme::employees::v1::employee_service_client::EmployeeServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_approval_service_server::QuotaBucketApprovalService;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;

/// Routes quota buckets through approval, with escalation and delegation.
#[derive(Clone)]
pub struct QuotaBucketApprovalHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    submit_quota_bucket_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::GenerateQuotaBucketReportRequest,
            ratelimiting::GenerateQuotaBucketReportResponse,
        >,
    >,
    submit_quota_bucket_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<employees::UpdateEmployeeRequest, employees::UpdateEmployeeResponse>,
    >,
    submit_quota_bucket_3_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::SubmitConfigChangeRequest,
            configuration::SubmitConfigChangeResponse,
        >,
    >,
}
impl QuotaBucketApprovalHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::GenerateQuotaBucketReportRequest,
            ratelimiting::GenerateQuotaBucketReportResponse,
        > = {
            let client = QuotaBucketReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::GenerateQuotaBucketReportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::GenerateQuotaBucketReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.generate_quota_bucket_report(request).await?.into_inner())
})
})
        };
        let submit_quota_bucket_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            employees::UpdateEmployeeRequest,
            employees::UpdateEmployeeResponse,
        > = {
            let client = EmployeeServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: employees::UpdateEmployeeRequest| -> acme_platform::ports::RpcFuture<employees::UpdateEmployeeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.update_employee(request).await?.into_inner())
})
})
        };
        let submit_quota_bucket_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::SubmitConfigChangeRequest,
            configuration::SubmitConfigChangeResponse,
        > = {
            let client = ConfigChangeApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SubmitConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::SubmitConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.submit_config_change(request).await?.into_inner())
})
})
        };
        let submit_quota_bucket_3_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            submit_quota_bucket_0_port,
            submit_quota_bucket_1_port,
            submit_quota_bucket_3_port,
        }
    }
    /// Run SubmitQuotaBucket after validating the tenant and durably recording the command.
    async fn run_submit_quota_bucket(
        &self,
        req: ratelimiting::SubmitQuotaBucketRequest,
    ) -> Result<ratelimiting::SubmitQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::GenerateQuotaBucketReportRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.submit_quota_bucket_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = employees::UpdateEmployeeRequest {
            context: req.context.clone(),
            employee: Some(employees::Employee {
                audit: Some(common::AuditInfo {
                    etag: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_at: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    email: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .quota_bucket
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
        let reply_1 = acme_platform::ports::execute(self.submit_quota_bucket_1_port.as_ref(), outgoing_1).await?;
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
                    updated_at: req
                        .quota_bucket
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
        let outgoing_3 = configuration::SubmitConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
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
            details: Some(configuration::submit_config_change_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_2
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: reply_1
                        .employee
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: reply_1
                        .employee
                        .clone()
                        .unwrap_or_default()
                        .relationship_manager
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
        let reply_3 =
            self.submit_quota_bucket_3_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_3)
            .await?;
        let mut response = ratelimiting::SubmitQuotaBucketResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .employee
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.quota_bucket.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ApproveQuotaBucket after validating the tenant and durably recording the command.
    async fn run_approve_quota_bucket(
        &self,
        req: ratelimiting::ApproveQuotaBucketRequest,
    ) -> Result<ratelimiting::ApproveQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::QuoteCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            etag: req
                                .quota_bucket
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
                                .clone(),
                            updated_at: req
                                .quota_bucket
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
                            display_name: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .approver
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.certificates.v1.QuoteCertificateAuthorityRequest")?;
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
        let outgoing_1 = configuration::ScheduleEnvironmentExpiryRequest {
            context: req.context.clone(),
            details: Some(configuration::schedule_environment_expiry_request::Details {
                notify: Some(common::ContactInfo {
                    email: reply_0
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::ScheduleEnvironmentExpiryRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = EnvironmentLifecycleServiceClient::new(channel);
                let input: tonic::Request<configuration::ScheduleEnvironmentExpiryRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.schedule_environment_expiry(input).await?;
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
        let mut response = ratelimiting::ApproveQuotaBucketResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.quota_bucket.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run EscalateQuotaBucket after validating the tenant and durably recording the command.
    async fn run_escalate_quota_bucket(
        &self,
        req: ratelimiting::EscalateQuotaBucketRequest,
    ) -> Result<ratelimiting::EscalateQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamQuotaBucketMetricsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::stream_quota_bucket_metrics_request::Filter {
                labels: req.quota_bucket.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
                name: req
                    .quota_bucket
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
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_quota_bucket_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamQuotaBucketMetricsResponse::default();
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
        let outgoing_1 = featureflags::QuoteFlagEvaluationRequest {
            context: req.context.clone(),
            details: Some(featureflags::quote_flag_evaluation_request::Details {
                currency_code: req.quota_bucket.clone().unwrap_or_default().currency_code.clone(),
                ..Default::default()
            }),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagEvaluationCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::QuoteFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::EscalateQuotaBucketResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.quota_bucket.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ListPendingQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_list_pending_quota_buckets(
        &self,
        req: ratelimiting::ListPendingQuotaBucketsRequest,
    ) -> Result<ratelimiting::ListPendingQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPendingQuotaBuckets", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = ratelimiting::ListPendingQuotaBucketsResponse::default();
        Ok(response)
    }
    /// Run GetQuotaBucketApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_quota_bucket_approval_status(
        &self,
        req: ratelimiting::GetQuotaBucketApprovalStatusRequest,
    ) -> Result<ratelimiting::GetQuotaBucketApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetQuotaBucketApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetQuotaBucketApprovalStatus", "running workflow");
        let mut response = ratelimiting::GetQuotaBucketApprovalStatusResponse::default();
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        Ok(response)
    }
    /// Run CommentQuotaBucketApproval after validating the tenant and durably recording the command.
    async fn run_comment_quota_bucket_approval(
        &self,
        req: ratelimiting::CommentQuotaBucketApprovalRequest,
    ) -> Result<ratelimiting::CommentQuotaBucketApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CommentQuotaBucketApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CommentQuotaBucketApproval", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let jobs_0 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = configuration::SignalConfigSetWorkflowRequest {
                        config_set: Some(configuration::ConfigSet {
                            children: vec![configuration::ConfigSet {
                                children: vec![configuration::ConfigSet {
                                    children: vec![configuration::ConfigSet {
                                        name: item.name.clone(),
                                        ..Default::default()
                                    }],
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
                        let mut client = ConfigSetWorkflowServiceClient::new(channel);
                        let input: tonic::Request<configuration::SignalConfigSetWorkflowRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.signal_config_set_workflow(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = ratelimiting::CommentQuotaBucketApprovalResponse::default();
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: String = req.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl QuotaBucketApprovalService for QuotaBucketApprovalHandler {
    /// Submits a quota bucket.
    async fn submit_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::SubmitQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::SubmitQuotaBucketResponse>, tonic::Status> {
        self.run_submit_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a quota bucket.
    async fn approve_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::ApproveQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::ApproveQuotaBucketResponse>, tonic::Status> {
        self.run_approve_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a quota bucket.
    async fn escalate_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::EscalateQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::EscalateQuotaBucketResponse>, tonic::Status> {
        self.run_escalate_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists quota buckets a page at a time.
    async fn list_pending_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::ListPendingQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListPendingQuotaBucketsResponse>, tonic::Status> {
        self.run_list_pending_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a quota bucket.
    async fn get_quota_bucket_approval_status(
        &self,
        request: tonic::Request<ratelimiting::GetQuotaBucketApprovalStatusRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetQuotaBucketApprovalStatusResponse>, tonic::Status> {
        self.run_get_quota_bucket_approval_status(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Comments a quota bucket.
    async fn comment_quota_bucket_approval(
        &self,
        request: tonic::Request<ratelimiting::CommentQuotaBucketApprovalRequest>,
    ) -> Result<tonic::Response<ratelimiting::CommentQuotaBucketApprovalResponse>, tonic::Status> {
        self.run_comment_quota_bucket_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
