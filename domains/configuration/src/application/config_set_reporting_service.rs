use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_reporting_service_server::ConfigSetReportingService;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::deliveries::v1::delivery_query_service_client::DeliveryQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::patients::v1::patient_service_client::PatientServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::sellers::v1 as sellers;
use acme_proto::acme::sellers::v1::seller_query_service_client::SellerQueryServiceClient;

/// Builds and schedules reports about config sets.
#[derive(Clone)]
pub struct ConfigSetReportingEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    delete_config_set_report_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::CalculateConfigSetRequest,
            configuration::CalculateConfigSetResponse,
        >,
    >,
    fetch_config_set_report_0_port: acme_platform::ports::Callback<
        configuration::ExplainEnvironmentDecisionRequest,
        configuration::ExplainEnvironmentDecisionResponse,
    >,
    share_config_set_report_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::NormalizeEnvironmentRequest,
            configuration::NormalizeEnvironmentResponse,
        >,
    >,
    share_config_set_report_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::GetConfigSetRatesRequest,
                configuration::GetConfigSetRatesResponse,
            >,
    >,
    stream_config_set_metrics_0_port: acme_platform::ports::Callback<
        configuration::SimulateConfigSetRequest,
        configuration::SimulateConfigSetResponse,
    >,
    stream_config_set_metrics_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<deliveries::ResolveDeliveriesRequest, deliveries::ResolveDeliveriesResponse>,
    >,
}
impl ConfigSetReportingEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::ExplainEnvironmentDecisionRequest,
            configuration::ExplainEnvironmentDecisionResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ExplainEnvironmentDecisionRequest| -> acme_platform::ports::RpcFuture<configuration::ExplainEnvironmentDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_environment_decision(request).await?.into_inner())
})
})
        };
        let fetch_config_set_report_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::SimulateConfigSetRequest,
            configuration::SimulateConfigSetResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SimulateConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::SimulateConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_config_set(request).await?.into_inner())
})
})
        };
        let stream_config_set_metrics_0_port = callback;
        let callback: acme_platform::ports::Callback<
            deliveries::ResolveDeliveriesRequest,
            deliveries::ResolveDeliveriesResponse,
        > = {
            let client = DeliveryQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: deliveries::ResolveDeliveriesRequest| -> acme_platform::ports::RpcFuture<deliveries::ResolveDeliveriesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_deliveries(request).await?.into_inner())
})
})
        };
        let stream_config_set_metrics_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::NormalizeEnvironmentRequest,
            configuration::NormalizeEnvironmentResponse,
        > = {
            let client = EnvironmentValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::NormalizeEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::NormalizeEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_environment(request).await?.into_inner())
})
})
        };
        let share_config_set_report_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::GetConfigSetRatesRequest,
            configuration::GetConfigSetRatesResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::GetConfigSetRatesRequest| -> acme_platform::ports::RpcFuture<configuration::GetConfigSetRatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.get_config_set_rates(request).await?.into_inner())
})
})
        };
        let share_config_set_report_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::CalculateConfigSetRequest,
            configuration::CalculateConfigSetResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CalculateConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::CalculateConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_config_set(request).await?.into_inner())
})
})
        };
        let delete_config_set_report_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            delete_config_set_report_0_port,
            fetch_config_set_report_0_port,
            share_config_set_report_0_port,
            share_config_set_report_1_port,
            stream_config_set_metrics_0_port,
            stream_config_set_metrics_1_port,
        }
    }
    /// Run GenerateConfigSetReport after validating the tenant and durably recording the command.
    async fn run_generate_config_set_report(
        &self,
        req: configuration::GenerateConfigSetReportRequest,
    ) -> Result<configuration::GenerateConfigSetReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GenerateConfigSetReport", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GenerateConfigSetReport", "running workflow");
        tracing::debug!(audit_value = ?req.config_set_id.clone(), "recorded audit field");
        let outgoing_0 = audit::ResolveAuditEventsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<audit::ResolveAuditEventsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AuditEventQueryServiceClient::new(channel);
            let input: tonic::Request<audit::ResolveAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = GenerateConfigSetReportDispatch1::from(req.clone());
        command_1.config_change_correlation_id = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        let outgoing_1: configuration::CompareConfigChangeOptionsRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CompareConfigChangeOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_config_change_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = secrets::CalculateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                owner: Some(identity::UserRef {
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::codec::through_any(&outgoing_2, "acme.secrets.v1.CalculateAccessGrantRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::CalculateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = patients::BatchCreatePatientsRequest {
            context: req.context.clone(),
            chunk: Some(patients::batch_create_patients_request::Chunk::Record(
                patients::Patient {
                    audit: Some(common::AuditInfo {
                        updated_at: reply_2
                            .access_grant
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
        let reply_3 = async move {
            let mut client = PatientServiceClient::new(channel);
            let input: tonic::Request<patients::BatchCreatePatientsRequest> =
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
            let response = client.batch_create_patients(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    action: Some(featureflags::feature_flag::Action::Block(
                        featureflags::FeatureFlagBlockAction {
                            reason: (match reply_2.access_grant.clone().unwrap_or_default().action.clone() {
                                Some(secrets::access_grant::Action::Block(value)) => value,
                                _ => Default::default(),
                            })
                            .reason
                            .clone(),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
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
        let mut response = configuration::GenerateConfigSetReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run FetchConfigSetReport after validating the tenant and durably recording the command.
    async fn run_fetch_config_set_report(
        &self,
        req: configuration::FetchConfigSetReportRequest,
    ) -> Result<configuration::FetchConfigSetReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FetchConfigSetReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FetchConfigSetReport", "running workflow");
        let outgoing_0 = configuration::ExplainEnvironmentDecisionRequest {
            context: req.context.clone(),
            details: Some(configuration::explain_environment_decision_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
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
        let port = acme_platform::ports::opaque(self.fetch_config_set_report_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = configuration::FetchConfigSetReportResponse::default();
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamConfigSetMetrics after validating the tenant and durably recording the command.
    async fn run_stream_config_set_metrics(
        &self,
        req: configuration::StreamConfigSetMetricsRequest,
    ) -> Result<configuration::StreamConfigSetMetricsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StreamConfigSetMetrics", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamConfigSetMetrics", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            region: req.filter.clone().unwrap_or_default().region.clone(),
                            ..Default::default()
                        }),
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
                    manager: Some(identity::UserRef {
                        email: req.subscriber.clone().unwrap_or_default().email.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = (self.stream_config_set_metrics_0_port)(outgoing_0).await?;
        let outgoing_1 = deliveries::ResolveDeliveriesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.stream_config_set_metrics_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = configuration::StreamConfigSetMetricsResponse::default();
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.config_set.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: i32 = reply_0.config_set.clone().unwrap_or_default().depth.clone();
        if contribution != <i32>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].depth = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListConfigSetReports after validating the tenant and durably recording the command.
    async fn run_list_config_set_reports(
        &self,
        req: configuration::ListConfigSetReportsRequest,
    ) -> Result<configuration::ListConfigSetReportsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListConfigSetReports", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListConfigSetReports", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().owner_id.clone(), "recorded audit field");
        let outgoing_0 = sellers::SuggestSellersRequest {
            context: req.context.clone(),
            filter: Some(sellers::suggest_sellers_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_token: req.page.clone().unwrap_or_default().page_token.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SellerQueryServiceClient::new(channel);
            let input: tonic::Request<sellers::SuggestSellersRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_sellers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::CalculateKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    location: Some(common::PostalAddress {
                        region: req.filter.clone().unwrap_or_default().region.clone(),
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
            let mut client = KeyRingCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::CalculateKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ListConfigSetReportsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response.config_sets.is_empty() {
                response.config_sets.push(Default::default());
            }
            if response.config_sets[0].children.is_empty() {
                response.config_sets[0].children.push(Default::default());
            }
            if response.config_sets[0].children[0].children.is_empty() {
                response.config_sets[0].children[0].children.push(Default::default());
            }
            response.config_sets[0].children[0].children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ShareConfigSetReport after validating the tenant and durably recording the command.
    async fn run_share_config_set_report(
        &self,
        req: configuration::ShareConfigSetReportRequest,
    ) -> Result<configuration::ShareConfigSetReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ShareConfigSetReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ShareConfigSetReport", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().template_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::NormalizeEnvironmentRequest {
            context: req.context.clone(),
            details: Some(configuration::normalize_environment_request::Details {
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
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
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
            environment: Some(configuration::Environment {
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
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.share_config_set_report_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::GetConfigSetRatesRequest {
            config_set_id: req.config_set_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.share_config_set_report_1_port.as_ref(), outgoing_1).await?;
        let mut response = configuration::ShareConfigSetReportResponse::default();
        let contribution: String = (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .locality
        .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.config_set.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
        }
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.config_set.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response
                .config_set
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .config_set
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
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
                .config_set
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run CountConfigSetTotals after validating the tenant and durably recording the command.
    async fn run_count_config_set_totals(
        &self,
        req: configuration::CountConfigSetTotalsRequest,
    ) -> Result<configuration::CountConfigSetTotalsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountConfigSetTotals", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountConfigSetTotals", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = configuration::CountConfigSetTotalsResponse::default();
        Ok(response)
    }
    /// Run DeleteConfigSetReport after validating the tenant and durably recording the command.
    async fn run_delete_config_set_report(
        &self,
        req: configuration::DeleteConfigSetReportRequest,
    ) -> Result<configuration::DeleteConfigSetReportResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteConfigSetReport", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteConfigSetReport", "running workflow");
        tracing::debug!(audit_value = ?req.config_set_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CalculateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        etag: req.etag.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.delete_config_set_report_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = configuration::DeleteConfigSetReportResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by GenerateConfigSetReport for CompareConfigChangeOptions.
struct GenerateConfigSetReportDispatch1 {
    context: Option<common::RequestContext>,
    config_change_correlation_id: String,
}
impl From<configuration::GenerateConfigSetReportRequest> for GenerateConfigSetReportDispatch1 {
    fn from(req: configuration::GenerateConfigSetReportRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_change_correlation_id: Default::default(),
        }
    }
}
impl From<GenerateConfigSetReportDispatch1> for configuration::CompareConfigChangeOptionsRequest {
    fn from(value: GenerateConfigSetReportDispatch1) -> Self {
        configuration::CompareConfigChangeOptionsRequest {
            config_change: Some(configuration::ConfigChange {
                correlation_id: value.config_change_correlation_id,
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigSetReportingService for ConfigSetReportingEndpoint {
    /// Generates a config set.
    async fn generate_config_set_report(
        &self,
        request: tonic::Request<configuration::GenerateConfigSetReportRequest>,
    ) -> Result<tonic::Response<configuration::GenerateConfigSetReportResponse>, tonic::Status> {
        self.run_generate_config_set_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config set.
    async fn fetch_config_set_report(
        &self,
        request: tonic::Request<configuration::FetchConfigSetReportRequest>,
    ) -> Result<tonic::Response<configuration::FetchConfigSetReportResponse>, tonic::Status> {
        self.run_fetch_config_set_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamConfigSetMetricsStream =
        tokio_stream::wrappers::ReceiverStream<Result<configuration::StreamConfigSetMetricsResponse, tonic::Status>>;
    /// Streams the changes of config sets as they happen.
    async fn stream_config_set_metrics(
        &self,
        request: tonic::Request<configuration::StreamConfigSetMetricsRequest>,
    ) -> Result<tonic::Response<Self::StreamConfigSetMetricsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_stream_config_set_metrics(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Lists config sets a page at a time.
    async fn list_config_set_reports(
        &self,
        request: tonic::Request<configuration::ListConfigSetReportsRequest>,
    ) -> Result<tonic::Response<configuration::ListConfigSetReportsResponse>, tonic::Status> {
        self.run_list_config_set_reports(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Shares a config set.
    async fn share_config_set_report(
        &self,
        request: tonic::Request<configuration::ShareConfigSetReportRequest>,
    ) -> Result<tonic::Response<configuration::ShareConfigSetReportResponse>, tonic::Status> {
        self.run_share_config_set_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts config sets by status.
    async fn count_config_set_totals(
        &self,
        request: tonic::Request<configuration::CountConfigSetTotalsRequest>,
    ) -> Result<tonic::Response<configuration::CountConfigSetTotalsResponse>, tonic::Status> {
        self.run_count_config_set_totals(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a config set.
    async fn delete_config_set_report(
        &self,
        request: tonic::Request<configuration::DeleteConfigSetReportRequest>,
    ) -> Result<tonic::Response<configuration::DeleteConfigSetReportResponse>, tonic::Status> {
        self.run_delete_config_set_report(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
