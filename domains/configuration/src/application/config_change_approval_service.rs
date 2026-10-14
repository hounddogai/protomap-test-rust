use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_server::ConfigChangeApprovalService;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::config_set_approval_service_client::ConfigSetApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_set_query_service_client::ConfigSetQueryServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::contracts::v1 as contracts;
use acme_proto::acme::contracts::v1::contract_service_client::ContractServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_query_service_client::UserQueryServiceClient;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_query_service_client::MetricSeriesQueryServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::warehousing::v1 as warehousing;
use acme_proto::acme::warehousing::v1::warehouse_service_client::WarehouseServiceClient;

/// Routes config changes through approval, with escalation and delegation.
#[derive(Clone)]
pub struct ConfigChangeApprovalHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    approve_config_change_0_port: acme_platform::ports::Callback<
        configuration::ListEnvironmentConflictsRequest,
        configuration::ListEnvironmentConflictsResponse,
    >,
    approve_config_change_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<configuration::SubmitConfigSetRequest, configuration::SubmitConfigSetResponse>,
    >,
    delegate_config_change_approval_0_port: acme_platform::ports::Callback<
        configuration::ExplainEnvironmentDecisionRequest,
        configuration::ExplainEnvironmentDecisionResponse,
    >,
    reject_config_change_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<warehousing::SearchWarehousesRequest, warehousing::SearchWarehousesResponse>,
    >,
    submit_config_change_0_port: acme_platform::ports::Callback<
        ratelimiting::RecalculateQuotaBucketsRequest,
        ratelimiting::RecalculateQuotaBucketsResponse,
    >,
    submit_config_change_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<identity::PeekUserRequest, identity::PeekUserResponse>,
    >,
    submit_config_change_2_port: acme_platform::ports::Callback<
        configuration::SummarizeConfigChangesRequest,
        configuration::SummarizeConfigChangesResponse,
    >,
}
impl ConfigChangeApprovalHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::RecalculateQuotaBucketsRequest,
            ratelimiting::RecalculateQuotaBucketsResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::RecalculateQuotaBucketsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::RecalculateQuotaBucketsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_quota_buckets(request).await?.into_inner())
})
})
        };
        let submit_config_change_0_port = callback;
        let callback: acme_platform::ports::Callback<identity::PeekUserRequest, identity::PeekUserResponse> = {
            let client = UserQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: identity::PeekUserRequest| -> acme_platform::ports::RpcFuture<identity::PeekUserResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.peek_user(request).await?.into_inner())
})
})
        };
        let submit_config_change_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::SummarizeConfigChangesRequest,
            configuration::SummarizeConfigChangesResponse,
        > = {
            let client = ConfigChangeQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SummarizeConfigChangesRequest| -> acme_platform::ports::RpcFuture<configuration::SummarizeConfigChangesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.summarize_config_changes(request).await?.into_inner())
})
})
        };
        let submit_config_change_2_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::ListEnvironmentConflictsRequest,
            configuration::ListEnvironmentConflictsResponse,
        > = {
            let client = EnvironmentSyncServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ListEnvironmentConflictsRequest| -> acme_platform::ports::RpcFuture<configuration::ListEnvironmentConflictsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_environment_conflicts(request).await?.into_inner())
})
})
        };
        let approve_config_change_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::SubmitConfigSetRequest,
            configuration::SubmitConfigSetResponse,
        > = {
            let client = ConfigSetApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SubmitConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::SubmitConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.submit_config_set(request).await?.into_inner())
})
})
        };
        let approve_config_change_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            warehousing::SearchWarehousesRequest,
            warehousing::SearchWarehousesResponse,
        > = {
            let client = WarehouseServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: warehousing::SearchWarehousesRequest| -> acme_platform::ports::RpcFuture<warehousing::SearchWarehousesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.search_warehouses(request).await?.into_inner())
})
})
        };
        let reject_config_change_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let delegate_config_change_approval_0_port = callback;
        Self {
            channel,
            repository,
            approve_config_change_0_port,
            approve_config_change_1_port,
            delegate_config_change_approval_0_port,
            reject_config_change_0_port,
            submit_config_change_0_port,
            submit_config_change_1_port,
            submit_config_change_2_port,
        }
    }
    /// Run SubmitConfigChange after validating the tenant and durably recording the command.
    async fn run_submit_config_change(
        &self,
        req: configuration::SubmitConfigChangeRequest,
    ) -> Result<configuration::SubmitConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::RecalculateQuotaBucketsRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                holder: Some(ratelimiting::RateLimitRef {
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
        let reply_0 = (self.submit_config_change_0_port)(outgoing_0).await?;
        let outgoing_1 = identity::PeekUserRequest {
            context: req.context.clone(),
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
        };
        let reply_1 =
            self.submit_config_change_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = configuration::SummarizeConfigChangesRequest {
            annotations: req.annotations.clone(),
            config_change_id: req.config_change_id.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.submit_config_change_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let outgoing_3 = contracts::BatchCreateContractsRequest {
            context: req.context.clone(),
            chunk: Some(contracts::batch_create_contracts_request::Chunk::Header(
                contracts::batch_create_contracts_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
                        name: reply_1
                            .user
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        preferred_channel: reply_1
                            .user
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ContractServiceClient::new(channel);
            let input: tonic::Request<contracts::BatchCreateContractsRequest> =
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
            let response = client.batch_create_contracts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        updated_by: reply_1
                            .user
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
        let mut response = configuration::SubmitConfigChangeResponse::default();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: Option<configuration::ConfigSetRef> =
            req.config_change.clone().unwrap_or_default().subject.clone();
        if contribution != <Option<configuration::ConfigSetRef>>::default() {
            response.config_change.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run ApproveConfigChange after validating the tenant and durably recording the command.
    async fn run_approve_config_change(
        &self,
        req: configuration::ApproveConfigChangeRequest,
    ) -> Result<configuration::ApproveConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ListEnvironmentConflictsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_environment_conflicts_request::Filter {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.approve_config_change_0_port)(outgoing_0).await?;
        let outgoing_1 = configuration::SubmitConfigSetRequest {
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
        let reply_1 =
            self.approve_config_change_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = configuration::ApproveConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
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
    /// Run RejectConfigChange after validating the tenant and durably recording the command.
    async fn run_reject_config_change(
        &self,
        req: configuration::RejectConfigChangeRequest,
    ) -> Result<configuration::RejectConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = warehousing::SearchWarehousesRequest {
            context: req.context.clone(),
            filter: Some(warehousing::search_warehouses_request::Filter {
                created_after: req.config_change.clone().unwrap_or_default().occurred_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req.config_change.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.reject_config_change_0_port.as_ref(), outgoing_0).await?;
        let jobs_1 = req
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
                    let outgoing_1 = configuration::StreamConfigChangeMetricsRequest {
                        context: req.context.clone(),
                        filter: Some(configuration::stream_config_change_metrics_request::Filter {
                            created_after: item.observed_at.clone(),
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
                        subscriber: Some(common::ContactInfo {
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .approver
                                .clone()
                                .unwrap_or_default()
                                .email
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = ConfigChangeReportingServiceClient::new(channel);
                        let input: tonic::Request<configuration::StreamConfigChangeMetricsRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.stream_config_change_metrics(input).await?;
                        let mut frames = response.into_inner();
                        let mut last = configuration::StreamConfigChangeMetricsResponse::default();
                        // The caller asks for a bounded snapshot, then drops its subscription.
                        for _ in 0..8 {
                            match frames.message().await? {
                                Some(frame) => last = frame,
                                None => break,
                            }
                        }
                        Ok::<_, tonic::Status>(last)
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = configuration::RejectConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
    /// Run ListPendingConfigChanges after validating the tenant and durably recording the command.
    async fn run_list_pending_config_changes(
        &self,
        req: configuration::ListPendingConfigChangesRequest,
    ) -> Result<configuration::ListPendingConfigChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPendingConfigChanges", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingConfigChanges", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::SubmitConfigSetRequest {
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
                location: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::SubmitConfigSetRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigSetApprovalServiceClient::new(channel);
                let input: tonic::Request<configuration::SubmitConfigSetRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.submit_config_set(input).await?;
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
        let outgoing_1 = notifications::BatchGetNotificationsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = NotificationServiceClient::new(channel);
            let input: tonic::Request<notifications::BatchGetNotificationsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_notifications(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = configuration::ListPendingConfigChangesResponse::default();
        let contribution: String = reply_0.config_set.clone().unwrap_or_default().config_set_id.clone();
        if contribution != <String>::default() {
            if response.config_changes.is_empty() {
                response.config_changes.push(Default::default());
            }
            response.config_changes[0]
                .subject
                .get_or_insert_with(Default::default)
                .config_set_id = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run DelegateConfigChangeApproval after validating the tenant and durably recording the command.
    async fn run_delegate_config_change_approval(
        &self,
        req: configuration::DelegateConfigChangeApprovalRequest,
    ) -> Result<configuration::DelegateConfigChangeApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateConfigChangeApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateConfigChangeApproval", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ExplainEnvironmentDecisionRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    display_name: req
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
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
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.delegate_config_change_approval_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = metrics::SuggestMetricSeriesesRequest {
            context: req.context.clone(),
            filter: Some(metrics::suggest_metric_serieses_request::Filter {
                created_after: reply_0
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<metrics::SuggestMetricSeriesesRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = MetricSeriesQueryServiceClient::new(channel);
            let input: tonic::Request<metrics::SuggestMetricSeriesesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_metric_serieses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = DelegateConfigChangeApprovalDispatch2::from(req.clone());
        command_2.certificate_audit_etag = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        let outgoing_2: certificates::LintCertificatesRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::LintCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::LookupConfigSetRequest {
            config_set_id: req
                .config_change
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .config_set_id
                .clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.configuration.v1.LookupConfigSetRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigSetQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::LookupConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lookup_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::DelegateConfigChangeApprovalResponse::default();
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
            response
                .config_change
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
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
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
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
    /// Run RecallConfigChange after validating the tenant and durably recording the command.
    async fn run_recall_config_change(
        &self,
        req: configuration::RecallConfigChangeRequest,
    ) -> Result<configuration::RecallConfigChangeResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallConfigChange", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallConfigChange", "running workflow");
        tracing::debug!(audit_value = ?req.config_change.clone().unwrap_or_default().config_change_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::EstimateFlagRolloutRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                organizer: Some(identity::UserRef {
                    display_name: req
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
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
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .approver
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::EstimateFlagRolloutRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_flag_rollout(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::ExportCertificateAuthoritiesRequest {
            context: req.context.clone(),
            filter: Some(certificates::export_certificate_authorities_request::Filter {
                created_after: reply_0
                    .flag_rollout
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.config_change.clone().unwrap_or_default().attributes.clone(),
                owner_id: req.config_change.clone().unwrap_or_default().correlation_id.clone(),
                region: reply_0
                    .flag_rollout
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
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = CertificateAuthorityQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::ExportCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_certificate_authorities(input).await?;
            let mut frames = response.into_inner();
            let mut last = certificates::ExportCertificateAuthoritiesResponse::default();
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
        let mut response = configuration::RecallConfigChangeResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .organizer
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
        let contribution: String = req.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .config_change
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by DelegateConfigChangeApproval for LintCertificates.
struct DelegateConfigChangeApprovalDispatch2 {
    context: Option<common::RequestContext>,
    certificate_audit_etag: String,
}
impl From<configuration::DelegateConfigChangeApprovalRequest> for DelegateConfigChangeApprovalDispatch2 {
    fn from(req: configuration::DelegateConfigChangeApprovalRequest) -> Self {
        Self {
            context: req.context.clone(),
            certificate_audit_etag: Default::default(),
        }
    }
}
impl From<DelegateConfigChangeApprovalDispatch2> for certificates::LintCertificatesRequest {
    fn from(value: DelegateConfigChangeApprovalDispatch2) -> Self {
        certificates::LintCertificatesRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    etag: value.certificate_audit_etag,
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
impl ConfigChangeApprovalService for ConfigChangeApprovalHandler {
    /// Submits a config change.
    async fn submit_config_change(
        &self,
        request: tonic::Request<configuration::SubmitConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::SubmitConfigChangeResponse>, tonic::Status> {
        self.run_submit_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a config change.
    async fn approve_config_change(
        &self,
        request: tonic::Request<configuration::ApproveConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::ApproveConfigChangeResponse>, tonic::Status> {
        self.run_approve_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a config change.
    async fn reject_config_change(
        &self,
        request: tonic::Request<configuration::RejectConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::RejectConfigChangeResponse>, tonic::Status> {
        self.run_reject_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists config changes a page at a time.
    async fn list_pending_config_changes(
        &self,
        request: tonic::Request<configuration::ListPendingConfigChangesRequest>,
    ) -> Result<tonic::Response<configuration::ListPendingConfigChangesResponse>, tonic::Status> {
        self.run_list_pending_config_changes(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates a config change.
    async fn delegate_config_change_approval(
        &self,
        request: tonic::Request<configuration::DelegateConfigChangeApprovalRequest>,
    ) -> Result<tonic::Response<configuration::DelegateConfigChangeApprovalResponse>, tonic::Status> {
        self.run_delegate_config_change_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls a config change.
    async fn recall_config_change(
        &self,
        request: tonic::Request<configuration::RecallConfigChangeRequest>,
    ) -> Result<tonic::Response<configuration::RecallConfigChangeResponse>, tonic::Status> {
        self.run_recall_config_change(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
