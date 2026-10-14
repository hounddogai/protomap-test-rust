use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::configuration::v1::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_integration_service_client::ApiKeyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_client::ApiKeyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_notification_service_server::GatewayPolicyNotificationService;
use acme_proto::acme::edgegateway::v1::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_client::RouteBindingIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_notification_service_client::RouteBindingNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_query_service_client::RouteBindingQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_query_service_client::MetricSeriesQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_query_service_client::SearchIndexQueryServiceClient;
use acme_proto::acme::treasury::v1 as treasury;
use acme_proto::acme::treasury::v1::cash_position_calculation_service_client::CashPositionCalculationServiceClient;

/// Tells the people who follow a gateway policy what changed.
#[derive(Clone)]
pub struct GatewayPolicyNotificationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    notify_gateway_policy_watchers_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<ratelimiting::SubmitQuotaBucketRequest, ratelimiting::SubmitQuotaBucketResponse>,
    >,
    send_gateway_policy_test_notice_0_port: acme_platform::ports::Callback<
        edgegateway::TestRouteBindingConnectionRequest,
        edgegateway::TestRouteBindingConnectionResponse,
    >,
    send_gateway_policy_test_notice_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            metrics::SummarizeMetricSeriesesRequest,
            metrics::SummarizeMetricSeriesesResponse,
        >,
    >,
    stream_gateway_policy_notices_0_port:
        acme_platform::ports::Callback<treasury::EstimateCashPositionRequest, treasury::EstimateCashPositionResponse>,
    unsubscribe_gateway_policy_updates_0_port: acme_platform::ports::Callback<
        edgegateway::StartApiKeyWorkflowRequest,
        edgegateway::StartApiKeyWorkflowResponse,
    >,
    unsubscribe_gateway_policy_updates_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            edgegateway::AcknowledgeRouteBindingEventRequest,
            edgegateway::AcknowledgeRouteBindingEventResponse,
        >,
    >,
    unsubscribe_gateway_policy_updates_2_port: acme_platform::ports::Callback<
        configuration::RejectConfigChangeRequest,
        configuration::RejectConfigChangeResponse,
    >,
}
impl GatewayPolicyNotificationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            edgegateway::StartApiKeyWorkflowRequest,
            edgegateway::StartApiKeyWorkflowResponse,
        > = {
            let client = ApiKeyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::StartApiKeyWorkflowRequest| -> acme_platform::ports::RpcFuture<edgegateway::StartApiKeyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.start_api_key_workflow(request).await?.into_inner())
})
})
        };
        let unsubscribe_gateway_policy_updates_0_port = callback;
        let callback: acme_platform::ports::Callback<
            edgegateway::AcknowledgeRouteBindingEventRequest,
            edgegateway::AcknowledgeRouteBindingEventResponse,
        > = {
            let client = RouteBindingIntegrationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::AcknowledgeRouteBindingEventRequest| -> acme_platform::ports::RpcFuture<edgegateway::AcknowledgeRouteBindingEventResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.acknowledge_route_binding_event(request).await?.into_inner())
})
})
        };
        let unsubscribe_gateway_policy_updates_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::RejectConfigChangeRequest,
            configuration::RejectConfigChangeResponse,
        > = {
            let client = ConfigChangeApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::RejectConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::RejectConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.reject_config_change(request).await?.into_inner())
})
})
        };
        let unsubscribe_gateway_policy_updates_2_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::SubmitQuotaBucketRequest,
            ratelimiting::SubmitQuotaBucketResponse,
        > = {
            let client = QuotaBucketApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::SubmitQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::SubmitQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.submit_quota_bucket(request).await?.into_inner())
})
})
        };
        let notify_gateway_policy_watchers_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            edgegateway::TestRouteBindingConnectionRequest,
            edgegateway::TestRouteBindingConnectionResponse,
        > = {
            let client = RouteBindingIntegrationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::TestRouteBindingConnectionRequest| -> acme_platform::ports::RpcFuture<edgegateway::TestRouteBindingConnectionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.test_route_binding_connection(request).await?.into_inner())
})
})
        };
        let send_gateway_policy_test_notice_0_port = callback;
        let callback: acme_platform::ports::Callback<
            metrics::SummarizeMetricSeriesesRequest,
            metrics::SummarizeMetricSeriesesResponse,
        > = {
            let client = MetricSeriesQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: metrics::SummarizeMetricSeriesesRequest| -> acme_platform::ports::RpcFuture<metrics::SummarizeMetricSeriesesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.summarize_metric_serieses(request).await?.into_inner())
})
})
        };
        let send_gateway_policy_test_notice_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            treasury::EstimateCashPositionRequest,
            treasury::EstimateCashPositionResponse,
        > = {
            let client = CashPositionCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: treasury::EstimateCashPositionRequest| -> acme_platform::ports::RpcFuture<treasury::EstimateCashPositionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_cash_position(request).await?.into_inner())
})
})
        };
        let stream_gateway_policy_notices_0_port = callback;
        Self {
            channel,
            repository,
            notify_gateway_policy_watchers_0_port,
            send_gateway_policy_test_notice_0_port,
            send_gateway_policy_test_notice_1_port,
            stream_gateway_policy_notices_0_port,
            unsubscribe_gateway_policy_updates_0_port,
            unsubscribe_gateway_policy_updates_1_port,
            unsubscribe_gateway_policy_updates_2_port,
        }
    }
    /// Run SubscribeGatewayPolicyUpdates after validating the tenant and durably recording the command.
    async fn run_subscribe_gateway_policy_updates(
        &self,
        req: edgegateway::SubscribeGatewayPolicyUpdatesRequest,
    ) -> Result<edgegateway::SubscribeGatewayPolicyUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SubscribeGatewayPolicyUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubscribeGatewayPolicyUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::ScheduleGatewayPolicyReportRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            gateway_policy_id: req.gateway_policy_id.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ScheduleGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::PauseGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            details: Some(edgegateway::pause_gateway_policy_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::PauseGatewayPolicyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pause_gateway_policy_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = edgegateway::GenerateGatewayPolicyReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::generate_gateway_policy_report_request::Details {
                period: reply_0.period.clone(),
                ..Default::default()
            }),
            gateway_policy_id: req.gateway_policy_id.clone(),
            reason: (match reply_1.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyReportingServiceClient::new(channel);
            let input: tonic::Request<edgegateway::GenerateGatewayPolicyReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_gateway_policy_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = edgegateway::DisconnectApiKeySourceRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::disconnect_api_key_source_request::Details {
                message_id: reply_0.result.clone().unwrap_or_default().message.clone(),
                ..Default::default()
            }),
            reason: (match reply_1.gateway_policy.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::gateway_policy::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ApiKeyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::DisconnectApiKeySourceRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.disconnect_api_key_source(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = configuration::BatchCreateConfigSetsRequest {
            context: req.context.clone(),
            chunk: Some(configuration::batch_create_config_sets_request::Chunk::Record(
                configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        children: vec![configuration::ConfigSet {
                            audit: Some(common::AuditInfo {
                                created_at: reply_1
                                    .gateway_policy
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .created_at
                                    .clone(),
                                updated_by: reply_3
                                    .api_key
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
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::BatchCreateConfigSetsRequest> =
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
            let response = client.batch_create_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = edgegateway::RelayApiKeyMessagesRequest {
            frame: Some(edgegateway::relay_api_key_messages_request::Frame::Change(
                edgegateway::ApiKey {
                    funding: Some(edgegateway::api_key::Funding::Card(edgegateway::ApiKeyCardFunding {
                        network: (match reply_3.api_key.clone().unwrap_or_default().funding.clone() {
                            Some(edgegateway::api_key::Funding::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .network
                        .clone(),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_5 = async move {
            let mut client = ApiKeyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::RelayApiKeyMessagesRequest> =
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
            let response = client.relay_api_key_messages(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::RelayApiKeyMessagesResponse::default();
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
        let mut response = edgegateway::SubscribeGatewayPolicyUpdatesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.gateway_policy.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: String = reply_1
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
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
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run UnsubscribeGatewayPolicyUpdates after validating the tenant and durably recording the command.
    async fn run_unsubscribe_gateway_policy_updates(
        &self,
        req: edgegateway::UnsubscribeGatewayPolicyUpdatesRequest,
    ) -> Result<edgegateway::UnsubscribeGatewayPolicyUpdatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("UnsubscribeGatewayPolicyUpdates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UnsubscribeGatewayPolicyUpdates", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().preferred_channel.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::StartApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                contact: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = (self.unsubscribe_gateway_policy_updates_0_port)(outgoing_0).await?;
        let outgoing_1 = edgegateway::AcknowledgeRouteBindingEventRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.unsubscribe_gateway_policy_updates_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = configuration::RejectConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::reject_config_change_request::Details {
                approver: Some(identity::UserRef {
                    email: reply_1
                        .route_binding
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
        let port = acme_platform::ports::opaque(self.unsubscribe_gateway_policy_updates_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let mut response = edgegateway::UnsubscribeGatewayPolicyUpdatesResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = reply_2
            .config_change
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
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
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run NotifyGatewayPolicyWatchers after validating the tenant and durably recording the command.
    async fn run_notify_gateway_policy_watchers(
        &self,
        req: edgegateway::NotifyGatewayPolicyWatchersRequest,
    ) -> Result<edgegateway::NotifyGatewayPolicyWatchersResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NotifyGatewayPolicyWatchers", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NotifyGatewayPolicyWatchers", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().preferred_channel.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::SubmitQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.notify_gateway_policy_watchers_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = edgegateway::AdvanceApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                funding: Some(edgegateway::api_key::Funding::Bank(edgegateway::ApiKeyBankFunding {
                    bic: (match reply_0.quota_bucket.clone().unwrap_or_default().funding.clone() {
                        Some(ratelimiting::quota_bucket::Funding::Bank(value)) => value,
                        _ => Default::default(),
                    })
                    .bic
                    .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AdvanceApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::NotifyGatewayPolicyWatchersResponse::default();
        let contribution: String = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
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
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ListGatewayPolicySubscriptions after validating the tenant and durably recording the command.
    async fn run_list_gateway_policy_subscriptions(
        &self,
        req: edgegateway::ListGatewayPolicySubscriptionsRequest,
    ) -> Result<edgegateway::ListGatewayPolicySubscriptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListGatewayPolicySubscriptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListGatewayPolicySubscriptions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::RetryGatewayPolicyWorkflowRequest {
            annotations: (req.filter.clone().unwrap_or_default().labels.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            context: req.context.clone(),
            details: Some(edgegateway::retry_gateway_policy_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
            gateway_policy: Some(edgegateway::GatewayPolicy {
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
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
        let reply_0 = async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::RetryGatewayPolicyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.retry_gateway_policy_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ScheduleEnvironmentExpiryRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .gateway_policy
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    user_id: reply_0
                        .gateway_policy
                        .clone()
                        .unwrap_or_default()
                        .owner
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
        let reply_1 = async move {
            let mut client = EnvironmentLifecycleServiceClient::new(channel);
            let input: tonic::Request<configuration::ScheduleEnvironmentExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_environment_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::ListGatewayPolicySubscriptionsResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.gateway_policies.is_empty() {
                response.gateway_policies.push(Default::default());
            }
            response.gateway_policies[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_0
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .gateway_policy_id
            .clone();
        if contribution != <String>::default() {
            if response.gateway_policies.is_empty() {
                response.gateway_policies.push(Default::default());
            }
            response.gateway_policies[0].gateway_policy_id = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run SendGatewayPolicyTestNotice after validating the tenant and durably recording the command.
    async fn run_send_gateway_policy_test_notice(
        &self,
        req: edgegateway::SendGatewayPolicyTestNoticeRequest,
    ) -> Result<edgegateway::SendGatewayPolicyTestNoticeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SendGatewayPolicyTestNotice", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SendGatewayPolicyTestNotice", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().recipient.clone().unwrap_or_default().phone.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::TestRouteBindingConnectionRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::test_route_binding_connection_request::Details {
                message_id: req.details.clone().unwrap_or_default().template_id.clone(),
                received_at: req.context.clone().unwrap_or_default().deadline.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 = (self.send_gateway_policy_test_notice_0_port)(outgoing_0).await?;
        let outgoing_1 = metrics::SummarizeMetricSeriesesRequest {
            context: req.context.clone(),
            reason: (match reply_0.route_binding.clone().unwrap_or_default().action.clone() {
                Some(edgegateway::route_binding::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let reply_1 =
            self.send_gateway_policy_test_notice_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = edgegateway::SendGatewayPolicyTestNoticeResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = reply_0
            .route_binding
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
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
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run GetGatewayPolicyPreferences after validating the tenant and durably recording the command.
    async fn run_get_gateway_policy_preferences(
        &self,
        req: edgegateway::GetGatewayPolicyPreferencesRequest,
    ) -> Result<edgegateway::GetGatewayPolicyPreferencesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetGatewayPolicyPreferences", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetGatewayPolicyPreferences", "running workflow");
        let outgoing_0 = search::BrowseSearchIndexesRequest {
            context: req.context.clone(),
            filter: Some(search::browse_search_indexes_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                owner_id: req.gateway_policy_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<search::BrowseSearchIndexesRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = SearchIndexQueryServiceClient::new(channel);
                let input: tonic::Request<search::BrowseSearchIndexesRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.browse_search_indexes(input).await?;
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
        let outgoing_1 = edgegateway::InspectGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            gateway_policy_id: req.gateway_policy_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = GatewayPolicyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::InspectGatewayPolicyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.inspect_gateway_policy_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = edgegateway::GetGatewayPolicyPreferencesResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.gateway_policy.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.gateway_policy.get_or_insert_with(Default::default).description = contribution;
        }
        let contribution: Vec<String> = reply_1.gateway_policy.clone().unwrap_or_default().scopes.clone();
        if contribution != <Vec<String>>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .scopes
                .extend(contribution);
        }
        let contribution: String = req.gateway_policy_id.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        Ok(response)
    }
    /// Run StreamGatewayPolicyNotices after validating the tenant and durably recording the command.
    async fn run_stream_gateway_policy_notices(
        &self,
        req: edgegateway::StreamGatewayPolicyNoticesRequest,
    ) -> Result<edgegateway::StreamGatewayPolicyNoticesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamGatewayPolicyNotices", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamGatewayPolicyNotices", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = treasury::EstimateCashPositionRequest {
            cash_position: Some(treasury::CashPosition {
                contact: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    preferred_channel: req.subscriber.clone().unwrap_or_default().preferred_channel.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.stream_gateway_policy_notices_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = configuration::ResumeConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
                        .cash_position
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_by: reply_0
                                .cash_position
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
                    }],
                    manager: Some(identity::UserRef {
                        display_name: reply_0
                            .cash_position
                            .clone()
                            .unwrap_or_default()
                            .holder
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        email: reply_0
                            .cash_position
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<configuration::ResumeConfigSetWorkflowRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::ResumeConfigSetWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_config_set_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = edgegateway::StreamRouteBindingNoticesRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: reply_1
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                phone: reply_0
                    .cash_position
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: reply_0
                    .cash_position
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_2 = async move {
            let mut client = RouteBindingNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::StreamRouteBindingNoticesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_route_binding_notices(input).await?;
            let mut frames = response.into_inner();
            let mut last = edgegateway::StreamRouteBindingNoticesResponse::default();
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
        let outgoing_3 = edgegateway::BrowseRouteBindingsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::browse_route_bindings_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.edgegateway.v1.BrowseRouteBindingsRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = RouteBindingQueryServiceClient::new(channel);
            let input: tonic::Request<edgegateway::BrowseRouteBindingsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_route_bindings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::StreamGatewayPolicyNoticesResponse::default();
        let contribution: String = reply_0
            .cash_position
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().email.clone();
        if contribution != <String>::default() {
            response
                .gateway_policy
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl GatewayPolicyNotificationService for GatewayPolicyNotificationHandler {
    /// Subscribes a gateway policy.
    async fn subscribe_gateway_policy_updates(
        &self,
        request: tonic::Request<edgegateway::SubscribeGatewayPolicyUpdatesRequest>,
    ) -> Result<tonic::Response<edgegateway::SubscribeGatewayPolicyUpdatesResponse>, tonic::Status> {
        self.run_subscribe_gateway_policy_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Unsubscribes a gateway policy.
    async fn unsubscribe_gateway_policy_updates(
        &self,
        request: tonic::Request<edgegateway::UnsubscribeGatewayPolicyUpdatesRequest>,
    ) -> Result<tonic::Response<edgegateway::UnsubscribeGatewayPolicyUpdatesResponse>, tonic::Status> {
        self.run_unsubscribe_gateway_policy_updates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Notifys a gateway policy.
    async fn notify_gateway_policy_watchers(
        &self,
        request: tonic::Request<edgegateway::NotifyGatewayPolicyWatchersRequest>,
    ) -> Result<tonic::Response<edgegateway::NotifyGatewayPolicyWatchersResponse>, tonic::Status> {
        self.run_notify_gateway_policy_watchers(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists gateway policies a page at a time.
    async fn list_gateway_policy_subscriptions(
        &self,
        request: tonic::Request<edgegateway::ListGatewayPolicySubscriptionsRequest>,
    ) -> Result<tonic::Response<edgegateway::ListGatewayPolicySubscriptionsResponse>, tonic::Status> {
        self.run_list_gateway_policy_subscriptions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Sends a gateway policy.
    async fn send_gateway_policy_test_notice(
        &self,
        request: tonic::Request<edgegateway::SendGatewayPolicyTestNoticeRequest>,
    ) -> Result<tonic::Response<edgegateway::SendGatewayPolicyTestNoticeResponse>, tonic::Status> {
        self.run_send_gateway_policy_test_notice(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a gateway policy.
    async fn get_gateway_policy_preferences(
        &self,
        request: tonic::Request<edgegateway::GetGatewayPolicyPreferencesRequest>,
    ) -> Result<tonic::Response<edgegateway::GetGatewayPolicyPreferencesResponse>, tonic::Status> {
        self.run_get_gateway_policy_preferences(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamGatewayPolicyNoticesStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::StreamGatewayPolicyNoticesResponse, tonic::Status>>;
    /// Streams the changes of gateway policies as they happen.
    async fn stream_gateway_policy_notices(
        &self,
        request: tonic::Request<edgegateway::StreamGatewayPolicyNoticesRequest>,
    ) -> Result<tonic::Response<Self::StreamGatewayPolicyNoticesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(
                async move { handler.run_stream_gateway_policy_notices(request).await },
            );
        Ok(tonic::Response::new(stream))
    }
}
