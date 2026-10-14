use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::authorization::v1::permission_check_export_service_client::PermissionCheckExportServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_client::FeatureFlagHistoryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_validation_service_client::UserValidationServiceClient;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_approval_service_server::CryptoKeyApprovalService;
use acme_proto::acme::keymanagement::v1::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_query_service_client::KeyRingQueryServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_client::KeyRingReportingServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_reporting_service_client::KeyRotationReportingServiceClient;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_query_service_client::MetricSeriesQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;

/// Routes crypto keys through approval, with escalation and delegation.
#[derive(Clone)]
pub struct CryptoKeyApprovalHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    delegate_crypto_key_approval_3_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            keymanagement::ShareKeyRotationReportRequest,
            keymanagement::ShareKeyRotationReportResponse,
        >,
    >,
    submit_crypto_key_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<keymanagement::PreviewKeyRingRequest, keymanagement::PreviewKeyRingResponse>,
    >,
}
impl CryptoKeyApprovalHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::PreviewKeyRingRequest,
            keymanagement::PreviewKeyRingResponse,
        > = {
            let client = KeyRingValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::PreviewKeyRingRequest| -> acme_platform::ports::RpcFuture<keymanagement::PreviewKeyRingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_key_ring(request).await?.into_inner())
})
})
        };
        let submit_crypto_key_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::ShareKeyRotationReportRequest,
            keymanagement::ShareKeyRotationReportResponse,
        > = {
            let client = KeyRotationReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ShareKeyRotationReportRequest| -> acme_platform::ports::RpcFuture<keymanagement::ShareKeyRotationReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_key_rotation_report(request).await?.into_inner())
})
})
        };
        let delegate_crypto_key_approval_3_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            delegate_crypto_key_approval_3_port,
            submit_crypto_key_1_port,
        }
    }
    /// Run SubmitCryptoKey after validating the tenant and durably recording the command.
    async fn run_submit_crypto_key(
        &self,
        req: keymanagement::SubmitCryptoKeyRequest,
    ) -> Result<keymanagement::SubmitCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitCryptoKey", "running workflow");
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
                    let outgoing_0 = keymanagement::ScheduleCryptoKeyExpiryRequest {
                        context: req.context.clone(),
                        crypto_key: Some(keymanagement::CryptoKey {
                            limits: vec![keymanagement::CryptoKeyLimit {
                                amount: item.amount.clone(),
                                name: item.name.clone(),
                                period: item.period.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CryptoKeyLifecycleServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::ScheduleCryptoKeyExpiryRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.schedule_crypto_key_expiry(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = keymanagement::PreviewKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
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
        let reply_1 = acme_platform::ports::execute(self.submit_crypto_key_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = authorization::BatchCreateAccessPoliciesRequest {
            context: req.context.clone(),
            chunk: Some(authorization::batch_create_access_policies_request::Chunk::Record(
                authorization::AccessPolicy {
                    audit: Some(common::AuditInfo {
                        created_at: reply_1
                            .key_ring
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
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
                    owner: Some(identity::UserRef {
                        email: reply_0
                            .crypto_key
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AccessPolicyServiceClient::new(channel);
            let input: tonic::Request<authorization::BatchCreateAccessPoliciesRequest> =
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
            let response = client.batch_create_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::StreamQuotaBucketChangesRequest {
            frame: Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(
                ratelimiting::QuotaBucket {
                    funding: Some(ratelimiting::quota_bucket::Funding::Bank(
                        ratelimiting::QuotaBucketBankFunding {
                            bic: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                                Some(keymanagement::crypto_key::Funding::Bank(value)) => value,
                                _ => Default::default(),
                            })
                            .bic
                            .clone(),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = QuotaBucketSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketChangesRequest> =
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
        let mut response = keymanagement::SubmitCryptoKeyResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: String = req.crypto_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run ApproveCryptoKey after validating the tenant and durably recording the command.
    async fn run_approve_crypto_key(
        &self,
        req: keymanagement::ApproveCryptoKeyRequest,
    ) -> Result<keymanagement::ApproveCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = identity::ValidateUserRequest {
            context: req.context.clone(),
            details: Some(identity::validate_user_request::Details {
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
            user: Some(identity::User {
                audit: Some(common::AuditInfo {
                    etag: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
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
                relationship_manager: Some(identity::UserRef {
                    email: req
                        .crypto_key
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = UserValidationServiceClient::new(channel);
            let input: tonic::Request<identity::ValidateUserRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_user(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::CountKeyRingTotalsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::count_key_ring_totals_request::Filter {
                created_after: reply_0
                    .user
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
                region: reply_0
                    .user
                    .clone()
                    .unwrap_or_default()
                    .primary_address
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
            let mut client = KeyRingReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::CountKeyRingTotalsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_key_ring_totals(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ApproveCryptoKeyResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0
            .user
            .clone()
            .unwrap_or_default()
            .contact
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
    /// Run RejectCryptoKey after validating the tenant and durably recording the command.
    async fn run_reject_crypto_key(
        &self,
        req: keymanagement::RejectCryptoKeyRequest,
    ) -> Result<keymanagement::RejectCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::PeekKeyRingRequest {
            context: req.context.clone(),
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = authorization::SharePermissionCheckExportRequest {
            context: req.context.clone(),
            details: Some(authorization::share_permission_check_export_request::Details {
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
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = KeyRingQueryServiceClient::new(channel_0);
                let input: tonic::Request<keymanagement::PeekKeyRingRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.peek_key_ring(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = PermissionCheckExportServiceClient::new(channel_1);
                let input: tonic::Request<authorization::SharePermissionCheckExportRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.share_permission_check_export(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = keymanagement::RejectCryptoKeyResponse::default();
        let contribution: String = reply_0
            .key_ring
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
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run EscalateCryptoKey after validating the tenant and durably recording the command.
    async fn run_escalate_crypto_key(
        &self,
        req: keymanagement::EscalateCryptoKeyRequest,
    ) -> Result<keymanagement::EscalateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::StreamQuotaBucketMetricsRequest {
            context: req.context.clone(),
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
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::StreamQuotaBucketMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_quota_bucket_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::StreamQuotaBucketMetricsResponse::default();
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
        let outgoing_1 = featureflags::ListFeatureFlagVersionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_versions_request::Filter {
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
                owner_id: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                    Some(keymanagement::crypto_key::Funding::ExternalAccountId(value)) => value,
                    _ => Default::default(),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<featureflags::ListFeatureFlagVersionsRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FeatureFlagHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::ListFeatureFlagVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_feature_flag_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = configuration::StreamConfigChangeMetricsRequest {
            context: req.context.clone(),
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
        };
        let channel = self.channel.clone();
        let _reply_2 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamConfigChangeMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
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
        .await?;
        let outgoing_3 = keymanagement::ActivateKeyRingRequest {
            context: req.context.clone(),
            details: Some(keymanagement::activate_key_ring_request::Details {
                new_owner: Some(identity::UserRef {
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
                notify_contact: Some(common::ContactInfo {
                    email: req
                        .crypto_key
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
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.keymanagement.v1.ActivateKeyRingRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = KeyRingLifecycleServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ActivateKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.activate_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::EscalateCryptoKeyResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_3.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
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
    /// Run ListPendingCryptoKeys after validating the tenant and durably recording the command.
    async fn run_list_pending_crypto_keys(
        &self,
        req: keymanagement::ListPendingCryptoKeysRequest,
    ) -> Result<keymanagement::ListPendingCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListPendingCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ScheduleKeyRingExpiryRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
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
                    location: Some(common::PostalAddress {
                        region: req.filter.clone().unwrap_or_default().region.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::ScheduleKeyRingExpiryRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = KeyRingLifecycleServiceClient::new(channel);
                let input: tonic::Request<keymanagement::ScheduleKeyRingExpiryRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.schedule_key_ring_expiry(input).await?;
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
        let mut response = keymanagement::ListPendingCryptoKeysResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.crypto_keys.is_empty() {
                response.crypto_keys.push(Default::default());
            }
            response.crypto_keys[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        Ok(response)
    }
    /// Run DelegateCryptoKeyApproval after validating the tenant and durably recording the command.
    async fn run_delegate_crypto_key_approval(
        &self,
        req: keymanagement::DelegateCryptoKeyApprovalRequest,
    ) -> Result<keymanagement::DelegateCryptoKeyApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateCryptoKeyApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateCryptoKeyApproval", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = metrics::SuggestMetricSeriesesRequest {
            context: req.context.clone(),
            filter: Some(metrics::suggest_metric_serieses_request::Filter {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::ResolveKeyRingsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = MetricSeriesQueryServiceClient::new(channel_0);
                let input: tonic::Request<metrics::SuggestMetricSeriesesRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.suggest_metric_serieses(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = KeyRingQueryServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::ResolveKeyRingsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_key_rings(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = keymanagement::StreamKeyRotationMetricsRequest {
            context: req.context.clone(),
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
            let mut client = KeyRotationReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::StreamKeyRotationMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_key_rotation_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = keymanagement::StreamKeyRotationMetricsResponse::default();
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
        let outgoing_3 = keymanagement::ShareKeyRotationReportRequest {
            context: req.context.clone(),
            details: Some(keymanagement::share_key_rotation_report_request::Details {
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
                    name: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .contact
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_3 =
            self.delegate_crypto_key_approval_3_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_3)
            .await?;
        let mut response = keymanagement::DelegateCryptoKeyApprovalResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_3
            .key_rotation
            .clone()
            .unwrap_or_default()
            .organizer
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: String = req.crypto_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl CryptoKeyApprovalService for CryptoKeyApprovalHandler {
    /// Submits a crypto key.
    async fn submit_crypto_key(
        &self,
        request: tonic::Request<keymanagement::SubmitCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::SubmitCryptoKeyResponse>, tonic::Status> {
        self.run_submit_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Approves a crypto key.
    async fn approve_crypto_key(
        &self,
        request: tonic::Request<keymanagement::ApproveCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::ApproveCryptoKeyResponse>, tonic::Status> {
        self.run_approve_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Rejects a crypto key.
    async fn reject_crypto_key(
        &self,
        request: tonic::Request<keymanagement::RejectCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::RejectCryptoKeyResponse>, tonic::Status> {
        self.run_reject_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Escalates a crypto key.
    async fn escalate_crypto_key(
        &self,
        request: tonic::Request<keymanagement::EscalateCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::EscalateCryptoKeyResponse>, tonic::Status> {
        self.run_escalate_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists crypto keys a page at a time.
    async fn list_pending_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::ListPendingCryptoKeysRequest>,
    ) -> Result<tonic::Response<keymanagement::ListPendingCryptoKeysResponse>, tonic::Status> {
        self.run_list_pending_crypto_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Delegates a crypto key.
    async fn delegate_crypto_key_approval(
        &self,
        request: tonic::Request<keymanagement::DelegateCryptoKeyApprovalRequest>,
    ) -> Result<tonic::Response<keymanagement::DelegateCryptoKeyApprovalResponse>, tonic::Status> {
        self.run_delegate_crypto_key_approval(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
