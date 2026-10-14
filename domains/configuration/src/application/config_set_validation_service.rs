use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::boxoffice::v1::box_office_sale_service_client::BoxOfficeSaleServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::concessions::v1::concession_stand_service_client::ConcessionStandServiceClient;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_server::ConfigSetValidationService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_validation_service_client::MetricSeriesValidationServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_query_service_client::NotificationQueryServiceClient;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::patients::v1::patient_query_service_client::PatientQueryServiceClient;

/// Checks config sets against the rules before anything acts on them.
#[derive(Clone)]
pub struct ConfigSetValidationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    explain_config_set_decision_1_port:
        std::sync::Arc<acme_platform::ports::Adapter<audit::CreateAuditEventRequest, audit::CreateAuditEventResponse>>,
    get_config_set_rules_0_port: acme_platform::ports::Callback<
        featureflags::ExplainFeatureFlagDecisionRequest,
        featureflags::ExplainFeatureFlagDecisionResponse,
    >,
    lint_config_sets_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::EstimateConfigSetRequest,
            configuration::EstimateConfigSetResponse,
        >,
    >,
    lint_config_sets_1_port: acme_platform::ports::Callback<
        configuration::CompareConfigSetOptionsRequest,
        configuration::CompareConfigSetOptionsResponse,
    >,
    score_config_set_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<patients::BrowsePatientsRequest, patients::BrowsePatientsResponse>,
    >,
    score_config_set_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::CompareConfigChangeOptionsRequest,
                configuration::CompareConfigChangeOptionsResponse,
            >,
    >,
    score_config_set_3_port:
        acme_platform::ports::Callback<boxoffice::ListBoxOfficeSalesRequest, boxoffice::ListBoxOfficeSalesResponse>,
    validate_config_set_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::CompareConfigChangeOptionsRequest,
            configuration::CompareConfigChangeOptionsResponse,
        >,
    >,
    validate_config_set_1_port: acme_platform::ports::Callback<
        configuration::SimulateConfigSetRequest,
        configuration::SimulateConfigSetResponse,
    >,
}
impl ConfigSetValidationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::CompareConfigChangeOptionsRequest,
            configuration::CompareConfigChangeOptionsResponse,
        > = {
            let client = ConfigChangeCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CompareConfigChangeOptionsRequest| -> acme_platform::ports::RpcFuture<configuration::CompareConfigChangeOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_config_change_options(request).await?.into_inner())
})
})
        };
        let validate_config_set_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
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
        let validate_config_set_1_port = callback;
        let callback: acme_platform::ports::Callback<audit::CreateAuditEventRequest, audit::CreateAuditEventResponse> = {
            let client = AuditEventServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: audit::CreateAuditEventRequest| -> acme_platform::ports::RpcFuture<audit::CreateAuditEventResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.create_audit_event(request).await?.into_inner())
})
})
        };
        let explain_config_set_decision_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            patients::BrowsePatientsRequest,
            patients::BrowsePatientsResponse,
        > = {
            let client = PatientQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: patients::BrowsePatientsRequest| -> acme_platform::ports::RpcFuture<patients::BrowsePatientsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_patients(request).await?.into_inner())
})
})
        };
        let score_config_set_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::CompareConfigChangeOptionsRequest,
            configuration::CompareConfigChangeOptionsResponse,
        > = {
            let client = ConfigChangeCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CompareConfigChangeOptionsRequest| -> acme_platform::ports::RpcFuture<configuration::CompareConfigChangeOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_config_change_options(request).await?.into_inner())
})
})
        };
        let score_config_set_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            boxoffice::ListBoxOfficeSalesRequest,
            boxoffice::ListBoxOfficeSalesResponse,
        > = {
            let client = BoxOfficeSaleServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: boxoffice::ListBoxOfficeSalesRequest| -> acme_platform::ports::RpcFuture<boxoffice::ListBoxOfficeSalesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_box_office_sales(request).await?.into_inner())
})
})
        };
        let score_config_set_3_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::EstimateConfigSetRequest,
            configuration::EstimateConfigSetResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::EstimateConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::EstimateConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_config_set(request).await?.into_inner())
})
})
        };
        let lint_config_sets_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::CompareConfigSetOptionsRequest,
            configuration::CompareConfigSetOptionsResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CompareConfigSetOptionsRequest| -> acme_platform::ports::RpcFuture<configuration::CompareConfigSetOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_config_set_options(request).await?.into_inner())
})
})
        };
        let lint_config_sets_1_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::ExplainFeatureFlagDecisionRequest,
            featureflags::ExplainFeatureFlagDecisionResponse,
        > = {
            let client = FeatureFlagValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ExplainFeatureFlagDecisionRequest| -> acme_platform::ports::RpcFuture<featureflags::ExplainFeatureFlagDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_feature_flag_decision(request).await?.into_inner())
})
})
        };
        let get_config_set_rules_0_port = callback;
        Self {
            channel,
            repository,
            explain_config_set_decision_1_port,
            get_config_set_rules_0_port,
            lint_config_sets_0_port,
            lint_config_sets_1_port,
            score_config_set_1_port,
            score_config_set_2_port,
            score_config_set_3_port,
            validate_config_set_0_port,
            validate_config_set_1_port,
        }
    }
    /// Run ValidateConfigSet after validating the tenant and durably recording the command.
    async fn run_validate_config_set(
        &self,
        req: configuration::ValidateConfigSetRequest,
    ) -> Result<configuration::ValidateConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::CompareConfigChangeOptionsRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::compare_config_change_options_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    locality: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    postal_code: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.validate_config_set_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            locality: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .locality
                                .clone(),
                            postal_code: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .postal_code
                                .clone(),
                            recipient: req
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
                        manager: Some(identity::UserRef {
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .contact
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
        let port = acme_platform::ports::opaque(self.validate_config_set_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let outgoing_2 = configuration::GetConfigSetRatesRequest {
            config_set_id: req.config_set.clone().unwrap_or_default().config_set_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<configuration::GetConfigSetRatesRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::GetConfigSetRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_config_set_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_3 = ValidateConfigSetDispatch3::from(req.clone());
        command_3.config_set_children_children_audit_etag = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        let outgoing_3: configuration::QuoteConfigSetRequest = command_3.into();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::QuoteConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = concessions::BatchCreateConcessionStandsRequest {
            context: req.context.clone(),
            chunk: Some(concessions::batch_create_concession_stands_request::Chunk::Record(
                concessions::ConcessionStand {
                    owner: Some(identity::UserRef {
                        email: reply_1
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = ConcessionStandServiceClient::new(channel);
            let input: tonic::Request<concessions::BatchCreateConcessionStandsRequest> =
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
            let response = client.batch_create_concession_stands(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    participants: vec![featureflags::FlagRolloutParticipant {
                        contact: Some(common::ContactInfo {
                            phone: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .contact
                                .clone()
                                .unwrap_or_default()
                                .phone
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
        let reply_5 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
        let mut response = configuration::ValidateConfigSetResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_change.clone().unwrap_or_default().attributes.clone();
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
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: i32 = reply_1.config_set.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.config_set.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
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
        let contribution: String = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .recipient
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
                .location
                .get_or_insert_with(Default::default)
                .recipient = contribution;
        }
        let contribution: Option<common::PostalAddress> =
            reply_3.config_set.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.config_set.get_or_insert_with(Default::default).location = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: String = req.config_set.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).parent_id = contribution;
        }
        Ok(response)
    }
    /// Run PreviewConfigSet after validating the tenant and durably recording the command.
    async fn run_preview_config_set(
        &self,
        req: configuration::PreviewConfigSetRequest,
    ) -> Result<configuration::PreviewConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = notifications::SummarizeNotificationsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(notifications::summarize_notifications_request::Details {
                filters: req.config_set.clone().unwrap_or_default().attributes.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<notifications::SummarizeNotificationsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = NotificationQueryServiceClient::new(channel);
                let input: tonic::Request<notifications::SummarizeNotificationsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.summarize_notifications(input).await?;
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
        let outgoing_1 = configuration::CompareConfigSetOptionsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_by: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            updated_at: req
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
                            locality: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .locality
                                .clone(),
                            postal_code: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .postal_code
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
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CompareConfigSetOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_config_set_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = configuration::RecalculateConfigChangesRequest {
            context: req.context.clone(),
            details: Some(configuration::recalculate_config_changes_request::Details {
                destination: Some(common::PostalAddress {
                    lines: (reply_1
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone())
                    .into_iter()
                    .filter_map(|value| {
                        let value = value.trim();
                        if value.is_empty() { None } else { Some(value.to_owned()) }
                    })
                    .collect(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::PreviewConfigSetResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
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
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.config_set.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CheckConfigSetEligibility after validating the tenant and durably recording the command.
    async fn run_check_config_set_eligibility(
        &self,
        req: configuration::CheckConfigSetEligibilityRequest,
    ) -> Result<configuration::CheckConfigSetEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckConfigSetEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckConfigSetEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = metrics::ScoreMetricSeriesRequest {
            context: req.context.clone(),
            details: Some(metrics::score_metric_series_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .config_set
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
            metric_series: Some(metrics::MetricSeries {
                owner: Some(identity::UserRef {
                    user_id: req
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
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.metrics.v1.ScoreMetricSeriesRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = MetricSeriesValidationServiceClient::new(channel);
            let input: tonic::Request<metrics::ScoreMetricSeriesRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_metric_series(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::EstimateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: reply_0
                                .metric_series
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
                            updated_by: reply_0
                                .metric_series
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_by
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            postal_code: (match reply_0.metric_series.clone().unwrap_or_default().locator.clone() {
                                Some(metrics::metric_series::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .postal_code
                            .clone(),
                            region: (match reply_0.metric_series.clone().unwrap_or_default().locator.clone() {
                                Some(metrics::metric_series::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .region
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::EstimateConfigSetRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigSetCalculationServiceClient::new(channel);
                let input: tonic::Request<configuration::EstimateConfigSetRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.estimate_config_set(input).await?;
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
        let outgoing_2 = configuration::GetConfigSetRatesRequest {
            config_set_id: req.config_set.clone().unwrap_or_default().config_set_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::GetConfigSetRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_config_set_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_3 = configuration::ExplainConfigSetCalculationRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            locality: reply_2
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .locality
                                .clone(),
                            recipient: (match reply_0.metric_series.clone().unwrap_or_default().locator.clone() {
                                Some(metrics::metric_series::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
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
        let reply_3 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainConfigSetCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_config_set_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::CheckConfigSetEligibilityResponse::default();
        let contribution: String = (match reply_0.metric_series.clone().unwrap_or_default().locator.clone() {
            Some(metrics::metric_series::Locator::Address(value)) => value,
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
            response.config_set.get_or_insert_with(Default::default).children[0]
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
        }
        let contribution: String = (match reply_0.metric_series.clone().unwrap_or_default().locator.clone() {
            Some(metrics::metric_series::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .region
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
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .locality
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
        }
        let contribution: Vec<String> = reply_1.config_set.clone().unwrap_or_default().path.clone();
        if contribution != <Vec<String>>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .path
                .extend(contribution);
        }
        let contribution: String = reply_2
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
                .config_set
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_2
            .config_set
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
            if response.config_set.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.config_set.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.config_set.get_or_insert_with(Default::default).children[0].children[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: String = req.config_set.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).parent_id = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeConfigSet after validating the tenant and durably recording the command.
    async fn run_normalize_config_set(
        &self,
        req: configuration::NormalizeConfigSetRequest,
    ) -> Result<configuration::NormalizeConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::EstimateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            locality: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .locality
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
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.EstimateConfigSetRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::EstimateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::NormalizeConfigSetResponse::default();
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .user_id
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
                .user_id = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: String = req.config_set.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run ExplainConfigSetDecision after validating the tenant and durably recording the command.
    async fn run_explain_config_set_decision(
        &self,
        req: configuration::ExplainConfigSetDecisionRequest,
    ) -> Result<configuration::ExplainConfigSetDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainConfigSetDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainConfigSetDecision", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CalculateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            etag: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            country_code: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            user_id: req
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
                    }],
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
            let input: tonic::Request<configuration::CalculateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = audit::CreateAuditEventRequest {
            audit_event: Some(audit::AuditEvent {
                subject: Some(audit::AuditTrailRef {
                    display_name: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
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
        let reply_1 =
            acme_platform::ports::execute(self.explain_config_set_decision_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::RecalculateConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_by: req
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
                        location: Some(common::PostalAddress {
                            lines: (req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone())
                            .into_iter()
                            .filter_map(|value| {
                                let value = value.trim();
                                if value.is_empty() { None } else { Some(value.to_owned()) }
                            })
                            .collect(),
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
        let reply_2 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ExplainConfigSetDecisionResponse::default();
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
        let contribution: String = reply_1
            .audit_event
            .clone()
            .unwrap_or_default()
            .subject
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
            response.config_set.get_or_insert_with(Default::default).children[0]
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_2
            .config_set
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
        let contribution: String = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
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
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: i32 = req.config_set.clone().unwrap_or_default().depth.clone();
        if contribution != <i32>::default() {
            response.config_set.get_or_insert_with(Default::default).depth = contribution;
        }
        Ok(response)
    }
    /// Run ScoreConfigSet after validating the tenant and durably recording the command.
    async fn run_score_config_set(
        &self,
        req: configuration::ScoreConfigSetRequest,
    ) -> Result<configuration::ScoreConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ValidateFeatureFlagRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    etag: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_by: req
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
                    display_name: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ValidateFeatureFlagRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_feature_flag(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = patients::BrowsePatientsRequest {
            context: req.context.clone(),
            filter: Some(patients::browse_patients_request::Filter {
                created_after: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.config_set.clone().unwrap_or_default().attributes.clone(),
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.score_config_set_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::CompareConfigChangeOptionsRequest {
            context: req.context.clone(),
            details: Some(configuration::compare_config_change_options_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    locality: req
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.score_config_set_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = boxoffice::ListBoxOfficeSalesRequest {
            context: req.context.clone(),
            filter: Some(boxoffice::list_box_office_sales_request::Filter {
                created_after: reply_0
                    .feature_flag
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_3 = (self.score_config_set_3_port)(outgoing_3).await?;
        let mut response = configuration::ScoreConfigSetResponse::default();
        let contribution: String = reply_0
            .feature_flag
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
            response.config_set.get_or_insert_with(Default::default).children[0]
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_3.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.config_set.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.config_set.get_or_insert_with(Default::default).manager = contribution;
        }
        Ok(response)
    }
    /// Run LintConfigSets after validating the tenant and durably recording the command.
    async fn run_lint_config_sets(
        &self,
        req: configuration::LintConfigSetsRequest,
    ) -> Result<configuration::LintConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::EstimateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_by: req
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
                        children: vec![configuration::ConfigSet {
                            config_set_id: req.config_set.clone().unwrap_or_default().config_set_id.clone(),
                            ..Default::default()
                        }],
                        location: Some(common::PostalAddress {
                            lines: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone(),
                            region: req
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.lint_config_sets_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = configuration::CompareConfigSetOptionsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_at: req
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
                            country_code: reply_0
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            lines: req
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone(),
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.lint_config_sets_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let outgoing_2 = configuration::RecalculateConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_at: req
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
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<configuration::RecalculateConfigSetsRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_3 = LintConfigSetsDispatch3::from(req.clone());
        command_3.config_set_children_children_location_country_code = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        command_3.config_set_children_children_audit_updated_by = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        let outgoing_3: configuration::CalculateConfigSetRequest = command_3.into();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CalculateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::LintConfigSetsResponse::default();
        let contribution: String = reply_0.config_set.clone().unwrap_or_default().config_set_id.clone();
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
        let contribution: String = reply_1.config_set.clone().unwrap_or_default().config_set_id.clone();
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
        let contribution: Vec<String> = reply_1.config_set.clone().unwrap_or_default().path.clone();
        if contribution != <Vec<String>>::default() {
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
                .path
                .extend(contribution);
        }
        let contribution: String = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
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
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: i32 = reply_3.config_set.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.config_set.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3
            .config_set
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .recipient
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
                .recipient = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.config_set.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run GetConfigSetRules after validating the tenant and durably recording the command.
    async fn run_get_config_set_rules(
        &self,
        req: configuration::GetConfigSetRulesRequest,
    ) -> Result<configuration::GetConfigSetRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetConfigSetRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetConfigSetRules", "running workflow");
        let outgoing_0 = featureflags::ExplainFeatureFlagDecisionRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_feature_flag_decision_request::Details {
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
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
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
        let reply_0 = (self.get_config_set_rules_0_port)(outgoing_0).await?;
        let mut response = configuration::GetConfigSetRulesResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
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
                .updated_at = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
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
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ValidateConfigSet for QuoteConfigSet.
struct ValidateConfigSetDispatch3 {
    context: Option<common::RequestContext>,
    config_set_children_children_audit_created_at: Option<prost_types::Timestamp>,
    config_set_children_children_audit_etag: String,
}
impl From<configuration::ValidateConfigSetRequest> for ValidateConfigSetDispatch3 {
    fn from(req: configuration::ValidateConfigSetRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_children_audit_created_at: req
                .config_set
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
            config_set_children_children_audit_etag: Default::default(),
        }
    }
}
impl From<ValidateConfigSetDispatch3> for configuration::QuoteConfigSetRequest {
    fn from(value: ValidateConfigSetDispatch3) -> Self {
        configuration::QuoteConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: value.config_set_children_children_audit_created_at,
                            etag: value.config_set_children_children_audit_etag,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
/// The domain command prepared by LintConfigSets for CalculateConfigSet.
struct LintConfigSetsDispatch3 {
    context: Option<common::RequestContext>,
    config_set_children_children_location_country_code: String,
    config_set_children_children_audit_updated_by: String,
}
impl From<configuration::LintConfigSetsRequest> for LintConfigSetsDispatch3 {
    fn from(req: configuration::LintConfigSetsRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_children_location_country_code: Default::default(),
            config_set_children_children_audit_updated_by: Default::default(),
        }
    }
}
impl From<LintConfigSetsDispatch3> for configuration::CalculateConfigSetRequest {
    fn from(value: LintConfigSetsDispatch3) -> Self {
        configuration::CalculateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_by: value.config_set_children_children_audit_updated_by,
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            country_code: value.config_set_children_children_location_country_code,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ConfigSetValidationService for ConfigSetValidationController {
    /// Validates a config set.
    async fn validate_config_set(
        &self,
        request: tonic::Request<configuration::ValidateConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::ValidateConfigSetResponse>, tonic::Status> {
        self.run_validate_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a config set.
    async fn preview_config_set(
        &self,
        request: tonic::Request<configuration::PreviewConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::PreviewConfigSetResponse>, tonic::Status> {
        self.run_preview_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a config set.
    async fn check_config_set_eligibility(
        &self,
        request: tonic::Request<configuration::CheckConfigSetEligibilityRequest>,
    ) -> Result<tonic::Response<configuration::CheckConfigSetEligibilityResponse>, tonic::Status> {
        self.run_check_config_set_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a config set.
    async fn normalize_config_set(
        &self,
        request: tonic::Request<configuration::NormalizeConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::NormalizeConfigSetResponse>, tonic::Status> {
        self.run_normalize_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a config set.
    async fn explain_config_set_decision(
        &self,
        request: tonic::Request<configuration::ExplainConfigSetDecisionRequest>,
    ) -> Result<tonic::Response<configuration::ExplainConfigSetDecisionResponse>, tonic::Status> {
        self.run_explain_config_set_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a config set.
    async fn score_config_set(
        &self,
        request: tonic::Request<configuration::ScoreConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::ScoreConfigSetResponse>, tonic::Status> {
        self.run_score_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a config set.
    async fn lint_config_sets(
        &self,
        request: tonic::Request<configuration::LintConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::LintConfigSetsResponse>, tonic::Status> {
        self.run_lint_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config set.
    async fn get_config_set_rules(
        &self,
        request: tonic::Request<configuration::GetConfigSetRulesRequest>,
    ) -> Result<tonic::Response<configuration::GetConfigSetRulesResponse>, tonic::Status> {
        self.run_get_config_set_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
