use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_client::CertificateApprovalServiceClient;
use acme_proto::acme::certificates::v1::certificate_workflow_service_client::CertificateWorkflowServiceClient;
use acme_proto::acme::certificates::v1::csr_workflow_service_client::CsrWorkflowServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_approval_service_client::EnvironmentApprovalServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::employees::v1 as employees;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::itsm::v1 as itsm;
use acme_proto::acme::itsm::v1::incident_service_client::IncidentServiceClient;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_client::CryptoKeyQueryServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_server::KeyRingReportingService;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_service_client::MetricSeriesServiceClient;

/// Builds and schedules reports about key rings.
#[derive(Clone)]
pub struct KeyRingReportingEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    count_key_ring_totals_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                certificates::EscalateCertificateRequest,
                certificates::EscalateCertificateResponse,
            >,
    >,
    count_key_ring_totals_1_port:
        acme_platform::ports::Callback<certificates::StartCsrWorkflowRequest, certificates::StartCsrWorkflowResponse>,
    generate_key_ring_report_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<configuration::PushEnvironmentRequest, configuration::PushEnvironmentResponse>,
    >,
    stream_key_ring_metrics_1_port:
        std::sync::Arc<acme_platform::ports::Adapter<itsm::CloneIncidentRequest, itsm::CloneIncidentResponse>>,
}
impl KeyRingReportingEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::PushEnvironmentRequest,
            configuration::PushEnvironmentResponse,
        > = {
            let client = EnvironmentSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::PushEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::PushEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.push_environment(request).await?.into_inner())
})
})
        };
        let generate_key_ring_report_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<itsm::CloneIncidentRequest, itsm::CloneIncidentResponse> = {
            let client = IncidentServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: itsm::CloneIncidentRequest| -> acme_platform::ports::RpcFuture<itsm::CloneIncidentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_incident(request).await?.into_inner())
})
})
        };
        let stream_key_ring_metrics_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::EscalateCertificateRequest,
            certificates::EscalateCertificateResponse,
        > = {
            let client = CertificateApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::EscalateCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::EscalateCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.escalate_certificate(request).await?.into_inner())
})
})
        };
        let count_key_ring_totals_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::StartCsrWorkflowRequest,
            certificates::StartCsrWorkflowResponse,
        > = {
            let client = CsrWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::StartCsrWorkflowRequest| -> acme_platform::ports::RpcFuture<certificates::StartCsrWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.start_csr_workflow(request).await?.into_inner())
})
})
        };
        let count_key_ring_totals_1_port = callback;
        Self {
            channel,
            repository,
            count_key_ring_totals_0_port,
            count_key_ring_totals_1_port,
            generate_key_ring_report_0_port,
            stream_key_ring_metrics_1_port,
        }
    }
    /// Run GenerateKeyRingReport after validating the tenant and durably recording the command.
    async fn run_generate_key_ring_report(
        &self,
        req: keymanagement::GenerateKeyRingReportRequest,
    ) -> Result<keymanagement::GenerateKeyRingReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GenerateKeyRingReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateKeyRingReport", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::PushEnvironmentRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.generate_key_ring_report_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = keymanagement::SimulateKeyRotationRequest {
            context: req.context.clone(),
            details: Some(keymanagement::simulate_key_rotation_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .country_code
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_rotation: Some(keymanagement::KeyRotation {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .environment
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
                    locality: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .locality
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = KeyRotationCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SimulateKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = metrics::UpdateMetricSeriesRequest {
            context: req.context.clone(),
            metric_series: Some(metrics::MetricSeries {
                audit: Some(common::AuditInfo {
                    created_at: reply_1
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    metrics::MetricSeriesComponent {
                        description: reply_0
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .components
                            .clone()
                            .get("primary")
                            .cloned()
                            .unwrap_or_default()
                            .description
                            .clone(),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<metrics::UpdateMetricSeriesRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = MetricSeriesServiceClient::new(channel);
            let input: tonic::Request<metrics::UpdateMetricSeriesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_metric_series(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::GenerateKeyRingReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run FetchKeyRingReport after validating the tenant and durably recording the command.
    async fn run_fetch_key_ring_report(
        &self,
        req: keymanagement::FetchKeyRingReportRequest,
    ) -> Result<keymanagement::FetchKeyRingReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FetchKeyRingReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchKeyRingReport", "running workflow");
        let mut response = keymanagement::FetchKeyRingReportResponse::default();
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleKeyRingReport after validating the tenant and durably recording the command.
    async fn run_schedule_key_ring_report(
        &self,
        req: keymanagement::ScheduleKeyRingReportRequest,
    ) -> Result<keymanagement::ScheduleKeyRingReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScheduleKeyRingReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleKeyRingReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = configuration::ListPendingEnvironmentsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_pending_environments_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: (req.details.clone().unwrap_or_default().filters.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req.key_ring_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::ListPendingEnvironmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_environments(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ScheduleKeyRingReportResponse::default();
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
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run StreamKeyRingMetrics after validating the tenant and durably recording the command.
    async fn run_stream_key_ring_metrics(
        &self,
        req: keymanagement::StreamKeyRingMetricsRequest,
    ) -> Result<keymanagement::StreamKeyRingMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StreamKeyRingMetrics", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamKeyRingMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = certificates::ResumeCertificateWorkflowRequest {
            context: req.context.clone(),
            details: Some(certificates::resume_certificate_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateWorkflowServiceClient::new(channel);
            let input: tonic::Request<certificates::ResumeCertificateWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_certificate_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = itsm::CloneIncidentRequest {
            context: req.context.clone(),
            incident: Some(itsm::Incident {
                origin: Some(itsm::incident::Origin::Customer(employees::EmployeeRef {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.stream_key_ring_metrics_1_port.as_ref(), outgoing_1).await?;
        let mut response = keymanagement::StreamKeyRingMetricsResponse::default();
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
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .incident
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_1
            .incident
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run ListKeyRingReports after validating the tenant and durably recording the command.
    async fn run_list_key_ring_reports(
        &self,
        req: keymanagement::ListKeyRingReportsRequest,
    ) -> Result<keymanagement::ListKeyRingReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListKeyRingReports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListKeyRingReports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let command_0 = ListKeyRingReportsDispatch0::from(req.clone());
        let outgoing_0: keymanagement::ScoreKeyRingRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ScoreKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ListKeyRingReportsResponse::default();
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            if response.key_rings[0].children.is_empty() {
                response.key_rings[0].children.push(Default::default());
            }
            response.key_rings[0].children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            if response.key_rings[0].children.is_empty() {
                response.key_rings[0].children.push(Default::default());
            }
            if response.key_rings[0].children[0].children.is_empty() {
                response.key_rings[0].children[0].children.push(Default::default());
            }
            response.key_rings[0].children[0].children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        Ok(response)
    }
    /// Run ShareKeyRingReport after validating the tenant and durably recording the command.
    async fn run_share_key_ring_report(
        &self,
        req: keymanagement::ShareKeyRingReportRequest,
    ) -> Result<keymanagement::ShareKeyRingReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareKeyRingReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareKeyRingReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::EstimateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::EstimateCryptoKeyRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CryptoKeyCalculationServiceClient::new(channel);
                let input: tonic::Request<keymanagement::EstimateCryptoKeyRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.estimate_crypto_key(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_0)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_0 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let outgoing_1 = configuration::ListEnvironmentReportsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_environment_reports_request::Filter {
                created_after: reply_0
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: reply_0.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: req.key_ring_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = EnvironmentReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::ListEnvironmentReportsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_environment_reports(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = keymanagement::ExportCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::export_crypto_keys_request::Filter {
                created_after: reply_0
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_0.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_0
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .key_ring_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_2 = async move {
            let mut client = CryptoKeyQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ExportCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_crypto_keys(input).await?;
            let mut frames = response.into_inner();
            let mut last = keymanagement::ExportCryptoKeysResponse::default();
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
        let mut response = keymanagement::ShareKeyRingReportResponse::default();
        let contribution: String = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .recipient
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run CountKeyRingTotals after validating the tenant and durably recording the command.
    async fn run_count_key_ring_totals(
        &self,
        req: keymanagement::CountKeyRingTotalsRequest,
    ) -> Result<keymanagement::CountKeyRingTotalsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountKeyRingTotals", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountKeyRingTotals", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::EscalateCertificateRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::escalate_certificate_request::Details {
                approver: Some(identity::UserRef {
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
        let reply_0 = acme_platform::ports::execute(self.count_key_ring_totals_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = certificates::StartCsrWorkflowRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                comments: vec![certificates::CsrComment {
                    created_at: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }],
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
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
                })),
                sla: Some(certificates::CsrSla {
                    due_at: reply_0.certificate.clone().unwrap_or_default().due_at.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = (self.count_key_ring_totals_1_port)(outgoing_1).await?;
        let mut response = keymanagement::CountKeyRingTotalsResponse::default();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ListKeyRingReports for ScoreKeyRing.
struct ListKeyRingReportsDispatch0 {
    context: Option<common::RequestContext>,
    key_ring_children_children_location_region: String,
    key_ring_manager_email: String,
    key_ring_children_manager_user_id: String,
}
impl From<keymanagement::ListKeyRingReportsRequest> for ListKeyRingReportsDispatch0 {
    fn from(req: keymanagement::ListKeyRingReportsRequest) -> Self {
        Self {
            context: req.context.clone(),
            key_ring_children_children_location_region: req.filter.clone().unwrap_or_default().region.clone(),
            key_ring_manager_email: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            key_ring_children_manager_user_id: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
        }
    }
}
impl From<ListKeyRingReportsDispatch0> for keymanagement::ScoreKeyRingRequest {
    fn from(value: ListKeyRingReportsDispatch0) -> Self {
        keymanagement::ScoreKeyRingRequest {
            context: value.context,
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        location: Some(common::PostalAddress {
                            region: value.key_ring_children_children_location_region,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        user_id: value.key_ring_children_manager_user_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                manager: Some(identity::UserRef {
                    email: value.key_ring_manager_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRingReportingService for KeyRingReportingEndpoint {
    /// Generates a key ring.
    async fn generate_key_ring_report(
        &self,
        request: tonic::Request<keymanagement::GenerateKeyRingReportRequest>,
    ) -> Result<tonic::Response<keymanagement::GenerateKeyRingReportResponse>, tonic::Status> {
        self.run_generate_key_ring_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key ring.
    async fn fetch_key_ring_report(
        &self,
        request: tonic::Request<keymanagement::FetchKeyRingReportRequest>,
    ) -> Result<tonic::Response<keymanagement::FetchKeyRingReportResponse>, tonic::Status> {
        self.run_fetch_key_ring_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a key ring.
    async fn schedule_key_ring_report(
        &self,
        request: tonic::Request<keymanagement::ScheduleKeyRingReportRequest>,
    ) -> Result<tonic::Response<keymanagement::ScheduleKeyRingReportResponse>, tonic::Status> {
        self.run_schedule_key_ring_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamKeyRingMetricsStream =
        tokio_stream::wrappers::ReceiverStream<Result<keymanagement::StreamKeyRingMetricsResponse, tonic::Status>>;
    /// Streams the changes of key rings as they happen.
    async fn stream_key_ring_metrics(
        &self,
        request: tonic::Request<keymanagement::StreamKeyRingMetricsRequest>,
    ) -> Result<tonic::Response<Self::StreamKeyRingMetricsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_key_ring_metrics(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Lists key rings a page at a time.
    async fn list_key_ring_reports(
        &self,
        request: tonic::Request<keymanagement::ListKeyRingReportsRequest>,
    ) -> Result<tonic::Response<keymanagement::ListKeyRingReportsResponse>, tonic::Status> {
        self.run_list_key_ring_reports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Shares a key ring.
    async fn share_key_ring_report(
        &self,
        request: tonic::Request<keymanagement::ShareKeyRingReportRequest>,
    ) -> Result<tonic::Response<keymanagement::ShareKeyRingReportResponse>, tonic::Status> {
        self.run_share_key_ring_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts key rings by status.
    async fn count_key_ring_totals(
        &self,
        request: tonic::Request<keymanagement::CountKeyRingTotalsRequest>,
    ) -> Result<tonic::Response<keymanagement::CountKeyRingTotalsResponse>, tonic::Status> {
        self.run_count_key_ring_totals(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
