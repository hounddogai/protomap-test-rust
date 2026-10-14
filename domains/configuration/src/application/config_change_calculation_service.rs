use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_history_service_client::RoleBindingHistoryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_server::ConfigChangeCalculationService;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_validation_service_client::DocumentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_query_service_client::NotificationQueryServiceClient;
use acme_proto::acme::shows::v1 as shows;
use acme_proto::acme::shows::v1::show_query_service_client::ShowQueryServiceClient;
use acme_proto::acme::tours::v1 as tours;
use acme_proto::acme::tours::v1::tour_service_client::TourServiceClient;

/// Computes the figures of a config change, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct ConfigChangeCalculationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    compare_config_change_options_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::SimulateConfigSetRequest,
            configuration::SimulateConfigSetResponse,
        >,
    >,
    estimate_config_change_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::RecalculateConfigSetsRequest,
                configuration::RecalculateConfigSetsResponse,
            >,
    >,
    estimate_config_change_1_port: acme_platform::ports::Callback<
        configuration::CompareConfigSetOptionsRequest,
        configuration::CompareConfigSetOptionsResponse,
    >,
    explain_config_change_calculation_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<shows::SuggestShowsRequest, shows::SuggestShowsResponse>,
    >,
    get_config_change_rates_0_port: acme_platform::ports::Callback<
        configuration::EstimateConfigSetRequest,
        configuration::EstimateConfigSetResponse,
    >,
    simulate_config_change_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::CalculateConfigSetRequest,
            configuration::CalculateConfigSetResponse,
        >,
    >,
}
impl ConfigChangeCalculationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::RecalculateConfigSetsRequest,
            configuration::RecalculateConfigSetsResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::RecalculateConfigSetsRequest| -> acme_platform::ports::RpcFuture<configuration::RecalculateConfigSetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_config_sets(request).await?.into_inner())
})
})
        };
        let estimate_config_change_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let estimate_config_change_1_port = callback;
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
        let compare_config_change_options_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let simulate_config_change_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<shows::SuggestShowsRequest, shows::SuggestShowsResponse> = {
            let client = ShowQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: shows::SuggestShowsRequest| -> acme_platform::ports::RpcFuture<shows::SuggestShowsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.suggest_shows(request).await?.into_inner())
})
})
        };
        let explain_config_change_calculation_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
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
        let get_config_change_rates_0_port = callback;
        Self {
            channel,
            repository,
            compare_config_change_options_2_port,
            estimate_config_change_0_port,
            estimate_config_change_1_port,
            explain_config_change_calculation_0_port,
            get_config_change_rates_0_port,
            simulate_config_change_1_port,
        }
    }
    /// Run CalculateConfigChange after validating the tenant and durably recording the command.
    async fn run_calculate_config_change(
        &self,
        req: configuration::CalculateConfigChangeRequest,
    ) -> Result<configuration::CalculateConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            country_code: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            lines: (req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
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
        let reply_0 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::SimulateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = tours::BatchCreateToursRequest {
            context: req.context.clone(),
            chunk: Some(tours::batch_create_tours_request::Chunk::Record(tours::Tour {
                participants: vec![tours::TourParticipant {
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
                    ..Default::default()
                }],
                ..Default::default()
            })),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = TourServiceClient::new(channel);
            let input: tonic::Request<tours::BatchCreateToursRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_tours(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
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
                            .details
                            .clone()
                            .unwrap_or_default()
                            .destination
                            .clone()
                            .unwrap_or_default()
                            .lines
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
        let reply_2 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
        let mut response = configuration::CalculateConfigChangeResponse::default();
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
        let contribution: String = req.config_change.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run EstimateConfigChange after validating the tenant and durably recording the command.
    async fn run_estimate_config_change(
        &self,
        req: configuration::EstimateConfigChangeRequest,
    ) -> Result<configuration::EstimateConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::RecalculateConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            postal_code: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
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
        let reply_0 = acme_platform::ports::execute(self.estimate_config_change_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::CompareConfigSetOptionsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            lines: reply_0
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
        let reply_1 = (self.estimate_config_change_1_port)(outgoing_1).await?;
        let mut response = configuration::EstimateConfigChangeResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_set.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
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
        let contribution: String = reply_1.config_set.clone().unwrap_or_default().config_set_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .config_set_id = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.config_change.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.config_change.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run QuoteConfigChange after validating the tenant and durably recording the command.
    async fn run_quote_config_change(
        &self,
        req: configuration::QuoteConfigChangeRequest,
    ) -> Result<configuration::QuoteConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let command_0 = QuoteConfigChangeDispatch0::from(req.clone());
        let outgoing_0: configuration::SimulateConfigSetRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::SimulateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = shows::ResolveShowsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::codec::through_any(&outgoing_1, "acme.shows.v1.ResolveShowsRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ShowQueryServiceClient::new(channel);
            let input: tonic::Request<shows::ResolveShowsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_shows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_2 = req
            .config_change
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_2 = authorization::ListRoleBindingVersionsRequest {
                        context: req.context.clone(),
                        filter: Some(authorization::list_role_binding_versions_request::Filter {
                            created_after: item.observed_at.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = RoleBindingHistoryServiceClient::new(channel);
                        let input: tonic::Request<authorization::ListRoleBindingVersionsRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.list_role_binding_versions(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let mut response = configuration::QuoteConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::Money> = reply_0.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: std::collections::BTreeMap<String, String> =
            req.config_change.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CompareConfigChangeOptions after validating the tenant and durably recording the command.
    async fn run_compare_config_change_options(
        &self,
        req: configuration::CompareConfigChangeOptionsRequest,
    ) -> Result<configuration::CompareConfigChangeOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareConfigChangeOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareConfigChangeOptions", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = notifications::ExportNotificationsRequest {
            context: req.context.clone(),
            filter: Some(notifications::export_notifications_request::Filter {
                created_after: req.config_change.clone().unwrap_or_default().occurred_at.clone(),
                labels: req.config_change.clone().unwrap_or_default().attributes.clone(),
                owner_id: req
                    .config_change
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .config_set_id
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
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = NotificationQueryServiceClient::new(channel);
            let input: tonic::Request<notifications::ExportNotificationsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_notifications(input).await?;
            let mut frames = response.into_inner();
            let mut last = notifications::ExportNotificationsResponse::default();
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
        let outgoing_1 = authorization::RevertRoleBindingRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
                billing_address: Some(common::PostalAddress {
                    lines: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    locality: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                counterparty: Some(authorization::AccessPolicyRef {
                    display_name: req
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
                lines: vec![authorization::RoleBindingLine {
                    attributes: req.config_change.clone().unwrap_or_default().attributes.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RoleBindingHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::RevertRoleBindingRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.revert_role_binding(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        children: vec![configuration::ConfigSet {
                            config_set_id: req
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
                            display_name: reply_1
                                .role_binding
                                .clone()
                                .unwrap_or_default()
                                .counterparty
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
        let reply_2 =
            acme_platform::ports::execute(self.compare_config_change_options_2_port.as_ref(), outgoing_2).await?;
        let mut response = configuration::CompareConfigChangeOptionsResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: Option<common::Money> = reply_2.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: String = req.config_change.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateConfigChanges after validating the tenant and durably recording the command.
    async fn run_recalculate_config_changes(
        &self,
        req: configuration::RecalculateConfigChangesRequest,
    ) -> Result<configuration::RecalculateConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecalculateConfigChanges", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateConfigChanges", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::RecalculateConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            locality: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
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
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = documents::CheckDocumentEligibilityRequest {
            context: req.context.clone(),
            document: Some(documents::Document {
                billing_address: Some(common::PostalAddress {
                    lines: (reply_0
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
                    postal_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = DocumentValidationServiceClient::new(channel);
            let input: tonic::Request<documents::CheckDocumentEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_document_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::RecalculateConfigChangesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: String = reply_1
            .document
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .path
            .clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .path = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: String = req.config_change.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.config_change.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run SimulateConfigChange after validating the tenant and durably recording the command.
    async fn run_simulate_config_change(
        &self,
        req: configuration::SimulateConfigChangeRequest,
    ) -> Result<configuration::SimulateConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CompareConfigSetOptionsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            lines: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone(),
                            postal_code: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .postal_code
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
                    }],
                    location: Some(common::PostalAddress {
                        locality: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .destination
                            .clone()
                            .unwrap_or_default()
                            .locality
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
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CompareConfigSetOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_config_set_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::CalculateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            lines: reply_0
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone(),
                            postal_code: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .postal_code
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
                                .config_change
                                .clone()
                                .unwrap_or_default()
                                .subject
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.simulate_config_change_1_port.as_ref(), outgoing_1).await?;
        let mut response = configuration::SimulateConfigChangeResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: i64 = req.config_change.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.config_change.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run ExplainConfigChangeCalculation after validating the tenant and durably recording the command.
    async fn run_explain_config_change_calculation(
        &self,
        req: configuration::ExplainConfigChangeCalculationRequest,
    ) -> Result<configuration::ExplainConfigChangeCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainConfigChangeCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainConfigChangeCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = shows::SuggestShowsRequest {
            context: req.context.clone(),
            filter: Some(shows::suggest_shows_request::Filter {
                owner_id: req
                    .config_change
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .config_set_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.explain_config_change_calculation_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = configuration::ExplainConfigChangeCalculationResponse::default();
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
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.config_change_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .config_change_id = contribution;
        }
        let contribution: i64 = req.config_change.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.config_change.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run GetConfigChangeRates after validating the tenant and durably recording the command.
    async fn run_get_config_change_rates(
        &self,
        req: configuration::GetConfigChangeRatesRequest,
    ) -> Result<configuration::GetConfigChangeRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetConfigChangeRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetConfigChangeRates", "running workflow");
        let outgoing_0 = configuration::EstimateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        parent_id: req.config_change_id.clone(),
                        ..Default::default()
                    }],
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.get_config_change_rates_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = configuration::CompareConfigSetOptionsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        children: vec![configuration::ConfigSet {
                            attributes: (reply_0.config_set.clone().unwrap_or_default().attributes.clone())
                                .into_iter()
                                .filter(|(key, _)| !key.is_empty())
                                .map(|(key, value)| (key, value.trim().to_owned()))
                                .collect(),
                            ..Default::default()
                        }],
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
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        display_name: reply_0
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CompareConfigSetOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_config_set_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::GetConfigChangeRatesResponse::default();
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_set.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
/// The domain command prepared by QuoteConfigChange for SimulateConfigSet.
struct QuoteConfigChangeDispatch0 {
    context: Option<common::RequestContext>,
    config_set_children_children_location_country_code: String,
    config_set_children_children_location_lines: Vec<String>,
}
impl From<configuration::QuoteConfigChangeRequest> for QuoteConfigChangeDispatch0 {
    fn from(req: configuration::QuoteConfigChangeRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_children_location_country_code: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .country_code
                .clone(),
            config_set_children_children_location_lines: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .lines
                .clone(),
        }
    }
}
impl From<QuoteConfigChangeDispatch0> for configuration::SimulateConfigSetRequest {
    fn from(value: QuoteConfigChangeDispatch0) -> Self {
        configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            country_code: value.config_set_children_children_location_country_code,
                            lines: value.config_set_children_children_location_lines,
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
impl ConfigChangeCalculationService for ConfigChangeCalculationController {
    /// Calculates a config change.
    async fn calculate_config_change(
        &self,
        request: tonic::Request<configuration::CalculateConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::CalculateConfigChangeResponse>, tonic::Status> {
        self.run_calculate_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a config change.
    async fn estimate_config_change(
        &self,
        request: tonic::Request<configuration::EstimateConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::EstimateConfigChangeResponse>, tonic::Status> {
        self.run_estimate_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a config change.
    async fn quote_config_change(
        &self,
        request: tonic::Request<configuration::QuoteConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::QuoteConfigChangeResponse>, tonic::Status> {
        self.run_quote_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a config change.
    async fn compare_config_change_options(
        &self,
        request: tonic::Request<configuration::CompareConfigChangeOptionsRequest>,
    ) -> Result<tonic::Response<configuration::CompareConfigChangeOptionsResponse>, tonic::Status> {
        self.run_compare_config_change_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a config change.
    async fn recalculate_config_changes(
        &self,
        request: tonic::Request<configuration::RecalculateConfigChangesRequest>,
    ) -> Result<tonic::Response<configuration::RecalculateConfigChangesResponse>, tonic::Status> {
        self.run_recalculate_config_changes(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a config change.
    async fn simulate_config_change(
        &self,
        request: tonic::Request<configuration::SimulateConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::SimulateConfigChangeResponse>, tonic::Status> {
        self.run_simulate_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a config change.
    async fn explain_config_change_calculation(
        &self,
        request: tonic::Request<configuration::ExplainConfigChangeCalculationRequest>,
    ) -> Result<tonic::Response<configuration::ExplainConfigChangeCalculationResponse>, tonic::Status> {
        self.run_explain_config_change_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config change.
    async fn get_config_change_rates(
        &self,
        request: tonic::Request<configuration::GetConfigChangeRatesRequest>,
    ) -> Result<tonic::Response<configuration::GetConfigChangeRatesResponse>, tonic::Status> {
        self.run_get_config_change_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
