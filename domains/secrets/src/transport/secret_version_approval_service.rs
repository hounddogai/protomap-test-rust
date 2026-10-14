use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fuel::v1 as fuel;
use acme_proto::acme::fuel::v1::fuel_card_query_service_client::FuelCardQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_approval_service_client::RateLimitApprovalServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::access_grant_validation_service_client::AccessGrantValidationServiceClient;
use acme_proto::acme::secrets::v1::secret_approval_service_client::SecretApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_service_client::SecretServiceClient;
use acme_proto::acme::secrets::v1::secret_version_approval_service_server::SecretVersionApprovalService;
use acme_proto::acme::secrets::v1::secret_version_query_service_client::SecretVersionQueryServiceClient;
use acme_proto::acme::secrets::v1::secret_version_validation_service_client::SecretVersionValidationServiceClient;

/// Routes secret versions through approval, with escalation and delegation.
#[derive(Clone)]
pub struct SecretVersionApprovalHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    reject_secret_version_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::DelegateRateLimitApprovalRequest,
            ratelimiting::DelegateRateLimitApprovalResponse,
        >,
    >,
    reject_secret_version_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                secrets::DelegateSecretApprovalRequest,
                secrets::DelegateSecretApprovalResponse,
            >,
    >,
    submit_secret_version_0_port: acme_platform::ports::Callback<
        keymanagement::ScheduleKeyRotationExpiryRequest,
        keymanagement::ScheduleKeyRotationExpiryResponse,
    >,
    submit_secret_version_2_port:
        acme_platform::ports::Callback<secrets::GetSecretVersionRulesRequest, secrets::GetSecretVersionRulesResponse>,
}
impl SecretVersionApprovalHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::ScheduleKeyRotationExpiryRequest,
            keymanagement::ScheduleKeyRotationExpiryResponse,
        > = {
            let client = KeyRotationLifecycleServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ScheduleKeyRotationExpiryRequest| -> acme_platform::ports::RpcFuture<keymanagement::ScheduleKeyRotationExpiryResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.schedule_key_rotation_expiry(request).await?.into_inner())
})
})
        };
        let submit_secret_version_0_port = callback;
        let callback: acme_platform::ports::Callback<
            secrets::GetSecretVersionRulesRequest,
            secrets::GetSecretVersionRulesResponse,
        > = {
            let client = SecretVersionValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::GetSecretVersionRulesRequest| -> acme_platform::ports::RpcFuture<secrets::GetSecretVersionRulesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.get_secret_version_rules(request).await?.into_inner())
})
})
        };
        let submit_secret_version_2_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::DelegateRateLimitApprovalRequest,
            ratelimiting::DelegateRateLimitApprovalResponse,
        > = {
            let client = RateLimitApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::DelegateRateLimitApprovalRequest| -> acme_platform::ports::RpcFuture<ratelimiting::DelegateRateLimitApprovalResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.delegate_rate_limit_approval(request).await?.into_inner())
})
})
        };
        let reject_secret_version_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            secrets::DelegateSecretApprovalRequest,
            secrets::DelegateSecretApprovalResponse,
        > = {
            let client = SecretApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::DelegateSecretApprovalRequest| -> acme_platform::ports::RpcFuture<secrets::DelegateSecretApprovalResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.delegate_secret_approval(request).await?.into_inner())
})
})
        };
        let reject_secret_version_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            reject_secret_version_1_port,
            reject_secret_version_2_port,
            submit_secret_version_0_port,
            submit_secret_version_2_port,
        }
    }
    /// Run SubmitSecretVersion after validating the tenant and durably recording the command.
    async fn run_submit_secret_version(
        &self,
        req: secrets::SubmitSecretVersionRequest,
    ) -> Result<secrets::SubmitSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitSecretVersion", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ScheduleKeyRotationExpiryRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                organizer: Some(identity::UserRef {
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
            }),
            ..Default::default()
        };
        let reply_0 = (self.submit_secret_version_0_port)(outgoing_0).await?;
        let outgoing_1 = secrets::WatchSecretsRequest {
            context: req.context.clone(),
            filter: Some(secrets::watch_secrets_request::Filter {
                region: reply_0
                    .key_rotation
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
            let mut client = SecretServiceClient::new(channel);
            let input: tonic::Request<secrets::WatchSecretsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_secrets(input).await?;
            let mut frames = response.into_inner();
            let mut last = secrets::WatchSecretsResponse::default();
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
        let outgoing_2 = secrets::GetSecretVersionRulesRequest {
            context: req.context.clone(),
            secret_version_id: req.secret_version_id.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.submit_secret_version_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let outgoing_3 = secrets::ExplainAccessGrantCalculationRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .key_rotation
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
                    display_name: reply_2
                        .secret_version
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    user_id: reply_0
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
        let outgoing_3 =
            acme_platform::ports::RequestBuilder::<secrets::ExplainAccessGrantCalculationRequest>::default()
                .map(|draft| *draft = outgoing_3)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::ExplainAccessGrantCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_access_grant_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::SubmitSecretVersionResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3.access_grant.clone().unwrap_or_default().access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: String = req.secret_version.clone().unwrap_or_default().source.clone();
        if contribution != <String>::default() {
            response.secret_version.get_or_insert_with(Default::default).source = contribution;
        }
        Ok(response)
    }
    /// Run ApproveSecretVersion after validating the tenant and durably recording the command.
    async fn run_approve_secret_version(
        &self,
        req: secrets::ApproveSecretVersionRequest,
    ) -> Result<secrets::ApproveSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveSecretVersion", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let jobs_0 = req
            .secret_version
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = ratelimiting::SignalRateLimitWorkflowRequest {
                        context: req.context.clone(),
                        rate_limit: Some(ratelimiting::RateLimit {
                            audit: Some(common::AuditInfo {
                                created_at: item.observed_at.clone(),
                                ..Default::default()
                            }),
                            owner: Some(identity::UserRef {
                                display_name: req
                                    .secret_version
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
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = RateLimitWorkflowServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::SignalRateLimitWorkflowRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.signal_rate_limit_workflow(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = secrets::ApproveSecretVersionResponse::default();
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: String = req.secret_version.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run RejectSecretVersion after validating the tenant and durably recording the command.
    async fn run_reject_secret_version(
        &self,
        req: secrets::RejectSecretVersionRequest,
    ) -> Result<secrets::RejectSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectSecretVersion", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::BatchCreateSecretsRequest {
            context: req.context.clone(),
            chunk: Some(secrets::batch_create_secrets_request::Chunk::Header(
                secrets::batch_create_secrets_request::StreamHeader {
                    source: req.secret_version.clone().unwrap_or_default().source.clone(),
                    submitted_by: Some(common::ContactInfo {
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
                },
            )),
            sequence: req.secret_version.clone().unwrap_or_default().sequence.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = SecretServiceClient::new(channel);
            let input: tonic::Request<secrets::BatchCreateSecretsRequest> =
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
            let response = client.batch_create_secrets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::DelegateRateLimitApprovalRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::delegate_rate_limit_approval_request::Details {
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
        let reply_1 = acme_platform::ports::execute(self.reject_secret_version_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = secrets::DelegateSecretApprovalRequest {
            context: req.context.clone(),
            secret: Some(secrets::Secret {
                contact: Some(common::ContactInfo {
                    email: reply_1
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(identity::UserRef {
                    user_id: reply_1
                        .rate_limit
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
        let reply_2 = acme_platform::ports::execute(self.reject_secret_version_2_port.as_ref(), outgoing_2).await?;
        let mut response = secrets::RejectSecretVersionResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
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
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: String = req.secret_version.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.secret_version.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run EscalateSecretVersion after validating the tenant and durably recording the command.
    async fn run_escalate_secret_version(
        &self,
        req: secrets::EscalateSecretVersionRequest,
    ) -> Result<secrets::EscalateSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateSecretVersion", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = fuel::SuggestFuelCardsRequest {
            context: req.context.clone(),
            filter: Some(fuel::suggest_fuel_cards_request::Filter {
                created_after: req.secret_version.clone().unwrap_or_default().occurred_at.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<fuel::SuggestFuelCardsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = FuelCardQueryServiceClient::new(channel);
                let input: tonic::Request<fuel::SuggestFuelCardsRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.suggest_fuel_cards(input).await?;
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
        let mut response = secrets::EscalateSecretVersionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.secret_version.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ListPendingSecretVersions after validating the tenant and durably recording the command.
    async fn run_list_pending_secret_versions(
        &self,
        req: secrets::ListPendingSecretVersionsRequest,
    ) -> Result<secrets::ListPendingSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPendingSecretVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingSecretVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamQuotaBucketChangesRequest {
            frame: Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(
                ratelimiting::QuotaBucket {
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
                    funding: Some(ratelimiting::quota_bucket::Funding::ExternalAccountId(
                        req.filter.clone().unwrap_or_default().owner_id.clone(),
                    )),
                    labels: req.filter.clone().unwrap_or_default().labels.clone(),
                    opened_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketChangesRequest> =
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
            let response = client.stream_quota_bucket_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamQuotaBucketChangesResponse::default();
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
        let response = secrets::ListPendingSecretVersionsResponse::default();
        Ok(response)
    }
    /// Run RecallSecretVersion after validating the tenant and durably recording the command.
    async fn run_recall_secret_version(
        &self,
        req: secrets::RecallSecretVersionRequest,
    ) -> Result<secrets::RecallSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallSecretVersion", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::ExportSecretVersionsRequest {
            context: req.context.clone(),
            filter: Some(secrets::export_secret_versions_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req
                    .secret_version
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .access_grant_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = SecretVersionQueryServiceClient::new(channel);
            let input: tonic::Request<secrets::ExportSecretVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_secret_versions(input).await?;
            let mut frames = response.into_inner();
            let mut last = secrets::ExportSecretVersionsResponse::default();
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
        let mut response = secrets::RecallSecretVersionResponse::default();
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: i64 = req.secret_version.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.secret_version.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run CommentSecretVersionApproval after validating the tenant and durably recording the command.
    async fn run_comment_secret_version_approval(
        &self,
        req: secrets::CommentSecretVersionApprovalRequest,
    ) -> Result<secrets::CommentSecretVersionApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CommentSecretVersionApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CommentSecretVersionApproval", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let command_0 = CommentSecretVersionApprovalDispatch0::from(req.clone());
        let outgoing_0: fuel::ResolveFuelCardsRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FuelCardQueryServiceClient::new(channel);
            let input: tonic::Request<fuel::ResolveFuelCardsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_fuel_cards(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::CheckAccessGrantEligibilityRequest {
            access_grant: Some(secrets::AccessGrant {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.secrets.v1.CheckAccessGrantEligibilityRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessGrantValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::CheckAccessGrantEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_access_grant_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::CommentSecretVersionApprovalResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: Option<secrets::AccessGrantRef> =
            req.secret_version.clone().unwrap_or_default().subject.clone();
        if contribution != <Option<secrets::AccessGrantRef>>::default() {
            response.secret_version.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CommentSecretVersionApproval for ResolveFuelCards.
struct CommentSecretVersionApprovalDispatch0 {
    context: Option<common::RequestContext>,
}
impl From<secrets::CommentSecretVersionApprovalRequest> for CommentSecretVersionApprovalDispatch0 {
    fn from(req: secrets::CommentSecretVersionApprovalRequest) -> Self {
        Self {
            context: req.context.clone(),
        }
    }
}
impl From<CommentSecretVersionApprovalDispatch0> for fuel::ResolveFuelCardsRequest {
    fn from(value: CommentSecretVersionApprovalDispatch0) -> Self {
        fuel::ResolveFuelCardsRequest {
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl SecretVersionApprovalService for SecretVersionApprovalHandler {
    /// Submits a secret version.
    async fn submit_secret_version(
        &self,
        request: tonic::Request<secrets::SubmitSecretVersionRequest>,
    ) -> Result<tonic::Response<secrets::SubmitSecretVersionResponse>, tonic::Status> {
        self.run_submit_secret_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a secret version.
    async fn approve_secret_version(
        &self,
        request: tonic::Request<secrets::ApproveSecretVersionRequest>,
    ) -> Result<tonic::Response<secrets::ApproveSecretVersionResponse>, tonic::Status> {
        self.run_approve_secret_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a secret version.
    async fn reject_secret_version(
        &self,
        request: tonic::Request<secrets::RejectSecretVersionRequest>,
    ) -> Result<tonic::Response<secrets::RejectSecretVersionResponse>, tonic::Status> {
        self.run_reject_secret_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a secret version.
    async fn escalate_secret_version(
        &self,
        request: tonic::Request<secrets::EscalateSecretVersionRequest>,
    ) -> Result<tonic::Response<secrets::EscalateSecretVersionResponse>, tonic::Status> {
        self.run_escalate_secret_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secret versions a page at a time.
    async fn list_pending_secret_versions(
        &self,
        request: tonic::Request<secrets::ListPendingSecretVersionsRequest>,
    ) -> Result<tonic::Response<secrets::ListPendingSecretVersionsResponse>, tonic::Status> {
        self.run_list_pending_secret_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalls a secret version.
    async fn recall_secret_version(
        &self,
        request: tonic::Request<secrets::RecallSecretVersionRequest>,
    ) -> Result<tonic::Response<secrets::RecallSecretVersionResponse>, tonic::Status> {
        self.run_recall_secret_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Comments a secret version.
    async fn comment_secret_version_approval(
        &self,
        request: tonic::Request<secrets::CommentSecretVersionApprovalRequest>,
    ) -> Result<tonic::Response<secrets::CommentSecretVersionApprovalResponse>, tonic::Status> {
        self.run_comment_secret_version_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
