use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::configuration::v1::environment_approval_service_server::EnvironmentApprovalService;
use acme_proto::acme::configuration::v1::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_client::FeatureFlagHistoryServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::giftcards::v1::gift_card_service_client::GiftCardServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::itsm::v1 as itsm;
use acme_proto::acme::itsm::v1::incident_service_client::IncidentServiceClient;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_service_client::CryptoKeyServiceClient;
use acme_proto::acme::pricing::v1 as pricing;
use acme_proto::acme::pricing::v1::price_list_service_client::PriceListServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;

/// Routes environments through approval, with escalation and delegation.
#[derive(Clone)]
pub struct EnvironmentApprovalEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    approve_environment_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::RejectConfigChangeRequest,
                configuration::RejectConfigChangeResponse,
            >,
    >,
    approve_environment_2_port: acme_platform::ports::Callback<
        configuration::AdvanceConfigSetWorkflowRequest,
        configuration::AdvanceConfigSetWorkflowResponse,
    >,
    comment_environment_approval_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::PauseConfigSetWorkflowRequest,
            configuration::PauseConfigSetWorkflowResponse,
        >,
    >,
    comment_environment_approval_1_port: acme_platform::ports::Callback<
        configuration::ApproveConfigChangeRequest,
        configuration::ApproveConfigChangeResponse,
    >,
    recall_environment_3_port: acme_platform::ports::Callback<
        certificates::NormalizeCertificateRequest,
        certificates::NormalizeCertificateResponse,
    >,
}
impl EnvironmentApprovalEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
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
        let approve_environment_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::AdvanceConfigSetWorkflowRequest,
            configuration::AdvanceConfigSetWorkflowResponse,
        > = {
            let client = ConfigSetWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::AdvanceConfigSetWorkflowRequest| -> acme_platform::ports::RpcFuture<configuration::AdvanceConfigSetWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.advance_config_set_workflow(request).await?.into_inner())
})
})
        };
        let approve_environment_2_port = callback;
        let callback: acme_platform::ports::Callback<
            certificates::NormalizeCertificateRequest,
            certificates::NormalizeCertificateResponse,
        > = {
            let client = CertificateValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::NormalizeCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::NormalizeCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_certificate(request).await?.into_inner())
})
})
        };
        let recall_environment_3_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::PauseConfigSetWorkflowRequest,
            configuration::PauseConfigSetWorkflowResponse,
        > = {
            let client = ConfigSetWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::PauseConfigSetWorkflowRequest| -> acme_platform::ports::RpcFuture<configuration::PauseConfigSetWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.pause_config_set_workflow(request).await?.into_inner())
})
})
        };
        let comment_environment_approval_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::ApproveConfigChangeRequest,
            configuration::ApproveConfigChangeResponse,
        > = {
            let client = ConfigChangeApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ApproveConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::ApproveConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.approve_config_change(request).await?.into_inner())
})
})
        };
        let comment_environment_approval_1_port = callback;
        Self {
            channel,
            repository,
            approve_environment_1_port,
            approve_environment_2_port,
            comment_environment_approval_0_port,
            comment_environment_approval_1_port,
            recall_environment_3_port,
        }
    }
    /// Run SubmitEnvironment after validating the tenant and durably recording the command.
    async fn run_submit_environment(
        &self,
        req: configuration::SubmitEnvironmentRequest,
    ) -> Result<configuration::SubmitEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::BatchGetCryptoKeysRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = giftcards::SearchGiftCardsRequest {
            context: req.context.clone(),
            filter: Some(giftcards::search_gift_cards_request::Filter {
                created_after: req
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: (match req.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CryptoKeyServiceClient::new(channel_0);
                let input: tonic::Request<keymanagement::BatchGetCryptoKeysRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.batch_get_crypto_keys(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = GiftCardServiceClient::new(channel_1);
                let input: tonic::Request<giftcards::SearchGiftCardsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.search_gift_cards(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = authorization::ListAccessPolicyVersionsRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_access_policy_versions_request::Filter {
                created_after: req
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: (match req.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<authorization::ListAccessPolicyVersionsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = AccessPolicyHistoryServiceClient::new(channel);
                let input: tonic::Request<authorization::ListAccessPolicyVersionsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_access_policy_versions(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_2)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_2 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let outgoing_3 = itsm::CreateIncidentRequest {
            context: req.context.clone(),
            incident: Some(itsm::Incident {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .environment
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = acme_platform::transport::deadline(async move {
            let mut client = IncidentServiceClient::new(channel);
            let input: tonic::Request<itsm::CreateIncidentRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_incident(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_4 = pricing::BatchCreatePriceListsRequest {
            context: req.context.clone(),
            chunk: Some(pricing::batch_create_price_lists_request::Chunk::Record(
                pricing::PriceList {
                    audit: Some(common::AuditInfo {
                        created_by: reply_3
                            .incident
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
                        user_id: req
                            .environment
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = PriceListServiceClient::new(channel);
            let input: tonic::Request<pricing::BatchCreatePriceListsRequest> =
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
            let response = client.batch_create_price_lists(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = configuration::StreamEnvironmentChangesRequest {
            frame: Some(configuration::stream_environment_changes_request::Frame::Change(
                configuration::Environment {
                    owner: Some(identity::UserRef {
                        user_id: reply_3
                            .incident
                            .clone()
                            .unwrap_or_default()
                            .assignee
                            .clone()
                            .unwrap_or_default()
                            .user_id
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
        let reply_5 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamEnvironmentChangesRequest> =
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
        let mut response = configuration::SubmitEnvironmentResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: Vec<String> = reply_3.incident.clone().unwrap_or_default().tags.clone();
        if contribution != <Vec<String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .tags
                .extend(contribution);
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run ApproveEnvironment after validating the tenant and durably recording the command.
    async fn run_approve_environment(
        &self,
        req: configuration::ApproveEnvironmentRequest,
    ) -> Result<configuration::ApproveEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::StreamConfigSetMetricsRequest {
            context: req.context.clone(),
            filter: Some(configuration::stream_config_set_metrics_request::Filter {
                region: (match req.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            since: req
                .environment
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
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
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = ConfigSetReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamConfigSetMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_config_set_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::StreamConfigSetMetricsResponse::default();
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
        let outgoing_1 = configuration::RejectConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    unit: req
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
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::reject_config_change_request::Details {
                approver: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.approve_environment_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = configuration::AdvanceConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        updated_at: req
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        ..Default::default()
                    }),
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            updated_by: req
                                .environment
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
                            locality: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .locality
                            .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            user_id: req
                                .environment
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_2 = (self.approve_environment_2_port)(outgoing_2).await?;
        let mut response = configuration::ApproveEnvironmentResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run RejectEnvironment after validating the tenant and durably recording the command.
    async fn run_reject_environment(
        &self,
        req: configuration::RejectEnvironmentRequest,
    ) -> Result<configuration::RejectEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().code.clone(), "recorded audit field");
        let outgoing_0 = certificates::SimulateCertificateRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    country_code: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .country_code
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::simulate_certificate_request::Details {
                destination: Some(common::PostalAddress {
                    recipient: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .recipient
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::ActivateEnvironmentRequest {
            context: req.context.clone(),
            details: Some(configuration::activate_environment_request::Details {
                new_owner: Some(identity::UserRef {
                    user_id: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                notify_contact: Some(common::ContactInfo {
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
            environment_id: req.environment.clone().unwrap_or_default().environment_id.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CertificateCalculationServiceClient::new(channel_0);
                let input: tonic::Request<certificates::SimulateCertificateRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.simulate_certificate(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = EnvironmentLifecycleServiceClient::new(channel_1);
                let input: tonic::Request<configuration::ActivateEnvironmentRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.activate_environment(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = configuration::RejectConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    unit: req
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
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::reject_config_change_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_1
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .environment
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
        let outgoing_2 = acme_platform::ports::RequestBuilder::<configuration::RejectConfigChangeRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigChangeApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::RejectConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reject_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::RejectEnvironmentResponse::default();
        let contribution: String = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_0
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
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
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
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.environment.clone().unwrap_or_default().specs.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .specs
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run EscalateEnvironment after validating the tenant and durably recording the command.
    async fn run_escalate_environment(
        &self,
        req: configuration::EscalateEnvironmentRequest,
    ) -> Result<configuration::EscalateEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::LintConfigChangesRequest {
            annotations: req.annotations.clone(),
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
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
            details: Some(configuration::lint_config_changes_request::Details {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::LintConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = configuration::EscalateEnvironmentResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingEnvironments after validating the tenant and durably recording the command.
    async fn run_list_pending_environments(
        &self,
        req: configuration::ListPendingEnvironmentsRequest,
    ) -> Result<configuration::ListPendingEnvironmentsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPendingEnvironments", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingEnvironments", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::CancelConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                location: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::CancelConfigSetWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.cancel_config_set_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ListPendingEnvironmentsResponse::default();
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            if response.environments.is_empty() {
                response.environments.push(Default::default());
            }
            response.environments[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response.environments.is_empty() {
                response.environments.push(Default::default());
            }
            response.environments[0].audit.get_or_insert_with(Default::default).etag = contribution;
        }
        Ok(response)
    }
    /// Run DelegateEnvironmentApproval after validating the tenant and durably recording the command.
    async fn run_delegate_environment_approval(
        &self,
        req: configuration::DelegateEnvironmentApprovalRequest,
    ) -> Result<configuration::DelegateEnvironmentApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateEnvironmentApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateEnvironmentApproval", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().code.clone(), "recorded audit field");
        let outgoing_0 = featureflags::LintFeatureFlagsRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    etag: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
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
        let outgoing_0 = acme_platform::ports::RequestBuilder::<featureflags::LintFeatureFlagsRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::LintFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = DelegateEnvironmentApprovalDispatch1::from(req.clone());
        command_1.reason = (match reply_0.feature_flag.clone().unwrap_or_default().action.clone() {
            Some(featureflags::feature_flag::Action::Block(value)) => value,
            _ => Default::default(),
        })
        .reason
        .clone();
        command_1.details_period = reply_0.feature_flag.clone().unwrap_or_default().effective.clone();
        command_1.details_filters = reply_0.feature_flag.clone().unwrap_or_default().parameters.clone();
        let outgoing_1: certificates::SummarizeCertificatesRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::SummarizeCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::SuspendEnvironmentRequest {
            context: req.context.clone(),
            details: Some(configuration::suspend_environment_request::Details {
                new_owner: Some(identity::UserRef {
                    user_id: req
                        .environment
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
            environment_id: req.environment.clone().unwrap_or_default().environment_id.clone(),
            ..Default::default()
        };
        let outgoing_2 =
            acme_platform::codec::through_any(&outgoing_2, "acme.configuration.v1.SuspendEnvironmentRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = EnvironmentLifecycleServiceClient::new(channel);
            let input: tonic::Request<configuration::SuspendEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suspend_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::FetchEnvironmentReportRequest {
            context: req.context.clone(),
            environment_id: req.environment_id.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::FetchEnvironmentReportRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = EnvironmentReportingServiceClient::new(channel);
                let input: tonic::Request<configuration::FetchEnvironmentReportRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.fetch_environment_report(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_3)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_3 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = configuration::DelegateEnvironmentApprovalResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: f64 = reply_2
            .environment
            .clone()
            .unwrap_or_default()
            .dimensions
            .clone()
            .unwrap_or_default()
            .weight
            .clone();
        if contribution != <f64>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .dimensions
                .get_or_insert_with(Default::default)
                .weight = contribution;
        }
        let contribution: String = reply_2
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
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i64 = req.environment.clone().unwrap_or_default().capacity.clone();
        if contribution != <i64>::default() {
            response.environment.get_or_insert_with(Default::default).capacity = contribution;
        }
        Ok(response)
    }
    /// Run RecallEnvironment after validating the tenant and durably recording the command.
    async fn run_recall_environment(
        &self,
        req: configuration::RecallEnvironmentRequest,
    ) -> Result<configuration::RecallEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SubmitConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    unit: req
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
                    display_name: req
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
            details: Some(configuration::submit_config_change_request::Details {
                approver: req.details.clone().unwrap_or_default().approver.clone(),
                conditions: req.details.clone().unwrap_or_default().conditions.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = featureflags::AnnotateFeatureFlagVersionRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .environment
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
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeApprovalServiceClient::new(channel_0);
                let input: tonic::Request<configuration::SubmitConfigChangeRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.submit_config_change(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = FeatureFlagHistoryServiceClient::new(channel_1);
                let input: tonic::Request<featureflags::AnnotateFeatureFlagVersionRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.annotate_feature_flag_version(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = ratelimiting::EstimateQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::estimate_quota_bucket_request::Details {
                destination: Some(common::PostalAddress {
                    lines: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .lines
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = certificates::NormalizeCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: reply_2
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    etag: reply_1
                        .feature_flag
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                lines: vec![certificates::CertificateLine {
                    amount: (match reply_1.feature_flag.clone().unwrap_or_default().action.clone() {
                        Some(featureflags::feature_flag::Action::Discount(value)) => value,
                        _ => Default::default(),
                    })
                    .amount
                    .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_3 = (self.recall_environment_3_port)(outgoing_3).await?;
        let mut response = configuration::RecallEnvironmentResponse::default();
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
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
        let contribution: String = reply_3
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i64 = req.environment.clone().unwrap_or_default().capacity.clone();
        if contribution != <i64>::default() {
            response.environment.get_or_insert_with(Default::default).capacity = contribution;
        }
        Ok(response)
    }
    /// Run GetEnvironmentApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_environment_approval_status(
        &self,
        req: configuration::GetEnvironmentApprovalStatusRequest,
    ) -> Result<configuration::GetEnvironmentApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetEnvironmentApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetEnvironmentApprovalStatus", "running workflow");
        let mut response = configuration::GetEnvironmentApprovalStatusResponse::default();
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        Ok(response)
    }
    /// Run CommentEnvironmentApproval after validating the tenant and durably recording the command.
    async fn run_comment_environment_approval(
        &self,
        req: configuration::CommentEnvironmentApprovalRequest,
    ) -> Result<configuration::CommentEnvironmentApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CommentEnvironmentApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CommentEnvironmentApproval", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::PauseConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_by: req
                                .environment
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            updated_by: req
                                .environment
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_by
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.comment_environment_approval_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = configuration::ApproveConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                measurements: vec![configuration::ConfigChangeMeasurement {
                    unit: req
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
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::approve_config_change_request::Details {
                approver: Some(identity::UserRef {
                    email: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
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
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.comment_environment_approval_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = configuration::CommentEnvironmentApprovalResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.environment.clone().unwrap_or_default().specs.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .specs
                .extend(contribution);
        }
        Ok(response)
    }
}
/// The domain command prepared by DelegateEnvironmentApproval for SummarizeCertificates.
struct DelegateEnvironmentApprovalDispatch1 {
    context: Option<common::RequestContext>,
    reason: String,
    annotations: std::collections::BTreeMap<String, String>,
    details_period: Option<common::TimeWindow>,
    details_filters: std::collections::BTreeMap<String, String>,
}
impl From<configuration::DelegateEnvironmentApprovalRequest> for DelegateEnvironmentApprovalDispatch1 {
    fn from(req: configuration::DelegateEnvironmentApprovalRequest) -> Self {
        Self {
            context: req.context.clone(),
            reason: Default::default(),
            annotations: req.annotations.clone(),
            details_period: Default::default(),
            details_filters: Default::default(),
        }
    }
}
impl From<DelegateEnvironmentApprovalDispatch1> for certificates::SummarizeCertificatesRequest {
    fn from(value: DelegateEnvironmentApprovalDispatch1) -> Self {
        certificates::SummarizeCertificatesRequest {
            annotations: value.annotations,
            context: value.context,
            details: Some(certificates::summarize_certificates_request::Details {
                filters: value.details_filters,
                period: value.details_period,
                ..Default::default()
            }),
            reason: value.reason,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl EnvironmentApprovalService for EnvironmentApprovalEndpoint {
    /// Submits an environment.
    async fn submit_environment(
        &self,
        request: tonic::Request<configuration::SubmitEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::SubmitEnvironmentResponse>, tonic::Status> {
        self.run_submit_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves an environment.
    async fn approve_environment(
        &self,
        request: tonic::Request<configuration::ApproveEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::ApproveEnvironmentResponse>, tonic::Status> {
        self.run_approve_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects an environment.
    async fn reject_environment(
        &self,
        request: tonic::Request<configuration::RejectEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::RejectEnvironmentResponse>, tonic::Status> {
        self.run_reject_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates an environment.
    async fn escalate_environment(
        &self,
        request: tonic::Request<configuration::EscalateEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::EscalateEnvironmentResponse>, tonic::Status> {
        self.run_escalate_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists environments a page at a time.
    async fn list_pending_environments(
        &self,
        request: tonic::Request<configuration::ListPendingEnvironmentsRequest>,
    ) -> Result<tonic::Response<configuration::ListPendingEnvironmentsResponse>, tonic::Status> {
        self.run_list_pending_environments(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates an environment.
    async fn delegate_environment_approval(
        &self,
        request: tonic::Request<configuration::DelegateEnvironmentApprovalRequest>,
    ) -> Result<tonic::Response<configuration::DelegateEnvironmentApprovalResponse>, tonic::Status> {
        self.run_delegate_environment_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls an environment.
    async fn recall_environment(
        &self,
        request: tonic::Request<configuration::RecallEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::RecallEnvironmentResponse>, tonic::Status> {
        self.run_recall_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an environment.
    async fn get_environment_approval_status(
        &self,
        request: tonic::Request<configuration::GetEnvironmentApprovalStatusRequest>,
    ) -> Result<tonic::Response<configuration::GetEnvironmentApprovalStatusResponse>, tonic::Status> {
        self.run_get_environment_approval_status(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Comments an environment.
    async fn comment_environment_approval(
        &self,
        request: tonic::Request<configuration::CommentEnvironmentApprovalRequest>,
    ) -> Result<tonic::Response<configuration::CommentEnvironmentApprovalResponse>, tonic::Status> {
        self.run_comment_environment_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
