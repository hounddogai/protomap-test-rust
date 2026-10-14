use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::admissions::v1::admission_service_client::AdmissionServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_query_service_server::ConfigSetQueryService;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_query_service_client::DocumentQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;

/// Answers read-only questions about config sets, for screens and reports.
#[derive(Clone)]
pub struct ConfigSetQueryApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    browse_config_sets_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<documents::FindDocumentsRequest, documents::FindDocumentsResponse>,
    >,
    count_config_sets_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::ExplainEnvironmentDecisionRequest,
                configuration::ExplainEnvironmentDecisionResponse,
            >,
    >,
    count_config_sets_1_port: acme_platform::ports::Callback<
        configuration::CheckConfigChangeEligibilityRequest,
        configuration::CheckConfigChangeEligibilityResponse,
    >,
    export_config_sets_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::LintConfigChangesRequest,
            configuration::LintConfigChangesResponse,
        >,
    >,
    export_config_sets_1_port: acme_platform::ports::Callback<
        configuration::NormalizeConfigChangeRequest,
        configuration::NormalizeConfigChangeResponse,
    >,
    summarize_config_sets_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<documents::CountDocumentsRequest, documents::CountDocumentsResponse>,
    >,
    summarize_config_sets_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::LintConfigChangesRequest,
                configuration::LintConfigChangesResponse,
            >,
    >,
}
impl ConfigSetQueryApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            documents::FindDocumentsRequest,
            documents::FindDocumentsResponse,
        > = {
            let client = DocumentQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: documents::FindDocumentsRequest| -> acme_platform::ports::RpcFuture<documents::FindDocumentsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.find_documents(request).await?.into_inner())
})
})
        };
        let browse_config_sets_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let count_config_sets_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::CheckConfigChangeEligibilityRequest,
            configuration::CheckConfigChangeEligibilityResponse,
        > = {
            let client = ConfigChangeValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CheckConfigChangeEligibilityRequest| -> acme_platform::ports::RpcFuture<configuration::CheckConfigChangeEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_config_change_eligibility(request).await?.into_inner())
})
})
        };
        let count_config_sets_1_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::LintConfigChangesRequest,
            configuration::LintConfigChangesResponse,
        > = {
            let client = ConfigChangeValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::LintConfigChangesRequest| -> acme_platform::ports::RpcFuture<configuration::LintConfigChangesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_config_changes(request).await?.into_inner())
})
})
        };
        let export_config_sets_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::NormalizeConfigChangeRequest,
            configuration::NormalizeConfigChangeResponse,
        > = {
            let client = ConfigChangeValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::NormalizeConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::NormalizeConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_config_change(request).await?.into_inner())
})
})
        };
        let export_config_sets_1_port = callback;
        let callback: acme_platform::ports::Callback<
            documents::CountDocumentsRequest,
            documents::CountDocumentsResponse,
        > = {
            let client = DocumentQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: documents::CountDocumentsRequest| -> acme_platform::ports::RpcFuture<documents::CountDocumentsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_documents(request).await?.into_inner())
})
})
        };
        let summarize_config_sets_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::LintConfigChangesRequest,
            configuration::LintConfigChangesResponse,
        > = {
            let client = ConfigChangeValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::LintConfigChangesRequest| -> acme_platform::ports::RpcFuture<configuration::LintConfigChangesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_config_changes(request).await?.into_inner())
})
})
        };
        let summarize_config_sets_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            browse_config_sets_0_port,
            count_config_sets_0_port,
            count_config_sets_1_port,
            export_config_sets_0_port,
            export_config_sets_1_port,
            summarize_config_sets_1_port,
            summarize_config_sets_2_port,
        }
    }
    /// Run LookupConfigSet after validating the tenant and durably recording the command.
    async fn run_lookup_config_set(
        &self,
        req: configuration::LookupConfigSetRequest,
    ) -> Result<configuration::LookupConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupConfigSet", "running workflow");
        let outgoing_0 = notifications::ListNotificationsRequest {
            context: req.context.clone(),
            filter: Some(notifications::list_notifications_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.config_set_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<notifications::ListNotificationsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = NotificationServiceClient::new(channel);
            let input: tonic::Request<notifications::ListNotificationsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_notifications(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = LookupConfigSetDispatch1::from(req.clone());
        command_1.config_change_correlation_id = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        let outgoing_1: configuration::ExplainConfigChangeDecisionRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainConfigChangeDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_config_change_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = admissions::BatchCreateAdmissionsRequest {
            context: req.context.clone(),
            chunk: Some(admissions::batch_create_admissions_request::Chunk::Record(
                admissions::Admission {
                    correlation_id: reply_1.config_change.clone().unwrap_or_default().correlation_id.clone(),
                    kind: reply_1.config_change.clone().unwrap_or_default().kind.clone(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AdmissionServiceClient::new(channel);
            let input: tonic::Request<admissions::BatchCreateAdmissionsRequest> =
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
            let response = client.batch_create_admissions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Ack(
                featureflags::stream_flag_rollout_changes_request::ChangeAck {
                    sequence: reply_1.config_change.clone().unwrap_or_default().sequence.clone(),
                    status: reply_1.outcome.clone().unwrap_or_default().status.clone(),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
        let mut response = configuration::LookupConfigSetResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.config_change.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .attributes
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseConfigSets after validating the tenant and durably recording the command.
    async fn run_browse_config_sets(
        &self,
        req: configuration::BrowseConfigSetsRequest,
    ) -> Result<configuration::BrowseConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = documents::FindDocumentsRequest {
            context: req.context.clone(),
            filter: Some(documents::find_documents_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.browse_config_sets_0_port.as_ref(), outgoing_0).await?;
        let mut response = configuration::BrowseConfigSetsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CountConfigSets after validating the tenant and durably recording the command.
    async fn run_count_config_sets(
        &self,
        req: configuration::CountConfigSetsRequest,
    ) -> Result<configuration::CountConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::ExplainEnvironmentDecisionRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.count_config_sets_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::CheckConfigChangeEligibilityRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
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
                subject: Some(configuration::ConfigSetRef {
                    display_name: reply_0
                        .environment
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
            }),
            context: req.context.clone(),
            details: Some(configuration::check_config_change_eligibility_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .owner
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
        let reply_1 = (self.count_config_sets_1_port)(outgoing_1).await?;
        let mut response = configuration::CountConfigSetsResponse::default();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ResolveConfigSets after validating the tenant and durably recording the command.
    async fn run_resolve_config_sets(
        &self,
        req: configuration::ResolveConfigSetsRequest,
    ) -> Result<configuration::ResolveConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveConfigSets", "running workflow");
        let outgoing_0 = configuration::NormalizeConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
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
            details: Some(configuration::normalize_config_set_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::NormalizeConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ResolveConfigSetsResponse::default();
        let contribution: i32 = reply_0.config_set.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            if response.config_sets.is_empty() {
                response.config_sets.push(Default::default());
            }
            if response.config_sets[0].children.is_empty() {
                response.config_sets[0].children.push(Default::default());
            }
            if response.config_sets[0].children[0].children.is_empty() {
                response.config_sets[0].children[0].children.push(Default::default());
            }
            response.config_sets[0].children[0].children[0].status = contribution;
        }
        Ok(response)
    }
    /// Run ExportConfigSets after validating the tenant and durably recording the command.
    async fn run_export_config_sets(
        &self,
        req: configuration::ExportConfigSetsRequest,
    ) -> Result<configuration::ExportConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = configuration::LintConfigChangesRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    observed_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::lint_config_changes_request::Details {
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
        let reply_0 =
            self.export_config_sets_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = configuration::NormalizeConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    config_set_id: reply_0
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .config_set_id
                        .clone(),
                    path: reply_0
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .path
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.export_config_sets_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = configuration::ExportConfigSetsResponse::default();
        let contribution: i64 = reply_0.config_change.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.sequence = contribution;
        }
        let contribution: String = reply_0
            .config_change
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .config_set_id
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
            if response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].children[0].children[0]
                .config_set_id = contribution;
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
            reply_1.config_change.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
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
                .attributes
                .extend(contribution);
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
        Ok(response)
    }
    /// Run SummarizeConfigSets after validating the tenant and durably recording the command.
    async fn run_summarize_config_sets(
        &self,
        req: configuration::SummarizeConfigSetsRequest,
    ) -> Result<configuration::SummarizeConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = configuration::CheckConfigChangeEligibilityRequest {
            annotations: req.annotations.clone(),
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    config_set_id: req.config_set_id.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::CheckConfigChangeEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_config_change_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = documents::CountDocumentsRequest {
            context: req.context.clone(),
            filter: Some(documents::count_documents_request::Filter {
                created_after: reply_0.config_change.clone().unwrap_or_default().occurred_at.clone(),
                labels: reply_0.config_change.clone().unwrap_or_default().attributes.clone(),
                owner_id: reply_0.outcome.clone().unwrap_or_default().reference_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.summarize_config_sets_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::LintConfigChangesRequest {
            config_change: Some(configuration::ConfigChange {
                correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                kind: reply_0.config_change.clone().unwrap_or_default().kind.clone(),
                occurred_at: reply_0.config_change.clone().unwrap_or_default().occurred_at.clone(),
                subject: reply_0.config_change.clone().unwrap_or_default().subject.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.summarize_config_sets_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = configuration::CompareConfigSetOptionsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        attributes: (reply_0.config_change.clone().unwrap_or_default().attributes.clone())
                            .into_iter()
                            .filter(|(key, _)| !key.is_empty())
                            .map(|(key, value)| (key, value.trim().to_owned()))
                            .collect(),
                        config_set_id: reply_2
                            .config_change
                            .clone()
                            .unwrap_or_default()
                            .subject
                            .clone()
                            .unwrap_or_default()
                            .config_set_id
                            .clone(),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        display_name: reply_0
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CompareConfigSetOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_config_set_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::SummarizeConfigSetsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run PeekConfigSet after validating the tenant and durably recording the command.
    async fn run_peek_config_set(
        &self,
        req: configuration::PeekConfigSetRequest,
    ) -> Result<configuration::PeekConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekConfigSet", "running workflow");
        let outgoing_0 = configuration::CompareEnvironmentOptionsRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
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
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = EnvironmentCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CompareEnvironmentOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_environment_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = configuration::PeekConfigSetResponse::default();
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by LookupConfigSet for ExplainConfigChangeDecision.
struct LookupConfigSetDispatch1 {
    context: Option<common::RequestContext>,
    config_change_correlation_id: String,
    config_change_subject_config_set_id: String,
}
impl From<configuration::LookupConfigSetRequest> for LookupConfigSetDispatch1 {
    fn from(req: configuration::LookupConfigSetRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_change_correlation_id: Default::default(),
            config_change_subject_config_set_id: req.config_set_id.clone(),
        }
    }
}
impl From<LookupConfigSetDispatch1> for configuration::ExplainConfigChangeDecisionRequest {
    fn from(value: LookupConfigSetDispatch1) -> Self {
        configuration::ExplainConfigChangeDecisionRequest {
            config_change: Some(configuration::ConfigChange {
                correlation_id: value.config_change_correlation_id,
                subject: Some(configuration::ConfigSetRef {
                    config_set_id: value.config_change_subject_config_set_id,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigSetQueryService for ConfigSetQueryApi {
    /// Returns a config set.
    async fn lookup_config_set(
        &self,
        request: tonic::Request<configuration::LookupConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::LookupConfigSetResponse>, tonic::Status> {
        self.run_lookup_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists config sets a page at a time.
    async fn browse_config_sets(
        &self,
        request: tonic::Request<configuration::BrowseConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::BrowseConfigSetsResponse>, tonic::Status> {
        self.run_browse_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts config sets by status.
    async fn count_config_sets(
        &self,
        request: tonic::Request<configuration::CountConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::CountConfigSetsResponse>, tonic::Status> {
        self.run_count_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several config sets at once.
    async fn resolve_config_sets(
        &self,
        request: tonic::Request<configuration::ResolveConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::ResolveConfigSetsResponse>, tonic::Status> {
        self.run_resolve_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportConfigSetsStream =
        tokio_stream::wrappers::ReceiverStream<Result<configuration::ExportConfigSetsResponse, tonic::Status>>;
    /// Streams every matching config set.
    async fn export_config_sets(
        &self,
        request: tonic::Request<configuration::ExportConfigSetsRequest>,
    ) -> Result<tonic::Response<Self::ExportConfigSetsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_export_config_sets(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a config set.
    async fn summarize_config_sets(
        &self,
        request: tonic::Request<configuration::SummarizeConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::SummarizeConfigSetsResponse>, tonic::Status> {
        self.run_summarize_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config set.
    async fn peek_config_set(
        &self,
        request: tonic::Request<configuration::PeekConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::PeekConfigSetResponse>, tonic::Status> {
        self.run_peek_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
