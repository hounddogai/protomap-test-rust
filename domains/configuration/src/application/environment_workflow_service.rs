use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::configuration::v1::config_set_approval_service_client::ConfigSetApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::configuration::v1::environment_approval_service_client::EnvironmentApprovalServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::configuration::v1::environment_workflow_service_server::EnvironmentWorkflowService;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_client::CryptoKeyQueryServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_client::QuotaBucketQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_client::ThrottleCalculationServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_service_client::SearchIndexServiceClient;
use acme_proto::acme::search::v1::search_index_validation_service_client::SearchIndexValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::subscriptions::v1 as subscriptions;
use acme_proto::acme::subscriptions::v1::subscription_service_client::SubscriptionServiceClient;

/// Runs the long-lived workflow that takes an environment from start to finish.
#[derive(Clone)]
pub struct EnvironmentWorkflowController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    advance_environment_workflow_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            keymanagement::ExplainKeyRingDecisionRequest,
            keymanagement::ExplainKeyRingDecisionResponse,
        >,
    >,
    list_environment_workflows_0_port: acme_platform::ports::Callback<
        keymanagement::CheckKeyRingEligibilityRequest,
        keymanagement::CheckKeyRingEligibilityResponse,
    >,
    signal_environment_workflow_0_port:
        acme_platform::ports::Callback<configuration::RecallConfigSetRequest, configuration::RecallConfigSetResponse>,
    signal_environment_workflow_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            secrets::CompareAccessGrantOptionsRequest,
            secrets::CompareAccessGrantOptionsResponse,
        >,
    >,
}
impl EnvironmentWorkflowController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::ExplainKeyRingDecisionRequest,
            keymanagement::ExplainKeyRingDecisionResponse,
        > = {
            let client = KeyRingValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ExplainKeyRingDecisionRequest| -> acme_platform::ports::RpcFuture<keymanagement::ExplainKeyRingDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_key_ring_decision(request).await?.into_inner())
})
})
        };
        let advance_environment_workflow_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::CheckKeyRingEligibilityRequest,
            keymanagement::CheckKeyRingEligibilityResponse,
        > = {
            let client = KeyRingValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::CheckKeyRingEligibilityRequest| -> acme_platform::ports::RpcFuture<keymanagement::CheckKeyRingEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_key_ring_eligibility(request).await?.into_inner())
})
})
        };
        let list_environment_workflows_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::RecallConfigSetRequest,
            configuration::RecallConfigSetResponse,
        > = {
            let client = ConfigSetApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::RecallConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::RecallConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recall_config_set(request).await?.into_inner())
})
})
        };
        let signal_environment_workflow_0_port = callback;
        let callback: acme_platform::ports::Callback<
            secrets::CompareAccessGrantOptionsRequest,
            secrets::CompareAccessGrantOptionsResponse,
        > = {
            let client = AccessGrantCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::CompareAccessGrantOptionsRequest| -> acme_platform::ports::RpcFuture<secrets::CompareAccessGrantOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_access_grant_options(request).await?.into_inner())
})
})
        };
        let signal_environment_workflow_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            advance_environment_workflow_2_port,
            list_environment_workflows_0_port,
            signal_environment_workflow_0_port,
            signal_environment_workflow_1_port,
        }
    }
    /// Run StartEnvironmentWorkflow after validating the tenant and durably recording the command.
    async fn run_start_environment_workflow(
        &self,
        req: configuration::StartEnvironmentWorkflowRequest,
    ) -> Result<configuration::StartEnvironmentWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartEnvironmentWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartEnvironmentWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.environment_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::RecordAccessPolicyVersionRequest {
            access_policy: Some(authorization::AccessPolicy {
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
            context: req.context.clone(),
            details: Some(authorization::record_access_policy_version_request::Details {
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessPolicyHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::RecordAccessPolicyVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.record_access_policy_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::FindQuotaBucketsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::find_quota_buckets_request::Filter {
                created_after: reply_0.outcome.clone().unwrap_or_default().effective_at.clone(),
                labels: req.environment.clone().unwrap_or_default().specs.clone(),
                owner_id: req.environment.clone().unwrap_or_default().environment_id.clone(),
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketQueryServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::FindQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::RejectEnvironmentRequest {
            context: req.context.clone(),
            details: Some(configuration::reject_environment_request::Details {
                approver: Some(identity::UserRef {
                    email: reply_0
                        .access_policy
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
            environment: Some(configuration::Environment {
                dimensions: Some(configuration::EnvironmentDimensions {
                    width: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .width
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::RejectEnvironmentRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = EnvironmentApprovalServiceClient::new(channel);
                let input: tonic::Request<configuration::RejectEnvironmentRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.reject_environment(input).await?;
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
        let outgoing_3 = configuration::PushEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
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
                    updated_by: reply_2
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = acme_platform::transport::deadline(async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::PushEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_4 = subscriptions::BatchCreateSubscriptionsRequest {
            context: req.context.clone(),
            chunk: Some(subscriptions::batch_create_subscriptions_request::Chunk::Header(
                subscriptions::batch_create_subscriptions_request::StreamHeader {
                    submitted_by: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify
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
            let mut client = SubscriptionServiceClient::new(channel);
            let input: tonic::Request<subscriptions::BatchCreateSubscriptionsRequest> =
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
            let response = client.batch_create_subscriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                    measurements: vec![featureflags::FlagEvaluationMeasurement {
                        unit: reply_2
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
                    subject: Some(featureflags::FlagRolloutRef {
                        display_name: reply_3
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_5 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
        let mut response = configuration::StartEnvironmentWorkflowResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .access_policy
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
            .environment
            .clone()
            .unwrap_or_default()
            .dimensions
            .clone()
            .unwrap_or_default()
            .unit
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .dimensions
                .get_or_insert_with(Default::default)
                .unit = contribution;
        }
        let contribution: String = reply_3
            .environment
            .clone()
            .unwrap_or_default()
            .components
            .clone()
            .get("primary")
            .cloned()
            .unwrap_or_default()
            .component_id
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .components
                .entry("primary".to_owned())
                .or_default()
                .component_id = contribution;
        }
        let contribution: String = reply_3.environment.clone().unwrap_or_default().environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.environment.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.environment.get_or_insert_with(Default::default).owner = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().code.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).code = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceEnvironmentWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_environment_workflow(
        &self,
        req: configuration::AdvanceEnvironmentWorkflowRequest,
    ) -> Result<configuration::AdvanceEnvironmentWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceEnvironmentWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceEnvironmentWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ResolveCryptoKeysRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::ScheduleConfigChangeExpiryRequest {
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CryptoKeyQueryServiceClient::new(channel_0);
                let input: tonic::Request<keymanagement::ResolveCryptoKeysRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_crypto_keys(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ConfigChangeLifecycleServiceClient::new(channel_1);
                let input: tonic::Request<configuration::ScheduleConfigChangeExpiryRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.schedule_config_change_expiry(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = keymanagement::ExplainKeyRingDecisionRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 =
            acme_platform::ports::execute(self.advance_environment_workflow_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = search::SearchSearchIndexesRequest {
            context: req.context.clone(),
            filter: Some(search::search_search_indexes_request::Filter {
                created_after: reply_2
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: reply_1.config_change.clone().unwrap_or_default().attributes.clone(),
                owner_id: req
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
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
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = SearchIndexServiceClient::new(channel);
            let input: tonic::Request<search::SearchSearchIndexesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_search_indexes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::AdvanceEnvironmentWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.config_change.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i32 = req.environment.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.environment.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run CancelEnvironmentWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_environment_workflow(
        &self,
        req: configuration::CancelEnvironmentWorkflowRequest,
    ) -> Result<configuration::CancelEnvironmentWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelEnvironmentWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelEnvironmentWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CompareThrottleOptionsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::compare_throttle_options_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .postal_code
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
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
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = search::SearchSearchIndexesRequest {
            context: req.context.clone(),
            filter: Some(search::search_search_indexes_request::Filter {
                created_after: req
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
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
                let mut client = ThrottleCalculationServiceClient::new(channel_0);
                let input: tonic::Request<ratelimiting::CompareThrottleOptionsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.compare_throttle_options(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = SearchIndexServiceClient::new(channel_1);
                let input: tonic::Request<search::SearchSearchIndexesRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.search_search_indexes(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = configuration::CancelEnvironmentWorkflowResponse::default();
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
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i32 = req.environment.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.environment.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run RetryEnvironmentWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_environment_workflow(
        &self,
        req: configuration::RetryEnvironmentWorkflowRequest,
    ) -> Result<configuration::RetryEnvironmentWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryEnvironmentWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryEnvironmentWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::FindQuotaBucketsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::find_quota_buckets_request::Filter {
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = search::ExplainSearchIndexDecisionRequest {
            context: req.context.clone(),
            details: Some(search::explain_search_index_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
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
            search_index: Some(search::SearchIndex {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    search::SearchIndexComponent {
                        component_id: req
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .components
                            .clone()
                            .get("primary")
                            .cloned()
                            .unwrap_or_default()
                            .component_id
                            .clone(),
                        description: req
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .components
                            .clone()
                            .get("primary")
                            .cloned()
                            .unwrap_or_default()
                            .description
                            .clone(),
                        ..Default::default()
                    },
                )]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = QuotaBucketQueryServiceClient::new(channel_0);
                let input: tonic::Request<ratelimiting::FindQuotaBucketsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.find_quota_buckets(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = SearchIndexValidationServiceClient::new(channel_1);
                let input: tonic::Request<search::ExplainSearchIndexDecisionRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_search_index_decision(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = configuration::RetryEnvironmentWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: f64 = reply_1
            .search_index
            .clone()
            .unwrap_or_default()
            .dimensions
            .clone()
            .unwrap_or_default()
            .length
            .clone();
        if contribution != <f64>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .dimensions
                .get_or_insert_with(Default::default)
                .length = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
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
    /// Run PauseEnvironmentWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_environment_workflow(
        &self,
        req: configuration::PauseEnvironmentWorkflowRequest,
    ) -> Result<configuration::PauseEnvironmentWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PauseEnvironmentWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseEnvironmentWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::StartConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            recipient: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
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
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<configuration::StartConfigSetWorkflowRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::StartConfigSetWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.start_config_set_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = PauseEnvironmentWorkflowDispatch1::from(req.clone());
        command_1.search_index_audit_created_at = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        let outgoing_1: search::LintSearchIndexesRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SearchIndexValidationServiceClient::new(channel);
            let input: tonic::Request<search::LintSearchIndexesRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_search_indexes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::ListPendingEnvironmentsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_pending_environments_request::Filter {
                created_after: reply_0
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: (match reply_1.search_index.clone().unwrap_or_default().locator.clone() {
                    Some(search::search_index::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = EnvironmentApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::ListPendingEnvironmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_environments(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = search::ScoreSearchIndexRequest {
            context: req.context.clone(),
            details: Some(search::score_search_index_request::Details {
                destination: Some(common::PostalAddress {
                    region: (match reply_1.search_index.clone().unwrap_or_default().locator.clone() {
                        Some(search::search_index::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .region
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            search_index: Some(search::SearchIndex {
                audit: Some(common::AuditInfo {
                    created_by: reply_1
                        .search_index
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                dimensions: Some(search::SearchIndexDimensions {
                    unit: reply_1
                        .search_index
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .unit
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
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
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<search::ScoreSearchIndexRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = SearchIndexValidationServiceClient::new(channel);
                let input: tonic::Request<search::ScoreSearchIndexRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.score_search_index(input).await?;
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
        let mut response = configuration::PauseEnvironmentWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.search_index.clone().unwrap_or_default().specs.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .specs
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_3.search_index.clone().unwrap_or_default().category_path.clone();
        if contribution != <Vec<String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .category_path
                .extend(contribution);
        }
        let contribution: String = reply_3
            .search_index
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().code.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).code = contribution;
        }
        Ok(response)
    }
    /// Run ResumeEnvironmentWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_environment_workflow(
        &self,
        req: configuration::ResumeEnvironmentWorkflowRequest,
    ) -> Result<configuration::ResumeEnvironmentWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeEnvironmentWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeEnvironmentWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SignalConfigChangeWorkflowRequest {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = ConfigChangeWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::SignalConfigChangeWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_config_change_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = configuration::AdvanceConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
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
                    manager: Some(identity::UserRef {
                        email: req
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::AdvanceConfigSetWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_config_set_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::RejectConfigChangeRequest {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::reject_config_change_request::Details {
                approver: Some(identity::UserRef {
                    user_id: reply_1
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
        let mut response = configuration::ResumeEnvironmentWorkflowResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1
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
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_2.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().code.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).code = contribution;
        }
        Ok(response)
    }
    /// Run ListEnvironmentWorkflows after validating the tenant and durably recording the command.
    async fn run_list_environment_workflows(
        &self,
        req: configuration::ListEnvironmentWorkflowsRequest,
    ) -> Result<configuration::ListEnvironmentWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListEnvironmentWorkflows", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListEnvironmentWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::CheckKeyRingEligibilityRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                location: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.list_environment_workflows_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = configuration::ListEnvironmentWorkflowsResponse::default();
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response.environments.is_empty() {
                response.environments.push(Default::default());
            }
            response.environments[0]
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run SignalEnvironmentWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_environment_workflow(
        &self,
        req: configuration::SignalEnvironmentWorkflowRequest,
    ) -> Result<configuration::SignalEnvironmentWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalEnvironmentWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalEnvironmentWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::RecallConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = (self.signal_environment_workflow_0_port)(outgoing_0).await?;
        let outgoing_1 = secrets::CompareAccessGrantOptionsRequest {
            access_grant: Some(secrets::AccessGrant {
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.signal_environment_workflow_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = certificates::ExplainCertificateAuthorityDecisionRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        location: Some(common::PostalAddress {
                            country_code: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .country_code
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateAuthorityValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ExplainCertificateAuthorityDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_certificate_authority_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::SubscribeFeatureFlagUpdatesRequest {
            context: req.context.clone(),
            details: Some(featureflags::subscribe_feature_flag_updates_request::Details {
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
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_3 =
            acme_platform::ports::RequestBuilder::<featureflags::SubscribeFeatureFlagUpdatesRequest>::default()
                .map(|draft| *draft = outgoing_3)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FeatureFlagNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::SubscribeFeatureFlagUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.subscribe_feature_flag_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::SignalEnvironmentWorkflowResponse::default();
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
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_3.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().code.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).code = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by PauseEnvironmentWorkflow for LintSearchIndexes.
struct PauseEnvironmentWorkflowDispatch1 {
    context: Option<common::RequestContext>,
    details_contact_email: String,
    search_index_audit_created_at: Option<prost_types::Timestamp>,
}
impl From<configuration::PauseEnvironmentWorkflowRequest> for PauseEnvironmentWorkflowDispatch1 {
    fn from(req: configuration::PauseEnvironmentWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_contact_email: req
                .details
                .clone()
                .unwrap_or_default()
                .notify
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            search_index_audit_created_at: Default::default(),
        }
    }
}
impl From<PauseEnvironmentWorkflowDispatch1> for search::LintSearchIndexesRequest {
    fn from(value: PauseEnvironmentWorkflowDispatch1) -> Self {
        search::LintSearchIndexesRequest {
            context: value.context,
            details: Some(search::lint_search_indexes_request::Details {
                contact: Some(common::ContactInfo {
                    email: value.details_contact_email,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            search_index: Some(search::SearchIndex {
                audit: Some(common::AuditInfo {
                    created_at: value.search_index_audit_created_at,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl EnvironmentWorkflowService for EnvironmentWorkflowController {
    /// Starts an environment.
    async fn start_environment_workflow(
        &self,
        request: tonic::Request<configuration::StartEnvironmentWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::StartEnvironmentWorkflowResponse>, tonic::Status> {
        self.run_start_environment_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances an environment.
    async fn advance_environment_workflow(
        &self,
        request: tonic::Request<configuration::AdvanceEnvironmentWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::AdvanceEnvironmentWorkflowResponse>, tonic::Status> {
        self.run_advance_environment_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels an environment.
    async fn cancel_environment_workflow(
        &self,
        request: tonic::Request<configuration::CancelEnvironmentWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::CancelEnvironmentWorkflowResponse>, tonic::Status> {
        self.run_cancel_environment_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys an environment.
    async fn retry_environment_workflow(
        &self,
        request: tonic::Request<configuration::RetryEnvironmentWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::RetryEnvironmentWorkflowResponse>, tonic::Status> {
        self.run_retry_environment_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses an environment.
    async fn pause_environment_workflow(
        &self,
        request: tonic::Request<configuration::PauseEnvironmentWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::PauseEnvironmentWorkflowResponse>, tonic::Status> {
        self.run_pause_environment_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes an environment.
    async fn resume_environment_workflow(
        &self,
        request: tonic::Request<configuration::ResumeEnvironmentWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::ResumeEnvironmentWorkflowResponse>, tonic::Status> {
        self.run_resume_environment_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists environments a page at a time.
    async fn list_environment_workflows(
        &self,
        request: tonic::Request<configuration::ListEnvironmentWorkflowsRequest>,
    ) -> Result<tonic::Response<configuration::ListEnvironmentWorkflowsResponse>, tonic::Status> {
        self.run_list_environment_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals an environment.
    async fn signal_environment_workflow(
        &self,
        request: tonic::Request<configuration::SignalEnvironmentWorkflowRequest>,
    ) -> Result<tonic::Response<configuration::SignalEnvironmentWorkflowResponse>, tonic::Status> {
        self.run_signal_environment_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
