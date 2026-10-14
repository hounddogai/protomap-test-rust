use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_history_service_client::RoleBindingHistoryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_reporting_service_server::ConfigChangeReportingService;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::employees::v1 as employees;
use acme_proto::acme::employees::v1::employee_service_client::EmployeeServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::payroll::v1 as payroll;
use acme_proto::acme::payroll::v1::pay_run_service_client::PayRunServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_query_service_client::SearchIndexQueryServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::secret_version_validation_service_client::SecretVersionValidationServiceClient;
use acme_proto::acme::telehealth::v1 as telehealth;
use acme_proto::acme::telehealth::v1::video_visit_calculation_service_client::VideoVisitCalculationServiceClient;
use acme_proto::acme::warehousing::v1 as warehousing;
use acme_proto::acme::warehousing::v1::warehouse_query_service_client::WarehouseQueryServiceClient;

/// Builds and schedules reports about config changes.
#[derive(Clone)]
pub struct ConfigChangeReportingHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
}
impl ConfigChangeReportingHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run GenerateConfigChangeReport after validating the tenant and durably recording the command.
    async fn run_generate_config_change_report(
        &self,
        req: configuration::GenerateConfigChangeReportRequest,
    ) -> Result<configuration::GenerateConfigChangeReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GenerateConfigChangeReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateConfigChangeReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = configuration::ExplainConfigSetDecisionRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
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
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainConfigSetDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_config_set_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::GetConfigChangeRulesRequest {
            config_change_id: req.config_change_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::GetConfigChangeRulesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_config_change_rules(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = employees::BatchCreateEmployeesRequest {
            context: req.context.clone(),
            chunk: Some(employees::batch_create_employees_request::Chunk::Record(
                employees::Employee {
                    audit: Some(common::AuditInfo {
                        updated_at: reply_0
                            .config_set
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
                        email: reply_0
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .manager
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    }),
                    relationship_manager: Some(identity::UserRef {
                        user_id: reply_0
                            .config_set
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = EmployeeServiceClient::new(channel);
            let input: tonic::Request<employees::BatchCreateEmployeesRequest> =
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
            let response = client.batch_create_employees(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        created_by: reply_0
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        ..Default::default()
                    }),
                    owner: Some(identity::UserRef {
                        display_name: reply_0
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .manager
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        email: reply_0
                            .config_set
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
        let mut response = configuration::GenerateConfigChangeReportResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.config_change.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run FetchConfigChangeReport after validating the tenant and durably recording the command.
    async fn run_fetch_config_change_report(
        &self,
        req: configuration::FetchConfigChangeReportRequest,
    ) -> Result<configuration::FetchConfigChangeReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("FetchConfigChangeReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchConfigChangeReport", "running workflow");
        let outgoing_0 = configuration::ResolveEnvironmentConflictRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::ResolveEnvironmentConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_environment_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::SimulateCertificateRequest {
            certificate: Some(certificates::Certificate {
                lines: vec![certificates::CertificateLine {
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::SimulateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::FetchConfigChangeReportResponse::default();
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
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleConfigChangeReport after validating the tenant and durably recording the command.
    async fn run_schedule_config_change_report(
        &self,
        req: configuration::ScheduleConfigChangeReportRequest,
    ) -> Result<configuration::ScheduleConfigChangeReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleConfigChangeReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleConfigChangeReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = telehealth::SimulateVideoVisitRequest {
            context: req.context.clone(),
            video_visit: Some(telehealth::VideoVisit {
                organizer: Some(identity::UserRef {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VideoVisitCalculationServiceClient::new(channel);
            let input: tonic::Request<telehealth::SimulateVideoVisitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_video_visit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::ResolveCertificatesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::ResolveCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = authorization::AnnotateRoleBindingVersionRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .video_visit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_by: reply_0
                        .video_visit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    locality: reply_0
                        .video_visit
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    recipient: reply_0
                        .video_visit
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RoleBindingHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::AnnotateRoleBindingVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.annotate_role_binding_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::NormalizeConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                config_change_id: req.config_change_id.clone(),
                correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                subject: Some(configuration::ConfigSetRef {
                    display_name: reply_0
                        .video_visit
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
            context: req.context.clone(),
            details: Some(configuration::normalize_config_change_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .video_visit
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
        let reply_3 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::NormalizeConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ScheduleConfigChangeReportResponse::default();
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run StreamConfigChangeMetrics after validating the tenant and durably recording the command.
    async fn run_stream_config_change_metrics(
        &self,
        req: configuration::StreamConfigChangeMetricsRequest,
    ) -> Result<configuration::StreamConfigChangeMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamConfigChangeMetrics", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamConfigChangeMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = warehousing::ResolveWarehousesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = WarehouseQueryServiceClient::new(channel);
            let input: tonic::Request<warehousing::ResolveWarehousesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_warehouses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::ScoreSecretVersionRequest {
            context: req.context.clone(),
            details: Some(secrets::score_secret_version_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret_version: Some(secrets::SecretVersion {
                correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SecretVersionValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::ScoreSecretVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_secret_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = payroll::UpdatePayRunRequest {
            context: req.context.clone(),
            pay_run: Some(payroll::PayRun {
                billing_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                counterparty: Some(payroll::DeductionRef {
                    display_name: reply_1
                        .secret_version
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = PayRunServiceClient::new(channel);
            let input: tonic::Request<payroll::UpdatePayRunRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_pay_run(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::StreamConfigChangeMetricsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1.secret_version.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).kind = contribution;
        }
        let contribution: String = reply_1
            .secret_version
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ListConfigChangeReports after validating the tenant and durably recording the command.
    async fn run_list_config_change_reports(
        &self,
        req: configuration::ListConfigChangeReportsRequest,
    ) -> Result<configuration::ListConfigChangeReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListConfigChangeReports", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListConfigChangeReports", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = configuration::ListConfigChangeReportsResponse::default();
        Ok(response)
    }
    /// Run ShareConfigChangeReport after validating the tenant and durably recording the command.
    async fn run_share_config_change_report(
        &self,
        req: configuration::ShareConfigChangeReportRequest,
    ) -> Result<configuration::ShareConfigChangeReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ShareConfigChangeReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareConfigChangeReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = configuration::PreviewConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .recipient
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
            details: Some(configuration::preview_config_set_request::Details {
                contact: Some(common::ContactInfo {
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
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
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::PreviewConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::SimulateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
                        .config_set
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
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(secrets::simulate_access_grant_request::Details {
                destination: Some(common::PostalAddress {
                    region: reply_0
                        .config_set
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::SimulateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ShareConfigChangeReportResponse::default();
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        Ok(response)
    }
    /// Run CountConfigChangeTotals after validating the tenant and durably recording the command.
    async fn run_count_config_change_totals(
        &self,
        req: configuration::CountConfigChangeTotalsRequest,
    ) -> Result<configuration::CountConfigChangeTotalsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CountConfigChangeTotals", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountConfigChangeTotals", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = search::SuggestSearchIndexesRequest {
            context: req.context.clone(),
            page: Some(common::PageRequest {
                page_size: req.page.clone().unwrap_or_default().page_size.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SearchIndexQueryServiceClient::new(channel);
            let input: tonic::Request<search::SuggestSearchIndexesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_search_indexes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::CountConfigChangeTotalsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run DeleteConfigChangeReport after validating the tenant and durably recording the command.
    async fn run_delete_config_change_report(
        &self,
        req: configuration::DeleteConfigChangeReportRequest,
    ) -> Result<configuration::DeleteConfigChangeReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DeleteConfigChangeReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteConfigChangeReport", "running workflow");
        tracing::debug!(audit_value = ?req.config_change_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PreviewThrottleRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PreviewThrottleRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::DeleteConfigChangeReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        Ok(response)
    }
}
acme_platform::endpoints! { ConfigChangeReportingService, ConfigChangeReportingHandler;
unary {
generate_config_change_report (configuration::GenerateConfigChangeReportRequest) -> configuration::GenerateConfigChangeReportResponse => run_generate_config_change_report;
fetch_config_change_report (configuration::FetchConfigChangeReportRequest) -> configuration::FetchConfigChangeReportResponse => run_fetch_config_change_report;
schedule_config_change_report (configuration::ScheduleConfigChangeReportRequest) -> configuration::ScheduleConfigChangeReportResponse => run_schedule_config_change_report;
list_config_change_reports (configuration::ListConfigChangeReportsRequest) -> configuration::ListConfigChangeReportsResponse => run_list_config_change_reports;
share_config_change_report (configuration::ShareConfigChangeReportRequest) -> configuration::ShareConfigChangeReportResponse => run_share_config_change_report;
count_config_change_totals (configuration::CountConfigChangeTotalsRequest) -> configuration::CountConfigChangeTotalsResponse => run_count_config_change_totals;
delete_config_change_report (configuration::DeleteConfigChangeReportRequest) -> configuration::DeleteConfigChangeReportResponse => run_delete_config_change_report;
}
server {
stream_config_change_metrics (configuration::StreamConfigChangeMetricsRequest) -> configuration::StreamConfigChangeMetricsResponse, StreamConfigChangeMetricsStream => run_stream_config_change_metrics;
}
client {
}
bidi {
}
}
