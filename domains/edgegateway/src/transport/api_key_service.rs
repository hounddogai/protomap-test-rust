use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_integration_service_client::ApiKeyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_notification_service_client::ApiKeyNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_service_server::ApiKeyService;
use acme_proto::acme::edgegateway::v1::api_key_workflow_service_client::ApiKeyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_integration_service_client::GatewayPolicyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_query_service_client::GatewayPolicyQueryServiceClient;
use acme_proto::acme::edgegateway::v1::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient;
use acme_proto::acme::edgegateway::v1::route_binding_integration_service_client::RouteBindingIntegrationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_client::FeatureFlagHistoryServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::patientapp::v1 as patientapp;
use acme_proto::acme::patientapp::v1::patient_home_service_client::PatientHomeServiceClient;
use acme_proto::acme::personalization::v1 as personalization;
use acme_proto::acme::personalization::v1::profile_calculation_service_client::ProfileCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::throttle_workflow_service_client::ThrottleWorkflowServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_version_approval_service_client::SecretVersionApprovalServiceClient;
use acme_proto::acme::warehousing::v1 as warehousing;
use acme_proto::acme::warehousing::v1::warehouse_service_client::WarehouseServiceClient;

/// Manages api keys: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct ApiKeyController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    create_api_key_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            edgegateway::NotifyGatewayPolicyWatchersRequest,
            edgegateway::NotifyGatewayPolicyWatchersResponse,
        >,
    >,
    watch_api_keys_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            edgegateway::ConnectApiKeySourceRequest,
            edgegateway::ConnectApiKeySourceResponse,
        >,
    >,
    watch_api_keys_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::ResumeThrottleWorkflowRequest,
                ratelimiting::ResumeThrottleWorkflowResponse,
            >,
    >,
    watch_api_keys_2_port: acme_platform::ports::Callback<
        configuration::ScheduleConfigChangeReportRequest,
        configuration::ScheduleConfigChangeReportResponse,
    >,
}
impl ApiKeyController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::EdgegatewayRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            edgegateway::NotifyGatewayPolicyWatchersRequest,
            edgegateway::NotifyGatewayPolicyWatchersResponse,
        > = {
            let client = GatewayPolicyNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::NotifyGatewayPolicyWatchersRequest| -> acme_platform::ports::RpcFuture<edgegateway::NotifyGatewayPolicyWatchersResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.notify_gateway_policy_watchers(request).await?.into_inner())
})
})
        };
        let create_api_key_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            edgegateway::ConnectApiKeySourceRequest,
            edgegateway::ConnectApiKeySourceResponse,
        > = {
            let client = ApiKeyIntegrationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: edgegateway::ConnectApiKeySourceRequest| -> acme_platform::ports::RpcFuture<edgegateway::ConnectApiKeySourceResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.connect_api_key_source(request).await?.into_inner())
})
})
        };
        let watch_api_keys_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::ResumeThrottleWorkflowRequest,
            ratelimiting::ResumeThrottleWorkflowResponse,
        > = {
            let client = ThrottleWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ResumeThrottleWorkflowRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ResumeThrottleWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resume_throttle_workflow(request).await?.into_inner())
})
})
        };
        let watch_api_keys_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ScheduleConfigChangeReportRequest,
            configuration::ScheduleConfigChangeReportResponse,
        > = {
            let client = ConfigChangeReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ScheduleConfigChangeReportRequest| -> acme_platform::ports::RpcFuture<configuration::ScheduleConfigChangeReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.schedule_config_change_report(request).await?.into_inner())
})
})
        };
        let watch_api_keys_2_port = callback;
        Self {
            channel,
            repository,
            create_api_key_0_port,
            watch_api_keys_0_port,
            watch_api_keys_1_port,
            watch_api_keys_2_port,
        }
    }
    /// Run CreateApiKey after validating the tenant and durably recording the command.
    async fn run_create_api_key(
        &self,
        req: edgegateway::CreateApiKeyRequest,
    ) -> Result<edgegateway::CreateApiKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateApiKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateApiKey", "running workflow");
        tracing::debug!(audit_value = ?req.api_key.clone().unwrap_or_default().api_key_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::NotifyGatewayPolicyWatchersRequest {
            context: req.context.clone(),
            details: Some(edgegateway::notify_gateway_policy_watchers_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .api_key
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
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.create_api_key_0_port.as_ref(), outgoing_0).await?;
        let jobs_1 = req
            .api_key
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = featureflags::AnnotateFeatureFlagVersionRequest {
                        context: req.context.clone(),
                        feature_flag: Some(featureflags::FeatureFlag {
                            audit: Some(common::AuditInfo {
                                etag: req
                                    .api_key
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .etag
                                    .clone(),
                                ..Default::default()
                            }),
                            action: Some(featureflags::feature_flag::Action::Discount(
                                featureflags::FeatureFlagDiscountAction {
                                    amount: item.amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            name: item.name.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = FeatureFlagHistoryServiceClient::new(channel);
                        let input: tonic::Request<featureflags::AnnotateFeatureFlagVersionRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.annotate_feature_flag_version(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = warehousing::BatchCreateWarehousesRequest {
            context: req.context.clone(),
            chunk: Some(warehousing::batch_create_warehouses_request::Chunk::Record(
                warehousing::Warehouse {
                    audit: Some(common::AuditInfo {
                        created_at: req
                            .api_key
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        updated_at: reply_1
                            .feature_flag
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        updated_by: reply_0
                            .gateway_policy
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
                        display_name: reply_1
                            .feature_flag
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = WarehouseServiceClient::new(channel);
            let input: tonic::Request<warehousing::BatchCreateWarehousesRequest> =
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
            let response = client.batch_create_warehouses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    subject: Some(featureflags::FlagRolloutRef {
                        display_name: reply_0
                            .gateway_policy
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
        let mut response = edgegateway::CreateApiKeyResponse::default();
        let contribution: String = reply_0
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
                .api_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_0
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .gateway_policy_id
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .gateway_policy_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.api_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.api_key.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.api_key.get_or_insert_with(Default::default).contact = contribution;
        }
        Ok(response)
    }
    /// Run GetApiKey after validating the tenant and durably recording the command.
    async fn run_get_api_key(
        &self,
        req: edgegateway::GetApiKeyRequest,
    ) -> Result<edgegateway::GetApiKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetApiKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetApiKey", "running workflow");
        let outgoing_0 = edgegateway::ListGatewayPolicySubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_gateway_policy_subscriptions_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(
            &outgoing_0,
            "acme.edgegateway.v1.ListGatewayPolicySubscriptionsRequest",
        )?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = GatewayPolicyNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListGatewayPolicySubscriptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_gateway_policy_subscriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::GetApiKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
    /// Run UpdateApiKey after validating the tenant and durably recording the command.
    async fn run_update_api_key(
        &self,
        req: edgegateway::UpdateApiKeyRequest,
    ) -> Result<edgegateway::UpdateApiKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UpdateApiKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UpdateApiKey", "running workflow");
        tracing::debug!(audit_value = ?req.api_key.clone().unwrap_or_default().holder.clone().unwrap_or_default().gateway_policy_id.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::IngestGatewayPolicyEventsRequest {
            context: req.context.clone(),
            chunk: Some(edgegateway::ingest_gateway_policy_events_request::Chunk::Record(
                edgegateway::GatewayPolicy {
                    audit: Some(common::AuditInfo {
                        etag: req
                            .api_key
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .etag
                            .clone(),
                        updated_by: req
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
                    owner: Some(identity::UserRef {
                        display_name: req
                            .api_key
                            .clone()
                            .unwrap_or_default()
                            .holder
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
        let _reply_0 = async move {
            let mut client = GatewayPolicyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::IngestGatewayPolicyEventsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.ingest_gateway_policy_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = edgegateway::ListRouteBindingSourcesRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_route_binding_sources_request::Filter {
                created_after: req
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.api_key.clone().unwrap_or_default().labels.clone(),
                owner_id: req.api_key.clone().unwrap_or_default().api_key_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RouteBindingIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListRouteBindingSourcesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_route_binding_sources(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::UpdateApiKeyResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: i32 = req.api_key.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.api_key.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.api_key.clone().unwrap_or_default().api_key_id.clone();
        if contribution != <String>::default() {
            response.api_key.get_or_insert_with(Default::default).api_key_id = contribution;
        }
        Ok(response)
    }
    /// Run DeleteApiKey after validating the tenant and durably recording the command.
    async fn run_delete_api_key(
        &self,
        req: edgegateway::DeleteApiKeyRequest,
    ) -> Result<edgegateway::DeleteApiKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteApiKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteApiKey", "running workflow");
        let outgoing_0 = configuration::CountConfigChangeTotalsRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_change_totals_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.api_key_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = edgegateway::AdvanceGatewayPolicyWorkflowRequest {
            context: req.context.clone(),
            details: Some(edgegateway::advance_gateway_policy_workflow_request::Details {
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
                    etag: req.etag.clone(),
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeReportingServiceClient::new(channel_0);
                let input: tonic::Request<configuration::CountConfigChangeTotalsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_config_change_totals(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = GatewayPolicyWorkflowServiceClient::new(channel_1);
                let input: tonic::Request<edgegateway::AdvanceGatewayPolicyWorkflowRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.advance_gateway_policy_workflow(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut command_2 = DeleteApiKeyDispatch2::from(req.clone());
        command_2.filter_created_after = reply_1
            .gateway_policy
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        command_2.filter_labels = reply_1.gateway_policy.clone().unwrap_or_default().parameters.clone();
        let outgoing_2: edgegateway::ListGatewayPolicySourcesRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListGatewayPolicySourcesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_gateway_policy_sources(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::DeleteApiKeyResponse::default();
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
        Ok(response)
    }
    /// Run ListApiKeys after validating the tenant and durably recording the command.
    async fn run_list_api_keys(
        &self,
        req: edgegateway::ListApiKeysRequest,
    ) -> Result<edgegateway::ListApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListApiKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::AdvanceApiKeyWorkflowRequest {
            api_key: Some(edgegateway::ApiKey {
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(edgegateway::advance_api_key_workflow_request::Details {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ApiKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<edgegateway::AdvanceApiKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_api_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = personalization::ExplainProfileCalculationRequest {
            context: req.context.clone(),
            details: Some(personalization::explain_profile_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    preferred_channel: reply_0
                        .api_key
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
            }),
            profile: Some(personalization::Profile {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_by: reply_0
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
                relationship_manager: Some(identity::UserRef {
                    display_name: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
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
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = ProfileCalculationServiceClient::new(channel);
            let input: tonic::Request<personalization::ExplainProfileCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_profile_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = edgegateway::ListApiKeysResponse::default();
        let contribution: i32 = reply_0.api_key.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            if response.api_keys.is_empty() {
                response.api_keys.push(Default::default());
            }
            response.api_keys[0].status = contribution;
        }
        let contribution: String = reply_0.api_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            if response.api_keys.is_empty() {
                response.api_keys.push(Default::default());
            }
            response.api_keys[0].currency_code = contribution;
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
        Ok(response)
    }
    /// Run BatchGetApiKeys after validating the tenant and durably recording the command.
    async fn run_batch_get_api_keys(
        &self,
        req: edgegateway::BatchGetApiKeysRequest,
    ) -> Result<edgegateway::BatchGetApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchGetApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchGetApiKeys", "running workflow");
        let response = edgegateway::BatchGetApiKeysResponse::default();
        Ok(response)
    }
    /// Run WatchApiKeys after validating the tenant and durably recording the command.
    async fn run_watch_api_keys(
        &self,
        req: edgegateway::WatchApiKeysRequest,
    ) -> Result<edgegateway::WatchApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("WatchApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "WatchApiKeys", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = edgegateway::ConnectApiKeySourceRequest {
            annotations: req.filter.clone().unwrap_or_default().labels.clone(),
            context: req.context.clone(),
            details: Some(edgegateway::connect_api_key_source_request::Details {
                received_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.watch_api_keys_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = ratelimiting::ResumeThrottleWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::resume_throttle_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    preferred_channel: reply_0
                        .api_key
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
            }),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    version: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .version
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.watch_api_keys_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::ScheduleConfigChangeReportRequest {
            annotations: reply_0.api_key.clone().unwrap_or_default().labels.clone(),
            context: req.context.clone(),
            details: Some(configuration::schedule_config_change_report_request::Details {
                filters: reply_0.result.clone().unwrap_or_default().details.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = (self.watch_api_keys_2_port)(outgoing_2).await?;
        let mut response = edgegateway::WatchApiKeysResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .api_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_0
            .api_key
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .phone
            .clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .phone = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().preferred_channel.clone();
        if contribution != <String>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .preferred_channel = contribution;
        }
        let contribution: Option<common::PersonName> = req.subscriber.clone().unwrap_or_default().name.clone();
        if contribution != <Option<common::PersonName>>::default() {
            response
                .api_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .name = contribution;
        }
        Ok(response)
    }
    /// Run BatchCreateApiKeys after validating the tenant and durably recording the command.
    async fn run_batch_create_api_keys(
        &self,
        req: edgegateway::BatchCreateApiKeysRequest,
    ) -> Result<edgegateway::BatchCreateApiKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchCreateApiKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateApiKeys", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(edgegateway::batch_create_api_keys_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let command_0 = BatchCreateApiKeysDispatch0::from(req.clone());
        let outgoing_0: edgegateway::ConnectApiKeySourceRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ApiKeyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ConnectApiKeySourceRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.connect_api_key_source(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = patientapp::ClonePatientHomeRequest {
            context: req.context.clone(),
            patient_home: Some(patientapp::PatientHome {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_at: reply_0
                        .api_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                requester: Some(common::ContactInfo {
                    phone: reply_0
                        .api_key
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
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::codec::through_any(&outgoing_1, "acme.patientapp.v1.ClonePatientHomeRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = PatientHomeServiceClient::new(channel);
            let input: tonic::Request<patientapp::ClonePatientHomeRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_patient_home(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::BatchCreateApiKeysResponse::default();
        let contribution: String = reply_0.api_key.clone().unwrap_or_default().api_key_id.clone();
        if contribution != <String>::default() {
            if response.rejections.is_empty() {
                response.rejections.push(Default::default());
            }
            response.rejections[0].api_key_id = contribution;
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
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CloneApiKey after validating the tenant and durably recording the command.
    async fn run_clone_api_key(
        &self,
        req: edgegateway::CloneApiKeyRequest,
    ) -> Result<edgegateway::CloneApiKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CloneApiKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CloneApiKey", "running workflow");
        tracing::debug!(audit_value = ?req.api_key.clone().unwrap_or_default().api_key_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::ListPendingSecretVersionsRequest {
            context: req.context.clone(),
            filter: Some(secrets::list_pending_secret_versions_request::Filter {
                created_after: req.api_key.clone().unwrap_or_default().opened_at.clone(),
                labels: req.api_key.clone().unwrap_or_default().labels.clone(),
                owner_id: req
                    .api_key
                    .clone()
                    .unwrap_or_default()
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .gateway_policy_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<secrets::ListPendingSecretVersionsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = SecretVersionApprovalServiceClient::new(channel);
                let input: tonic::Request<secrets::ListPendingSecretVersionsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_pending_secret_versions(input).await?;
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
        let outgoing_1 = edgegateway::ReplayGatewayPolicyEventsRequest {
            context: req.context.clone(),
            details: Some(edgegateway::replay_gateway_policy_events_request::Details {
                message_id: reply_0.result.clone().unwrap_or_default().message.clone(),
                received_at: req.api_key.clone().unwrap_or_default().opened_at.clone(),
                ..Default::default()
            }),
            gateway_policy_id: req
                .api_key
                .clone()
                .unwrap_or_default()
                .holder
                .clone()
                .unwrap_or_default()
                .gateway_policy_id
                .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = GatewayPolicyIntegrationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ReplayGatewayPolicyEventsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.replay_gateway_policy_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = edgegateway::PeekGatewayPolicyRequest {
            context: req.context.clone(),
            gateway_policy_id: req
                .api_key
                .clone()
                .unwrap_or_default()
                .holder
                .clone()
                .unwrap_or_default()
                .gateway_policy_id
                .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = GatewayPolicyQueryServiceClient::new(channel);
            let input: tonic::Request<edgegateway::PeekGatewayPolicyRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.peek_gateway_policy(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = edgegateway::ListApiKeySubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(edgegateway::list_api_key_subscriptions_request::Filter {
                labels: req.api_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ApiKeyNotificationServiceClient::new(channel);
            let input: tonic::Request<edgegateway::ListApiKeySubscriptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_api_key_subscriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = edgegateway::CloneApiKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_3.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: i32 = req.api_key.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.api_key.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::Money> = req.api_key.clone().unwrap_or_default().balance.clone();
        if contribution != <Option<common::Money>>::default() {
            response.api_key.get_or_insert_with(Default::default).balance = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by DeleteApiKey for ListGatewayPolicySources.
struct DeleteApiKeyDispatch2 {
    context: Option<common::RequestContext>,
    filter_created_after: Option<prost_types::Timestamp>,
    filter_labels: std::collections::BTreeMap<String, String>,
}
impl From<edgegateway::DeleteApiKeyRequest> for DeleteApiKeyDispatch2 {
    fn from(req: edgegateway::DeleteApiKeyRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_created_after: Default::default(),
            filter_labels: Default::default(),
        }
    }
}
impl From<DeleteApiKeyDispatch2> for edgegateway::ListGatewayPolicySourcesRequest {
    fn from(value: DeleteApiKeyDispatch2) -> Self {
        edgegateway::ListGatewayPolicySourcesRequest {
            context: value.context,
            filter: Some(edgegateway::list_gateway_policy_sources_request::Filter {
                created_after: value.filter_created_after,
                labels: value.filter_labels,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by BatchCreateApiKeys for ConnectApiKeySource.
struct BatchCreateApiKeysDispatch0 {
    context: Option<common::RequestContext>,
    api_key_id: String,
    details_received_at: Option<prost_types::Timestamp>,
    details_message_id: String,
    annotations: std::collections::BTreeMap<String, String>,
}
impl From<edgegateway::BatchCreateApiKeysRequest> for BatchCreateApiKeysDispatch0 {
    fn from(req: edgegateway::BatchCreateApiKeysRequest) -> Self {
        Self {
            context: req.context.clone(),
            api_key_id: (match req.chunk.clone() {
                Some(edgegateway::batch_create_api_keys_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .api_key_id
            .clone(),
            details_received_at: (match req.chunk.clone() {
                Some(edgegateway::batch_create_api_keys_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .opened_at
            .clone(),
            details_message_id: (match req.chunk.clone() {
                Some(edgegateway::batch_create_api_keys_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .holder
            .clone()
            .unwrap_or_default()
            .gateway_policy_id
            .clone(),
            annotations: (match req.chunk.clone() {
                Some(edgegateway::batch_create_api_keys_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .labels
            .clone(),
        }
    }
}
impl From<BatchCreateApiKeysDispatch0> for edgegateway::ConnectApiKeySourceRequest {
    fn from(value: BatchCreateApiKeysDispatch0) -> Self {
        edgegateway::ConnectApiKeySourceRequest {
            annotations: value.annotations,
            api_key_id: value.api_key_id,
            context: value.context,
            details: Some(edgegateway::connect_api_key_source_request::Details {
                message_id: value.details_message_id,
                received_at: value.details_received_at,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl ApiKeyService for ApiKeyController {
    /// Creates an api key.
    async fn create_api_key(
        &self,
        request: tonic::Request<edgegateway::CreateApiKeyRequest>,
    ) -> Result<tonic::Response<edgegateway::CreateApiKeyResponse>, tonic::Status> {
        self.run_create_api_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an api key.
    async fn get_api_key(
        &self,
        request: tonic::Request<edgegateway::GetApiKeyRequest>,
    ) -> Result<tonic::Response<edgegateway::GetApiKeyResponse>, tonic::Status> {
        self.run_get_api_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Changes an api key.
    async fn update_api_key(
        &self,
        request: tonic::Request<edgegateway::UpdateApiKeyRequest>,
    ) -> Result<tonic::Response<edgegateway::UpdateApiKeyResponse>, tonic::Status> {
        self.run_update_api_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes an api key.
    async fn delete_api_key(
        &self,
        request: tonic::Request<edgegateway::DeleteApiKeyRequest>,
    ) -> Result<tonic::Response<edgegateway::DeleteApiKeyResponse>, tonic::Status> {
        self.run_delete_api_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists api keys a page at a time.
    async fn list_api_keys(
        &self,
        request: tonic::Request<edgegateway::ListApiKeysRequest>,
    ) -> Result<tonic::Response<edgegateway::ListApiKeysResponse>, tonic::Status> {
        self.run_list_api_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several api keys at once.
    async fn batch_get_api_keys(
        &self,
        request: tonic::Request<edgegateway::BatchGetApiKeysRequest>,
    ) -> Result<tonic::Response<edgegateway::BatchGetApiKeysResponse>, tonic::Status> {
        self.run_batch_get_api_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type WatchApiKeysStream =
        tokio_stream::wrappers::ReceiverStream<Result<edgegateway::WatchApiKeysResponse, tonic::Status>>;
    /// Streams the changes of api keys as they happen.
    async fn watch_api_keys(
        &self,
        request: tonic::Request<edgegateway::WatchApiKeysRequest>,
    ) -> Result<tonic::Response<Self::WatchApiKeysStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_watch_api_keys(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Takes a stream of api keys and answers once at the end.
    async fn batch_create_api_keys(
        &self,
        request: tonic::Request<tonic::Streaming<edgegateway::BatchCreateApiKeysRequest>>,
    ) -> Result<tonic::Response<edgegateway::BatchCreateApiKeysResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = edgegateway::BatchCreateApiKeysResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_batch_create_api_keys(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Creates an api key.
    async fn clone_api_key(
        &self,
        request: tonic::Request<edgegateway::CloneApiKeyRequest>,
    ) -> Result<tonic::Response<edgegateway::CloneApiKeyResponse>, tonic::Status> {
        self.run_clone_api_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
