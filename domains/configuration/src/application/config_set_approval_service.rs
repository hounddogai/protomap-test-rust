use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_history_service_client::RoleBindingHistoryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_approval_service_server::ConfigSetApprovalService;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::customers::v1 as customers;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_service_client::DocumentServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::fulfillment::v1::fulfillment_query_service_client::FulfillmentQueryServiceClient;
use acme_proto::acme::fulfillment::v1::fulfillment_service_client::FulfillmentServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::secret_version_validation_service_client::SecretVersionValidationServiceClient;

/// Routes config sets through approval, with escalation and delegation.
#[derive(Clone)]
pub struct ConfigSetApprovalEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    comment_config_set_approval_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            authorization::ListRoleBindingVersionsRequest,
            authorization::ListRoleBindingVersionsResponse,
        >,
    >,
    delegate_config_set_approval_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::ShareConfigChangeReportRequest,
                configuration::ShareConfigChangeReportResponse,
            >,
    >,
    delegate_config_set_approval_1_port: acme_platform::ports::Callback<
        configuration::GetConfigSetLifecycleRequest,
        configuration::GetConfigSetLifecycleResponse,
    >,
    escalate_config_set_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::ScheduleConfigChangeExpiryRequest,
            configuration::ScheduleConfigChangeExpiryResponse,
        >,
    >,
    reject_config_set_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ListEnvironmentReportsRequest,
            configuration::ListEnvironmentReportsResponse,
        >,
    >,
    reject_config_set_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                certificates::ResolveCertificatesRequest,
                certificates::ResolveCertificatesResponse,
            >,
    >,
    submit_config_set_0_port: acme_platform::ports::Callback<
        configuration::FetchConfigSetReportRequest,
        configuration::FetchConfigSetReportResponse,
    >,
    submit_config_set_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<fulfillment::CountFulfillmentsRequest, fulfillment::CountFulfillmentsResponse>,
    >,
}
impl ConfigSetApprovalEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::FetchConfigSetReportRequest,
            configuration::FetchConfigSetReportResponse,
        > = {
            let client = ConfigSetReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::FetchConfigSetReportRequest| -> acme_platform::ports::RpcFuture<configuration::FetchConfigSetReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.fetch_config_set_report(request).await?.into_inner())
})
})
        };
        let submit_config_set_0_port = callback;
        let callback: acme_platform::ports::Callback<
            fulfillment::CountFulfillmentsRequest,
            fulfillment::CountFulfillmentsResponse,
        > = {
            let client = FulfillmentQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: fulfillment::CountFulfillmentsRequest| -> acme_platform::ports::RpcFuture<fulfillment::CountFulfillmentsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_fulfillments(request).await?.into_inner())
})
})
        };
        let submit_config_set_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::ListEnvironmentReportsRequest,
            configuration::ListEnvironmentReportsResponse,
        > = {
            let client = EnvironmentReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ListEnvironmentReportsRequest| -> acme_platform::ports::RpcFuture<configuration::ListEnvironmentReportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_environment_reports(request).await?.into_inner())
})
})
        };
        let reject_config_set_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::ResolveCertificatesRequest,
            certificates::ResolveCertificatesResponse,
        > = {
            let client = CertificateQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::ResolveCertificatesRequest| -> acme_platform::ports::RpcFuture<certificates::ResolveCertificatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_certificates(request).await?.into_inner())
})
})
        };
        let reject_config_set_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ScheduleConfigChangeExpiryRequest,
            configuration::ScheduleConfigChangeExpiryResponse,
        > = {
            let client = ConfigChangeLifecycleServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ScheduleConfigChangeExpiryRequest| -> acme_platform::ports::RpcFuture<configuration::ScheduleConfigChangeExpiryResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.schedule_config_change_expiry(request).await?.into_inner())
})
})
        };
        let escalate_config_set_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::ShareConfigChangeReportRequest,
            configuration::ShareConfigChangeReportResponse,
        > = {
            let client = ConfigChangeReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ShareConfigChangeReportRequest| -> acme_platform::ports::RpcFuture<configuration::ShareConfigChangeReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_config_change_report(request).await?.into_inner())
})
})
        };
        let delegate_config_set_approval_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::GetConfigSetLifecycleRequest,
            configuration::GetConfigSetLifecycleResponse,
        > = {
            let client = ConfigSetLifecycleServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::GetConfigSetLifecycleRequest| -> acme_platform::ports::RpcFuture<configuration::GetConfigSetLifecycleResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.get_config_set_lifecycle(request).await?.into_inner())
})
})
        };
        let delegate_config_set_approval_1_port = callback;
        let callback: acme_platform::ports::Callback<
            authorization::ListRoleBindingVersionsRequest,
            authorization::ListRoleBindingVersionsResponse,
        > = {
            let client = RoleBindingHistoryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::ListRoleBindingVersionsRequest| -> acme_platform::ports::RpcFuture<authorization::ListRoleBindingVersionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_role_binding_versions(request).await?.into_inner())
})
})
        };
        let comment_config_set_approval_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            comment_config_set_approval_1_port,
            delegate_config_set_approval_0_port,
            delegate_config_set_approval_1_port,
            escalate_config_set_0_port,
            reject_config_set_0_port,
            reject_config_set_1_port,
            submit_config_set_0_port,
            submit_config_set_1_port,
        }
    }
    /// Run SubmitConfigSet after validating the tenant and durably recording the command.
    async fn run_submit_config_set(
        &self,
        req: configuration::SubmitConfigSetRequest,
    ) -> Result<configuration::SubmitConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::FetchConfigSetReportRequest {
            config_set_id: req.config_set.clone().unwrap_or_default().config_set_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = (self.submit_config_set_0_port)(outgoing_0).await?;
        let outgoing_1 = fulfillment::CountFulfillmentsRequest {
            context: req.context.clone(),
            filter: Some(fulfillment::count_fulfillments_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
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
        };
        let reply_1 =
            self.submit_config_set_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = fulfillment::BatchCreateFulfillmentsRequest {
            context: req.context.clone(),
            chunk: Some(fulfillment::batch_create_fulfillments_request::Chunk::Record(
                fulfillment::Fulfillment {
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
                    billing_address: Some(common::PostalAddress {
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
                    counterparty: Some(customers::CustomerRef {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FulfillmentServiceClient::new(channel);
            let input: tonic::Request<fulfillment::BatchCreateFulfillmentsRequest> =
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
            let response = client.batch_create_fulfillments(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::StreamEnvironmentChangesRequest {
            frame: Some(configuration::stream_environment_changes_request::Frame::Change(
                configuration::Environment {
                    locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                        lines: reply_0
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .location
                            .clone()
                            .unwrap_or_default()
                            .lines
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
                    })),
                    owner: Some(identity::UserRef {
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamEnvironmentChangesRequest> =
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
            let response = client.stream_environment_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::StreamEnvironmentChangesResponse::default();
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
        let mut response = configuration::SubmitConfigSetResponse::default();
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
            response.config_set.get_or_insert_with(Default::default).children[0].config_set_id = contribution;
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
        let contribution: i32 = req.config_set.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.config_set.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ApproveConfigSet after validating the tenant and durably recording the command.
    async fn run_approve_config_set(
        &self,
        req: configuration::ApproveConfigSetRequest,
    ) -> Result<configuration::ApproveConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::QuoteConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        created_at: req
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        ..Default::default()
                    }),
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
                        manager: Some(identity::UserRef {
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
            let input: tonic::Request<configuration::QuoteConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = configuration::ValidateConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                attributes: (req.config_set.clone().unwrap_or_default().attributes.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                subject: Some(configuration::ConfigSetRef {
                    config_set_id: req.config_set.clone().unwrap_or_default().config_set_id.clone(),
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
            details: Some(configuration::validate_config_change_request::Details {
                contact: Some(common::ContactInfo {
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
        let reply_1 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ValidateConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ApproveConfigSetResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
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
        let contribution: Option<prost_types::Timestamp> = reply_0
            .config_set
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
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .config_change
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
    /// Run RejectConfigSet after validating the tenant and durably recording the command.
    async fn run_reject_config_set(
        &self,
        req: configuration::RejectConfigSetRequest,
    ) -> Result<configuration::RejectConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ListEnvironmentReportsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_environment_reports_request::Filter {
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
        };
        let reply_0 = acme_platform::ports::execute(self.reject_config_set_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = certificates::ResolveCertificatesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.reject_config_set_1_port.as_ref(), outgoing_1).await?;
        let mut response = configuration::RejectConfigSetResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
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
    /// Run EscalateConfigSet after validating the tenant and durably recording the command.
    async fn run_escalate_config_set(
        &self,
        req: configuration::EscalateConfigSetRequest,
    ) -> Result<configuration::EscalateConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::ScheduleConfigChangeExpiryRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    config_set_id: req.config_set.clone().unwrap_or_default().config_set_id.clone(),
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
            context: req.context.clone(),
            details: Some(configuration::schedule_config_change_expiry_request::Details {
                notify: Some(common::ContactInfo {
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
            }),
            ..Default::default()
        };
        let reply_0 =
            self.escalate_config_set_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = documents::WatchDocumentsRequest {
            context: req.context.clone(),
            filter: Some(documents::watch_documents_request::Filter {
                created_after: req
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
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
            subscriber: Some(common::ContactInfo {
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
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = DocumentServiceClient::new(channel);
            let input: tonic::Request<documents::WatchDocumentsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_documents(input).await?;
            let mut frames = response.into_inner();
            let mut last = documents::WatchDocumentsResponse::default();
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
        let mut response = configuration::EscalateConfigSetResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingConfigSets after validating the tenant and durably recording the command.
    async fn run_list_pending_config_sets(
        &self,
        req: configuration::ListPendingConfigSetsRequest,
    ) -> Result<configuration::ListPendingConfigSetsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListPendingConfigSets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingConfigSets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::ScheduleEnvironmentExpiryRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                })),
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.ScheduleEnvironmentExpiryRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentLifecycleServiceClient::new(channel);
            let input: tonic::Request<configuration::ScheduleEnvironmentExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_environment_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ListPendingConfigSetsResponse::default();
        let contribution: String = (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .region
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
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run DelegateConfigSetApproval after validating the tenant and durably recording the command.
    async fn run_delegate_config_set_approval(
        &self,
        req: configuration::DelegateConfigSetApprovalRequest,
    ) -> Result<configuration::DelegateConfigSetApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateConfigSetApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateConfigSetApproval", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::ShareConfigChangeReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(configuration::share_config_change_report_request::Details {
                recipient: Some(common::ContactInfo {
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
                variables: req.config_set.clone().unwrap_or_default().attributes.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.delegate_config_set_approval_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::GetConfigSetLifecycleRequest {
            config_set_id: req.config_set.clone().unwrap_or_default().config_set_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = (self.delegate_config_set_approval_1_port)(outgoing_1).await?;
        let mut response = configuration::DelegateConfigSetApprovalResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
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
            response
                .config_set
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
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
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.config_set.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.config_set.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run RecallConfigSet after validating the tenant and durably recording the command.
    async fn run_recall_config_set(
        &self,
        req: configuration::RecallConfigSetRequest,
    ) -> Result<configuration::RecallConfigSetResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallConfigSet", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallConfigSet", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ScheduleConfigChangeReportRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::ScheduleConfigChangeReportRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigChangeReportingServiceClient::new(channel);
                let input: tonic::Request<configuration::ScheduleConfigChangeReportRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.schedule_config_change_report(input).await?;
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
        let outgoing_1 = certificates::QuoteCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
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
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
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
                })),
                ..Default::default()
            }),
            details: Some(certificates::quote_csr_request::Details {
                destination: Some(common::PostalAddress {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CsrCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::QuoteCsrRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::EstimateFlagRolloutRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                location: Some(common::PostalAddress {
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
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: reply_1
                            .csr
                            .clone()
                            .unwrap_or_default()
                            .assignee
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        name: reply_1
                            .csr
                            .clone()
                            .unwrap_or_default()
                            .requester
                            .clone()
                            .unwrap_or_default()
                            .name
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
        let reply_2 = async move {
            let mut client = FlagRolloutCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::EstimateFlagRolloutRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_flag_rollout(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = secrets::LintSecretVersionsRequest {
            context: req.context.clone(),
            details: Some(secrets::lint_secret_versions_request::Details {
                contact: Some(common::ContactInfo {
                    email: (match reply_1.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .email
                    .clone(),
                    name: reply_1
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret_version: Some(secrets::SecretVersion {
                attributes: req.config_set.clone().unwrap_or_default().attributes.clone(),
                subject: Some(secrets::AccessGrantRef {
                    display_name: reply_2
                        .flag_rollout
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = SecretVersionValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::LintSecretVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_secret_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::RecallConfigSetResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = (match reply_1.csr.clone().unwrap_or_default().origin.clone() {
            Some(certificates::csr::Origin::Customer(value)) => value,
            _ => Default::default(),
        })
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
        let contribution: Option<prost_types::Timestamp> = reply_1
            .csr
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
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
                .created_at = contribution;
        }
        let contribution: String = reply_2
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: String = reply_2
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        if contribution != <String>::default() {
            response
                .config_set
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
    /// Run GetConfigSetApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_config_set_approval_status(
        &self,
        req: configuration::GetConfigSetApprovalStatusRequest,
    ) -> Result<configuration::GetConfigSetApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetConfigSetApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetConfigSetApprovalStatus", "running workflow");
        let outgoing_0 = configuration::FetchConfigSetReportRequest {
            config_set_id: req.config_set_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<configuration::FetchConfigSetReportRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::FetchConfigSetReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.fetch_config_set_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = documents::WatchDocumentsRequest {
            context: req.context.clone(),
            filter: Some(documents::watch_documents_request::Filter {
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
            let mut client = DocumentServiceClient::new(channel);
            let input: tonic::Request<documents::WatchDocumentsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_documents(input).await?;
            let mut frames = response.into_inner();
            let mut last = documents::WatchDocumentsResponse::default();
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
        let mut response = configuration::GetConfigSetApprovalStatusResponse::default();
        let contribution: String = reply_0.config_set.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).parent_id = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        Ok(response)
    }
    /// Run CommentConfigSetApproval after validating the tenant and durably recording the command.
    async fn run_comment_config_set_approval(
        &self,
        req: configuration::CommentConfigSetApprovalRequest,
    ) -> Result<configuration::CommentConfigSetApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CommentConfigSetApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CommentConfigSetApproval", "running workflow");
        tracing::debug!(audit_value = ?req.config_set.clone().unwrap_or_default().config_set_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ScheduleConfigChangeExpiryRequest {
            config_change: Some(configuration::ConfigChange {
                attributes: (req.config_set.clone().unwrap_or_default().attributes.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
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
            details: Some(configuration::schedule_config_change_expiry_request::Details {
                notify: Some(common::ContactInfo {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeLifecycleServiceClient::new(channel);
            let input: tonic::Request<configuration::ScheduleConfigChangeExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_config_change_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::ListRoleBindingVersionsRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_role_binding_versions_request::Filter {
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
        };
        let reply_1 =
            acme_platform::ports::execute(self.comment_config_set_approval_1_port.as_ref(), outgoing_1).await?;
        let mut response = configuration::CommentConfigSetApprovalResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.config_set_id.clone();
        if contribution != <String>::default() {
            response.config_set.get_or_insert_with(Default::default).config_set_id = contribution;
        }
        let contribution: i32 = req.config_set.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.config_set.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl ConfigSetApprovalService for ConfigSetApprovalEndpoint {
    /// Submits a config set.
    async fn submit_config_set(
        &self,
        request: tonic::Request<configuration::SubmitConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::SubmitConfigSetResponse>, tonic::Status> {
        self.run_submit_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a config set.
    async fn approve_config_set(
        &self,
        request: tonic::Request<configuration::ApproveConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::ApproveConfigSetResponse>, tonic::Status> {
        self.run_approve_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a config set.
    async fn reject_config_set(
        &self,
        request: tonic::Request<configuration::RejectConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::RejectConfigSetResponse>, tonic::Status> {
        self.run_reject_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a config set.
    async fn escalate_config_set(
        &self,
        request: tonic::Request<configuration::EscalateConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::EscalateConfigSetResponse>, tonic::Status> {
        self.run_escalate_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists config sets a page at a time.
    async fn list_pending_config_sets(
        &self,
        request: tonic::Request<configuration::ListPendingConfigSetsRequest>,
    ) -> Result<tonic::Response<configuration::ListPendingConfigSetsResponse>, tonic::Status> {
        self.run_list_pending_config_sets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates a config set.
    async fn delegate_config_set_approval(
        &self,
        request: tonic::Request<configuration::DelegateConfigSetApprovalRequest>,
    ) -> Result<tonic::Response<configuration::DelegateConfigSetApprovalResponse>, tonic::Status> {
        self.run_delegate_config_set_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls a config set.
    async fn recall_config_set(
        &self,
        request: tonic::Request<configuration::RecallConfigSetRequest>,
    ) -> Result<tonic::Response<configuration::RecallConfigSetResponse>, tonic::Status> {
        self.run_recall_config_set(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a config set.
    async fn get_config_set_approval_status(
        &self,
        request: tonic::Request<configuration::GetConfigSetApprovalStatusRequest>,
    ) -> Result<tonic::Response<configuration::GetConfigSetApprovalStatusResponse>, tonic::Status> {
        self.run_get_config_set_approval_status(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Comments a config set.
    async fn comment_config_set_approval(
        &self,
        request: tonic::Request<configuration::CommentConfigSetApprovalRequest>,
    ) -> Result<tonic::Response<configuration::CommentConfigSetApprovalResponse>, tonic::Status> {
        self.run_comment_config_set_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
