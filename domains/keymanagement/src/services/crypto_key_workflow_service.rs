use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_validation_service_client::RoleBindingValidationServiceClient;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::broadcasts::v1::broadcast_service_client::BroadcastServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::configuration::v1::environment_workflow_service_client::EnvironmentWorkflowServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_client::CryptoKeyQueryServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_service_client::CryptoKeyServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_workflow_service_server::CryptoKeyWorkflowService;
use acme_proto::acme::keymanagement::v1::key_ring_approval_service_client::KeyRingApprovalServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_client::KeyRingReportingServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_reporting_service_client::KeyRotationReportingServiceClient;
use acme_proto::acme::onboarding::v1 as onboarding;
use acme_proto::acme::onboarding::v1::onboarding_plan_calculation_service_client::OnboardingPlanCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;

/// Runs the long-lived workflow that takes a crypto key from start to finish.
#[derive(Clone)]
pub struct CryptoKeyWorkflowApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    advance_crypto_key_workflow_2_port: acme_platform::ports::Callback<
        keymanagement::CountKeyRingTotalsRequest,
        keymanagement::CountKeyRingTotalsResponse,
    >,
    signal_crypto_key_workflow_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            keymanagement::ShareKeyRingReportRequest,
            keymanagement::ShareKeyRingReportResponse,
        >,
    >,
}
impl CryptoKeyWorkflowApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::CountKeyRingTotalsRequest,
            keymanagement::CountKeyRingTotalsResponse,
        > = {
            let client = KeyRingReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::CountKeyRingTotalsRequest| -> acme_platform::ports::RpcFuture<keymanagement::CountKeyRingTotalsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_key_ring_totals(request).await?.into_inner())
})
})
        };
        let advance_crypto_key_workflow_2_port = callback;
        let callback: acme_platform::ports::Callback<
            keymanagement::ShareKeyRingReportRequest,
            keymanagement::ShareKeyRingReportResponse,
        > = {
            let client = KeyRingReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ShareKeyRingReportRequest| -> acme_platform::ports::RpcFuture<keymanagement::ShareKeyRingReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_key_ring_report(request).await?.into_inner())
})
})
        };
        let signal_crypto_key_workflow_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            advance_crypto_key_workflow_2_port,
            signal_crypto_key_workflow_0_port,
        }
    }
    /// Run StartCryptoKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_start_crypto_key_workflow(
        &self,
        req: keymanagement::StartCryptoKeyWorkflowRequest,
    ) -> Result<keymanagement::StartCryptoKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StartCryptoKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartCryptoKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::AdvanceEnvironmentWorkflowRequest {
            context: req.context.clone(),
            details: Some(configuration::advance_environment_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    preferred_channel: req
                        .crypto_key
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
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<configuration::AdvanceEnvironmentWorkflowRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::AdvanceEnvironmentWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_environment_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = StartCryptoKeyWorkflowDispatch1::from(req.clone());
        command_1.details_destination_locality = (match reply_0.environment.clone().unwrap_or_default().locator.clone()
        {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .locality
        .clone();
        let outgoing_1: authorization::ScoreRoleBindingRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RoleBindingValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::ScoreRoleBindingRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_role_binding(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = keymanagement::StreamKeyRingMetricsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::stream_key_ring_metrics_request::Filter {
                region: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
                email: req
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                name: req
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_2 = async move {
            let mut client = KeyRingReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::StreamKeyRingMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_key_ring_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = keymanagement::StreamKeyRingMetricsResponse::default();
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
        let outgoing_3 = broadcasts::BatchCreateBroadcastsRequest {
            context: req.context.clone(),
            chunk: Some(broadcasts::batch_create_broadcasts_request::Chunk::Record(
                broadcasts::Broadcast {
                    audit: Some(common::AuditInfo {
                        created_at: reply_0
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
                    organizer: Some(identity::UserRef {
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = BroadcastServiceClient::new(channel);
            let input: tonic::Request<broadcasts::BatchCreateBroadcastsRequest> =
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
            let response = client.batch_create_broadcasts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = configuration::StreamEnvironmentChangesRequest {
            frame: Some(configuration::stream_environment_changes_request::Frame::Change(
                configuration::Environment {
                    audit: Some(common::AuditInfo {
                        created_by: req
                            .crypto_key
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        ..Default::default()
                    }),
                    dimensions: Some(configuration::EnvironmentDimensions {
                        width: reply_0
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
                    owner: Some(identity::UserRef {
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
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamEnvironmentChangesRequest> =
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
        let mut response = keymanagement::StartCryptoKeyWorkflowResponse::default();
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
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: i32 = req.crypto_key.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.crypto_key.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceCryptoKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_crypto_key_workflow(
        &self,
        req: keymanagement::AdvanceCryptoKeyWorkflowRequest,
    ) -> Result<keymanagement::AdvanceCryptoKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceCryptoKeyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceCryptoKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().holder.clone().unwrap_or_default().display_name.clone(), "recorded audit field");
        let outgoing_0 = audit::SuggestAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::suggest_audit_events_request::Filter {
                created_after: req
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::ResolveCryptoKeysRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = AuditEventQueryServiceClient::new(channel_0);
                let input: tonic::Request<audit::SuggestAuditEventsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.suggest_audit_events(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CryptoKeyQueryServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::ResolveCryptoKeysRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_crypto_keys(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = keymanagement::CountKeyRingTotalsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::count_key_ring_totals_request::Filter {
                created_after: req
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: req
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .key_ring_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = (self.advance_crypto_key_workflow_2_port)(outgoing_2).await?;
        let mut response = keymanagement::AdvanceCryptoKeyWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.crypto_key.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
    /// Run CancelCryptoKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_crypto_key_workflow(
        &self,
        req: keymanagement::CancelCryptoKeyWorkflowRequest,
    ) -> Result<keymanagement::CancelCryptoKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelCryptoKeyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelCryptoKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let jobs_0 = req
            .crypto_key
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = keymanagement::RejectCryptoKeyRequest {
                        context: req.context.clone(),
                        crypto_key: Some(keymanagement::CryptoKey {
                            limits: vec![keymanagement::CryptoKeyLimit {
                                amount: item.amount.clone(),
                                period: item.period.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CryptoKeyApprovalServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::RejectCryptoKeyRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.reject_crypto_key(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = ratelimiting::PushQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::push_quota_bucket_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    phone: reply_0
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .crypto_key
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
        let reply_1 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = keymanagement::ShareKeyRingReportRequest {
            context: req.context.clone(),
            details: Some(keymanagement::share_key_ring_report_request::Details {
                recipient: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let outgoing_2 = acme_platform::ports::RequestBuilder::<keymanagement::ShareKeyRingReportRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = KeyRingReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ShareKeyRingReportRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_key_ring_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::CancelCryptoKeyWorkflowResponse::default();
        let contribution: Option<common::ContactInfo> =
            reply_1.quota_bucket.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).contact = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<common::Money> = req.crypto_key.clone().unwrap_or_default().balance.clone();
        if contribution != <Option<common::Money>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).balance = contribution;
        }
        Ok(response)
    }
    /// Run RetryCryptoKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_crypto_key_workflow(
        &self,
        req: keymanagement::RetryCryptoKeyWorkflowRequest,
    ) -> Result<keymanagement::RetryCryptoKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryCryptoKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryCryptoKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::RecallKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_at: req
                                .crypto_key
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
                            created_by: req
                                .crypto_key
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            etag: req
                                .crypto_key
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::BatchGetCryptoKeysRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = KeyRingApprovalServiceClient::new(channel_0);
                let input: tonic::Request<keymanagement::RecallKeyRingRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.recall_key_ring(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CryptoKeyServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::BatchGetCryptoKeysRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.batch_get_crypto_keys(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = keymanagement::RetryCryptoKeyWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.crypto_key.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PauseCryptoKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_crypto_key_workflow(
        &self,
        req: keymanagement::PauseCryptoKeyWorkflowRequest,
    ) -> Result<keymanagement::PauseCryptoKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PauseCryptoKeyWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseCryptoKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = onboarding::SimulateOnboardingPlanRequest {
            context: req.context.clone(),
            onboarding_plan: Some(onboarding::OnboardingPlan {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                settlement: Some(onboarding::onboarding_plan::Settlement::Bank(
                    onboarding::OnboardingPlanBankSettlement {
                        bic: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .bic
                        .clone(),
                        iban: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .iban
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::ShareKeyRotationReportRequest {
            context: req.context.clone(),
            details: Some(keymanagement::share_key_rotation_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .contact
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
                    phone: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .crypto_key
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = OnboardingPlanCalculationServiceClient::new(channel_0);
                let input: tonic::Request<onboarding::SimulateOnboardingPlanRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.simulate_onboarding_plan(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = KeyRotationReportingServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::ShareKeyRotationReportRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.share_key_rotation_report(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = keymanagement::PauseCryptoKeyWorkflowResponse::default();
        let contribution: String = reply_0.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
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
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_1
            .key_rotation
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.crypto_key.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
    /// Run ResumeCryptoKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_crypto_key_workflow(
        &self,
        req: keymanagement::ResumeCryptoKeyWorkflowRequest,
    ) -> Result<keymanagement::ResumeCryptoKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeCryptoKeyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeCryptoKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::ValidateRoleBindingRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .crypto_key
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::FindCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::find_crypto_keys_request::Filter {
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = RoleBindingValidationServiceClient::new(channel_0);
                let input: tonic::Request<authorization::ValidateRoleBindingRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.validate_role_binding(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CryptoKeyQueryServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::FindCryptoKeysRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.find_crypto_keys(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = keymanagement::ResumeCryptoKeyWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .role_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: i32 = req.crypto_key.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.crypto_key.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run InspectCryptoKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_crypto_key_workflow(
        &self,
        req: keymanagement::InspectCryptoKeyWorkflowRequest,
    ) -> Result<keymanagement::InspectCryptoKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectCryptoKeyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectCryptoKeyWorkflow", "running workflow");
        let outgoing_0 = configuration::ResolveEnvironmentConflictRequest {
            context: req.context.clone(),
            details: Some(configuration::resolve_environment_conflict_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            environment: Some(configuration::Environment {
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    configuration::EnvironmentComponent {
                        component_id: req.crypto_key_id.clone(),
                        ..Default::default()
                    },
                )]),
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
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::ResolveEnvironmentConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_environment_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::InspectCryptoKeyWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        Ok(response)
    }
    /// Run ListCryptoKeyWorkflows after validating the tenant and durably recording the command.
    async fn run_list_crypto_key_workflows(
        &self,
        req: keymanagement::ListCryptoKeyWorkflowsRequest,
    ) -> Result<keymanagement::ListCryptoKeyWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListCryptoKeyWorkflows", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListCryptoKeyWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::SubmitKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        location: Some(common::PostalAddress {
                            region: req.filter.clone().unwrap_or_default().region.clone(),
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
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.keymanagement.v1.SubmitKeyRingRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingApprovalServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SubmitKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.submit_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::ListPendingCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::list_pending_crypto_keys_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                region: reply_0
                    .key_ring
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::ListPendingCryptoKeysRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CryptoKeyApprovalServiceClient::new(channel);
                let input: tonic::Request<keymanagement::ListPendingCryptoKeysRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_pending_crypto_keys(input).await?;
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
        let mut response = keymanagement::ListCryptoKeyWorkflowsResponse::default();
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
            if response.crypto_keys.is_empty() {
                response.crypto_keys.push(Default::default());
            }
            response.crypto_keys[0]
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
    /// Run SignalCryptoKeyWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_crypto_key_workflow(
        &self,
        req: keymanagement::SignalCryptoKeyWorkflowRequest,
    ) -> Result<keymanagement::SignalCryptoKeyWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalCryptoKeyWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalCryptoKeyWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ShareKeyRingReportRequest {
            context: req.context.clone(),
            details: Some(keymanagement::share_key_ring_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    phone: req
                        .crypto_key
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
            key_ring_id: req
                .crypto_key
                .clone()
                .unwrap_or_default()
                .holder
                .clone()
                .unwrap_or_default()
                .key_ring_id
                .clone(),
            ..Default::default()
        };
        let reply_0 =
            self.signal_crypto_key_workflow_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = keymanagement::FindCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::find_crypto_keys_request::Filter {
                created_after: req
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: (req.crypto_key.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                region: reply_0
                    .key_ring
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
        let reply_1 = async move {
            let mut client = CryptoKeyQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::FindCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::SignalCryptoKeyWorkflowResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.crypto_key.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
}
/// The domain command prepared by StartCryptoKeyWorkflow for ScoreRoleBinding.
struct StartCryptoKeyWorkflowDispatch1 {
    context: Option<common::RequestContext>,
    role_binding_bank_iban: String,
    details_destination_locality: String,
}
impl From<keymanagement::StartCryptoKeyWorkflowRequest> for StartCryptoKeyWorkflowDispatch1 {
    fn from(req: keymanagement::StartCryptoKeyWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            role_binding_bank_iban: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                Some(keymanagement::crypto_key::Funding::Bank(value)) => value,
                _ => Default::default(),
            })
            .iban
            .clone(),
            details_destination_locality: Default::default(),
        }
    }
}
impl From<StartCryptoKeyWorkflowDispatch1> for authorization::ScoreRoleBindingRequest {
    fn from(value: StartCryptoKeyWorkflowDispatch1) -> Self {
        authorization::ScoreRoleBindingRequest {
            context: value.context,
            details: Some(authorization::score_role_binding_request::Details {
                destination: Some(common::PostalAddress {
                    locality: value.details_destination_locality,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            role_binding: Some(authorization::RoleBinding {
                settlement: Some(authorization::role_binding::Settlement::Bank(
                    authorization::RoleBindingBankSettlement {
                        iban: value.role_binding_bank_iban,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CryptoKeyWorkflowService for CryptoKeyWorkflowApi {
    /// Starts a crypto key.
    async fn start_crypto_key_workflow(
        &self,
        request: tonic::Request<keymanagement::StartCryptoKeyWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::StartCryptoKeyWorkflowResponse>, tonic::Status> {
        self.run_start_crypto_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances a crypto key.
    async fn advance_crypto_key_workflow(
        &self,
        request: tonic::Request<keymanagement::AdvanceCryptoKeyWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::AdvanceCryptoKeyWorkflowResponse>, tonic::Status> {
        self.run_advance_crypto_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a crypto key.
    async fn cancel_crypto_key_workflow(
        &self,
        request: tonic::Request<keymanagement::CancelCryptoKeyWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::CancelCryptoKeyWorkflowResponse>, tonic::Status> {
        self.run_cancel_crypto_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a crypto key.
    async fn retry_crypto_key_workflow(
        &self,
        request: tonic::Request<keymanagement::RetryCryptoKeyWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::RetryCryptoKeyWorkflowResponse>, tonic::Status> {
        self.run_retry_crypto_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses a crypto key.
    async fn pause_crypto_key_workflow(
        &self,
        request: tonic::Request<keymanagement::PauseCryptoKeyWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::PauseCryptoKeyWorkflowResponse>, tonic::Status> {
        self.run_pause_crypto_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes a crypto key.
    async fn resume_crypto_key_workflow(
        &self,
        request: tonic::Request<keymanagement::ResumeCryptoKeyWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::ResumeCryptoKeyWorkflowResponse>, tonic::Status> {
        self.run_resume_crypto_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a crypto key.
    async fn inspect_crypto_key_workflow(
        &self,
        request: tonic::Request<keymanagement::InspectCryptoKeyWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::InspectCryptoKeyWorkflowResponse>, tonic::Status> {
        self.run_inspect_crypto_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists crypto keys a page at a time.
    async fn list_crypto_key_workflows(
        &self,
        request: tonic::Request<keymanagement::ListCryptoKeyWorkflowsRequest>,
    ) -> Result<tonic::Response<keymanagement::ListCryptoKeyWorkflowsResponse>, tonic::Status> {
        self.run_list_crypto_key_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals a crypto key.
    async fn signal_crypto_key_workflow(
        &self,
        request: tonic::Request<keymanagement::SignalCryptoKeyWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::SignalCryptoKeyWorkflowResponse>, tonic::Status> {
        self.run_signal_crypto_key_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
