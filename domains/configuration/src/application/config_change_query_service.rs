use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_notification_service_client::AccessPolicyNotificationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_query_service_server::ConfigChangeQueryService;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_service_client::DocumentServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Answers read-only questions about config changes, for screens and reports.
#[derive(Clone)]
pub struct ConfigChangeQueryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    lookup_config_change_0_port:
        std::sync::Arc<dyn acme_platform::ports::RpcPort<venues::BrowseVenuesRequest, venues::BrowseVenuesResponse>>,
    peek_config_change_0_port: acme_platform::ports::Callback<
        configuration::CheckConfigSetEligibilityRequest,
        configuration::CheckConfigSetEligibilityResponse,
    >,
}
impl ConfigChangeQueryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<venues::BrowseVenuesRequest, venues::BrowseVenuesResponse> = {
            let client = VenueQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: venues::BrowseVenuesRequest| -> acme_platform::ports::RpcFuture<venues::BrowseVenuesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_venues(request).await?.into_inner())
})
})
        };
        let lookup_config_change_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::CheckConfigSetEligibilityRequest,
            configuration::CheckConfigSetEligibilityResponse,
        > = {
            let client = ConfigSetValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CheckConfigSetEligibilityRequest| -> acme_platform::ports::RpcFuture<configuration::CheckConfigSetEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_config_set_eligibility(request).await?.into_inner())
})
})
        };
        let peek_config_change_0_port = callback;
        Self {
            channel,
            repository,
            lookup_config_change_0_port,
            peek_config_change_0_port,
        }
    }
    /// Run LookupConfigChange after validating the tenant and durably recording the command.
    async fn run_lookup_config_change(
        &self,
        req: configuration::LookupConfigChangeRequest,
    ) -> Result<configuration::LookupConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupConfigChange", "running workflow");
        let outgoing_0 = venues::BrowseVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::browse_venues_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.config_change_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.lookup_config_change_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = documents::BatchCreateDocumentsRequest {
            context: req.context.clone(),
            chunk: Some(documents::batch_create_documents_request::Chunk::Header(
                documents::batch_create_documents_request::StreamHeader {
                    source: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                    submitted_by: Some(common::ContactInfo {
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = DocumentServiceClient::new(channel);
            let input: tonic::Request<documents::BatchCreateDocumentsRequest> =
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
            let response = client.batch_create_documents(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                    measurements: vec![featureflags::FlagEvaluationMeasurement {
                        observed_at: req.context.clone().unwrap_or_default().deadline.clone(),
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
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
        let mut response = configuration::LookupConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run BrowseConfigChanges after validating the tenant and durably recording the command.
    async fn run_browse_config_changes(
        &self,
        req: configuration::BrowseConfigChangesRequest,
    ) -> Result<configuration::BrowseConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseConfigChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseConfigChanges", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = audit::CountAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::count_audit_events_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_size: req.page.clone().unwrap_or_default().page_size.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<audit::CountAuditEventsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = AuditEventQueryServiceClient::new(channel);
                let input: tonic::Request<audit::CountAuditEventsRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_audit_events(input).await?;
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
        let mut response = configuration::BrowseConfigChangesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run FindConfigChanges after validating the tenant and durably recording the command.
    async fn run_find_config_changes(
        &self,
        req: configuration::FindConfigChangesRequest,
    ) -> Result<configuration::FindConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindConfigChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindConfigChanges", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::ExplainConfigSetCalculationRequest {
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
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    location: Some(common::PostalAddress {
                        region: req.filter.clone().unwrap_or_default().region.clone(),
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
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainConfigSetCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_config_set_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::StreamAccessPolicyNoticesRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
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
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = AccessPolicyNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::StreamAccessPolicyNoticesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_access_policy_notices(input).await?;
            let mut frames = response.into_inner();
            let mut last = authorization::StreamAccessPolicyNoticesResponse::default();
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
        let mut response = configuration::FindConfigChangesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_set.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            if response.config_changes.is_empty() {
                response.config_changes.push(Default::default());
            }
            response.config_changes[0].attributes.extend(contribution);
        }
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
            if response.config_changes.is_empty() {
                response.config_changes.push(Default::default());
            }
            response.config_changes[0]
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run CountConfigChanges after validating the tenant and durably recording the command.
    async fn run_count_config_changes(
        &self,
        req: configuration::CountConfigChangesRequest,
    ) -> Result<configuration::CountConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountConfigChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountConfigChanges", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = configuration::CountConfigChangesResponse::default();
        Ok(response)
    }
    /// Run ResolveConfigChanges after validating the tenant and durably recording the command.
    async fn run_resolve_config_changes(
        &self,
        req: configuration::ResolveConfigChangesRequest,
    ) -> Result<configuration::ResolveConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveConfigChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveConfigChanges", "running workflow");
        let response = configuration::ResolveConfigChangesResponse::default();
        Ok(response)
    }
    /// Run ExportConfigChanges after validating the tenant and durably recording the command.
    async fn run_export_config_changes(
        &self,
        req: configuration::ExportConfigChangesRequest,
    ) -> Result<configuration::ExportConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportConfigChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportConfigChanges", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = configuration::EstimateEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                })),
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
            let input: tonic::Request<configuration::EstimateEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = authorization::ListAccessPolicySubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_access_policy_subscriptions_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessPolicyNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::ListAccessPolicySubscriptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_access_policy_subscriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ExportConfigChangesResponse::default();
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
        Ok(response)
    }
    /// Run SummarizeConfigChanges after validating the tenant and durably recording the command.
    async fn run_summarize_config_changes(
        &self,
        req: configuration::SummarizeConfigChangesRequest,
    ) -> Result<configuration::SummarizeConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeConfigChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeConfigChanges", "running workflow");
        tracing::debug!(audit_value = ?req.config_change_id.clone(), "recorded audit field");
        let command_0 = SummarizeConfigChangesDispatch0::from(req.clone());
        let outgoing_0: configuration::ExplainConfigSetCalculationRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainConfigSetCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_config_set_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::EstimateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            etag: reply_0
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
                                .clone(),
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
                        location: Some(common::PostalAddress {
                            recipient: reply_0
                                .config_set
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.configuration.v1.EstimateConfigSetRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::EstimateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::SummarizeConfigChangesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
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
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run PeekConfigChange after validating the tenant and durably recording the command.
    async fn run_peek_config_change(
        &self,
        req: configuration::PeekConfigChangeRequest,
    ) -> Result<configuration::PeekConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekConfigChange", "running workflow");
        let outgoing_0 = configuration::CheckConfigSetEligibilityRequest {
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
        let port = acme_platform::ports::opaque(self.peek_config_change_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = authorization::SendAccessPolicyTestNoticeRequest {
            annotations: reply_0.result.clone().unwrap_or_default().details.clone(),
            context: req.context.clone(),
            details: Some(authorization::send_access_policy_test_notice_request::Details {
                recipient: Some(common::ContactInfo {
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
                variables: reply_0.config_set.clone().unwrap_or_default().attributes.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<authorization::SendAccessPolicyTestNoticeRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessPolicyNotificationServiceClient::new(channel);
            let input: tonic::Request<authorization::SendAccessPolicyTestNoticeRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.send_access_policy_test_notice(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::PeekConfigChangeResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_set.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        let contribution: String = reply_1
            .access_policy
            .clone()
            .unwrap_or_default()
            .owner
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
}
/// The domain command prepared by SummarizeConfigChanges for ExplainConfigSetCalculation.
struct SummarizeConfigChangesDispatch0 {
    context: Option<common::RequestContext>,
    config_set_children_children_manager_user_id: String,
    config_set_children_manager_email: String,
    annotations: std::collections::BTreeMap<String, String>,
    details_locale: String,
}
impl From<configuration::SummarizeConfigChangesRequest> for SummarizeConfigChangesDispatch0 {
    fn from(req: configuration::SummarizeConfigChangesRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_children_manager_user_id: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            config_set_children_manager_email: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            annotations: req.annotations.clone(),
            details_locale: req.context.clone().unwrap_or_default().locale.clone(),
        }
    }
}
impl From<SummarizeConfigChangesDispatch0> for configuration::ExplainConfigSetCalculationRequest {
    fn from(value: SummarizeConfigChangesDispatch0) -> Self {
        configuration::ExplainConfigSetCalculationRequest {
            annotations: value.annotations,
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            user_id: value.config_set_children_children_manager_user_id,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        email: value.config_set_children_manager_email,
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: value.context,
            details: Some(configuration::explain_config_set_calculation_request::Details {
                locale: value.details_locale,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigChangeQueryService for ConfigChangeQueryEndpoint {
    /// Returns a config change.
    async fn lookup_config_change(
        &self,
        request: tonic::Request<configuration::LookupConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::LookupConfigChangeResponse>, tonic::Status> {
        self.run_lookup_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists config changes a page at a time.
    async fn browse_config_changes(
        &self,
        request: tonic::Request<configuration::BrowseConfigChangesRequest>,
    ) -> Result<tonic::Response<configuration::BrowseConfigChangesResponse>, tonic::Status> {
        self.run_browse_config_changes(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists config changes a page at a time.
    async fn find_config_changes(
        &self,
        request: tonic::Request<configuration::FindConfigChangesRequest>,
    ) -> Result<tonic::Response<configuration::FindConfigChangesResponse>, tonic::Status> {
        self.run_find_config_changes(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts config changes by status.
    async fn count_config_changes(
        &self,
        request: tonic::Request<configuration::CountConfigChangesRequest>,
    ) -> Result<tonic::Response<configuration::CountConfigChangesResponse>, tonic::Status> {
        self.run_count_config_changes(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several config changes at once.
    async fn resolve_config_changes(
        &self,
        request: tonic::Request<configuration::ResolveConfigChangesRequest>,
    ) -> Result<tonic::Response<configuration::ResolveConfigChangesResponse>, tonic::Status> {
        self.run_resolve_config_changes(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportConfigChangesStream =
        tokio_stream::wrappers::ReceiverStream<Result<configuration::ExportConfigChangesResponse, tonic::Status>>;
    /// Streams every matching config change.
    async fn export_config_changes(
        &self,
        request: tonic::Request<configuration::ExportConfigChangesRequest>,
    ) -> Result<tonic::Response<Self::ExportConfigChangesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_export_config_changes(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a config change.
    async fn summarize_config_changes(
        &self,
        request: tonic::Request<configuration::SummarizeConfigChangesRequest>,
    ) -> Result<tonic::Response<configuration::SummarizeConfigChangesResponse>, tonic::Status> {
        self.run_summarize_config_changes(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config change.
    async fn peek_config_change(
        &self,
        request: tonic::Request<configuration::PeekConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::PeekConfigChangeResponse>, tonic::Status> {
        self.run_peek_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
