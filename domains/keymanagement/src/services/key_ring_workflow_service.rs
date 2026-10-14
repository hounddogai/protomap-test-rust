use acme_proto::acme::checkout::v1 as checkout;
use acme_proto::acme::checkout::v1::checkout_session_query_service_client::CheckoutSessionQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_service_client::CryptoKeyServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_client::KeyRingReportingServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_workflow_service_server::KeyRingWorkflowService;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;

/// Runs the long-lived workflow that takes a key ring from start to finish.
#[derive(Clone)]
pub struct KeyRingWorkflowController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    cancel_key_ring_workflow_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::ListFlagRolloutVersionsRequest,
            featureflags::ListFlagRolloutVersionsResponse,
        >,
    >,
    inspect_key_ring_workflow_0_port: acme_platform::ports::Callback<
        ratelimiting::ListThrottleReportsRequest,
        ratelimiting::ListThrottleReportsResponse,
    >,
    list_key_ring_workflows_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            keymanagement::ResumeCryptoKeyWorkflowRequest,
            keymanagement::ResumeCryptoKeyWorkflowResponse,
        >,
    >,
    pause_key_ring_workflow_0_port: acme_platform::ports::Callback<
        keymanagement::EstimateKeyRotationRequest,
        keymanagement::EstimateKeyRotationResponse,
    >,
    start_key_ring_workflow_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::ResumeQuotaBucketWorkflowRequest,
            ratelimiting::ResumeQuotaBucketWorkflowResponse,
        >,
    >,
    start_key_ring_workflow_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                keymanagement::BatchGetCryptoKeysRequest,
                keymanagement::BatchGetCryptoKeysResponse,
            >,
    >,
}
impl KeyRingWorkflowController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::ResumeQuotaBucketWorkflowRequest,
            ratelimiting::ResumeQuotaBucketWorkflowResponse,
        > = {
            let client = QuotaBucketWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ResumeQuotaBucketWorkflowRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ResumeQuotaBucketWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resume_quota_bucket_workflow(request).await?.into_inner())
})
})
        };
        let start_key_ring_workflow_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::BatchGetCryptoKeysRequest,
            keymanagement::BatchGetCryptoKeysResponse,
        > = {
            let client = CryptoKeyServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::BatchGetCryptoKeysRequest| -> acme_platform::ports::RpcFuture<keymanagement::BatchGetCryptoKeysResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.batch_get_crypto_keys(request).await?.into_inner())
})
})
        };
        let start_key_ring_workflow_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::ListFlagRolloutVersionsRequest,
            featureflags::ListFlagRolloutVersionsResponse,
        > = {
            let client = FlagRolloutHistoryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ListFlagRolloutVersionsRequest| -> acme_platform::ports::RpcFuture<featureflags::ListFlagRolloutVersionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_flag_rollout_versions(request).await?.into_inner())
})
})
        };
        let cancel_key_ring_workflow_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::EstimateKeyRotationRequest,
            keymanagement::EstimateKeyRotationResponse,
        > = {
            let client = KeyRotationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::EstimateKeyRotationRequest| -> acme_platform::ports::RpcFuture<keymanagement::EstimateKeyRotationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_key_rotation(request).await?.into_inner())
})
})
        };
        let pause_key_ring_workflow_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::ListThrottleReportsRequest,
            ratelimiting::ListThrottleReportsResponse,
        > = {
            let client = ThrottleReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ListThrottleReportsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ListThrottleReportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_throttle_reports(request).await?.into_inner())
})
})
        };
        let inspect_key_ring_workflow_0_port = callback;
        let callback: acme_platform::ports::Callback<
            keymanagement::ResumeCryptoKeyWorkflowRequest,
            keymanagement::ResumeCryptoKeyWorkflowResponse,
        > = {
            let client = CryptoKeyWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ResumeCryptoKeyWorkflowRequest| -> acme_platform::ports::RpcFuture<keymanagement::ResumeCryptoKeyWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resume_crypto_key_workflow(request).await?.into_inner())
})
})
        };
        let list_key_ring_workflows_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            cancel_key_ring_workflow_0_port,
            inspect_key_ring_workflow_0_port,
            list_key_ring_workflows_0_port,
            pause_key_ring_workflow_0_port,
            start_key_ring_workflow_0_port,
            start_key_ring_workflow_1_port,
        }
    }
    /// Run StartKeyRingWorkflow after validating the tenant and durably recording the command.
    async fn run_start_key_ring_workflow(
        &self,
        req: keymanagement::StartKeyRingWorkflowRequest,
    ) -> Result<keymanagement::StartKeyRingWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StartKeyRingWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartKeyRingWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ResumeQuotaBucketWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::resume_quota_bucket_workflow_request::Details {
                notify: Some(common::ContactInfo {
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
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let reply_0 = acme_platform::ports::execute(self.start_key_ring_workflow_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = keymanagement::BatchGetCryptoKeysRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.start_key_ring_workflow_1_port.as_ref(), outgoing_1).await?;
        let mut response = keymanagement::StartKeyRingWorkflowResponse::default();
        let contribution: String = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.key_ring.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.key_ring.get_or_insert_with(Default::default).manager = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceKeyRingWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_key_ring_workflow(
        &self,
        req: keymanagement::AdvanceKeyRingWorkflowRequest,
    ) -> Result<keymanagement::AdvanceKeyRingWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("AdvanceKeyRingWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceKeyRingWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::AdvanceCryptoKeyWorkflowRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let reply_0 = async move {
            let mut client = CryptoKeyWorkflowServiceClient::new(channel);
            let input: tonic::Request<keymanagement::AdvanceCryptoKeyWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.advance_crypto_key_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::AdvanceKeyRingWorkflowResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.key_ring.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.key_ring.get_or_insert_with(Default::default).manager = contribution;
        }
        Ok(response)
    }
    /// Run CancelKeyRingWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_key_ring_workflow(
        &self,
        req: keymanagement::CancelKeyRingWorkflowRequest,
    ) -> Result<keymanagement::CancelKeyRingWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CancelKeyRingWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelKeyRingWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFlagRolloutVersionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_rollout_versions_request::Filter {
                region: req
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
        let reply_0 = acme_platform::ports::execute(self.cancel_key_ring_workflow_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = keymanagement::StreamKeyRingMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                name: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                preferred_channel: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = KeyRingReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::StreamKeyRingMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
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
        let mut response = keymanagement::CancelKeyRingWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.key_ring.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.key_ring.get_or_insert_with(Default::default).manager = contribution;
        }
        Ok(response)
    }
    /// Run RetryKeyRingWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_key_ring_workflow(
        &self,
        req: keymanagement::RetryKeyRingWorkflowRequest,
    ) -> Result<keymanagement::RetryKeyRingWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RetryKeyRingWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryKeyRingWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ListThrottleConflictsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_throttle_conflicts_request::Filter {
                created_after: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.details.clone().unwrap_or_default().parameters.clone(),
                owner_id: req.key_ring_id.clone(),
                region: req
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
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ListThrottleConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_throttle_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = keymanagement::PreviewCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    updated_by: req
                        .key_ring
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
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            details: Some(keymanagement::preview_crypto_key_request::Details {
                contact: Some(common::ContactInfo {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::PreviewCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = keymanagement::GetKeyRingLifecycleRequest {
            context: req.context.clone(),
            key_ring_id: req.key_ring.clone().unwrap_or_default().key_ring_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = KeyRingLifecycleServiceClient::new(channel);
            let input: tonic::Request<keymanagement::GetKeyRingLifecycleRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_key_ring_lifecycle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::RetryKeyRingWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::PostalAddress> = reply_2.key_ring.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].location = contribution;
        }
        let contribution: Option<common::PostalAddress> = reply_2.key_ring.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0].location = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.key_ring.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PauseKeyRingWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_key_ring_workflow(
        &self,
        req: keymanagement::PauseKeyRingWorkflowRequest,
    ) -> Result<keymanagement::PauseKeyRingWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PauseKeyRingWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseKeyRingWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::EstimateKeyRotationRequest {
            context: req.context.clone(),
            details: Some(keymanagement::estimate_key_rotation_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .key_ring
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
            }),
            key_rotation: Some(keymanagement::KeyRotation {
                audit: Some(common::AuditInfo {
                    etag: req
                        .key_ring
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
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.pause_key_ring_workflow_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = checkout::SummarizeCheckoutSessionsRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(checkout::summarize_checkout_sessions_request::Details {
                period: reply_0.key_rotation.clone().unwrap_or_default().window.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<checkout::SummarizeCheckoutSessionsRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CheckoutSessionQueryServiceClient::new(channel);
            let input: tonic::Request<checkout::SummarizeCheckoutSessionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_checkout_sessions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_2 = PauseKeyRingWorkflowDispatch2::from(req.clone());
        let outgoing_2: keymanagement::ReinstateKeyRingRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = KeyRingLifecycleServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ReinstateKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reinstate_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = keymanagement::CountKeyRingTotalsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::count_key_ring_totals_request::Filter {
                created_after: reply_0
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.details.clone().unwrap_or_default().parameters.clone(),
                owner_id: reply_0
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .organizer
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
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
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.keymanagement.v1.CountKeyRingTotalsRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = KeyRingReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::CountKeyRingTotalsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_key_ring_totals(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::PauseKeyRingWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_2
            .key_ring
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .lines
            .clone();
        if contribution != <Vec<String>>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .lines
                .extend(contribution);
        }
        let contribution: String = reply_2
            .key_ring
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).parent_id = contribution;
        }
        Ok(response)
    }
    /// Run ResumeKeyRingWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_key_ring_workflow(
        &self,
        req: keymanagement::ResumeKeyRingWorkflowRequest,
    ) -> Result<keymanagement::ResumeKeyRingWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResumeKeyRingWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeKeyRingWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::SimulateFlagRolloutRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .key_ring
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
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<featureflags::SimulateFlagRolloutRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = FlagRolloutCalculationServiceClient::new(channel);
                let input: tonic::Request<featureflags::SimulateFlagRolloutRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.simulate_flag_rollout(input).await?;
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
        let mut response = keymanagement::ResumeKeyRingWorkflowResponse::default();
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .organizer
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .manager
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: i32 = req.key_ring.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.key_ring.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run InspectKeyRingWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_key_ring_workflow(
        &self,
        req: keymanagement::InspectKeyRingWorkflowRequest,
    ) -> Result<keymanagement::InspectKeyRingWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("InspectKeyRingWorkflow", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectKeyRingWorkflow", "running workflow");
        let outgoing_0 = ratelimiting::ListThrottleReportsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_throttle_reports_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.key_ring_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.inspect_key_ring_workflow_0_port)(outgoing_0).await?;
        let mut response = keymanagement::InspectKeyRingWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        Ok(response)
    }
    /// Run ListKeyRingWorkflows after validating the tenant and durably recording the command.
    async fn run_list_key_ring_workflows(
        &self,
        req: keymanagement::ListKeyRingWorkflowsRequest,
    ) -> Result<keymanagement::ListKeyRingWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListKeyRingWorkflows", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListKeyRingWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ResumeCryptoKeyWorkflowRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.list_key_ring_workflows_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = keymanagement::ListKeyRingWorkflowsResponse::default();
        let contribution: String = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .key_ring_id
            .clone();
        if contribution != <String>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            if response.key_rings[0].children.is_empty() {
                response.key_rings[0].children.push(Default::default());
            }
            if response.key_rings[0].children[0].children.is_empty() {
                response.key_rings[0].children[0].children.push(Default::default());
            }
            if response.key_rings[0].children[0].children[0].children.is_empty() {
                response.key_rings[0].children[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.key_rings[0].children[0].children[0].children[0].key_ring_id = contribution;
        }
        let contribution: String = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .key_ring_id
            .clone();
        if contribution != <String>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            response.key_rings[0].key_ring_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by PauseKeyRingWorkflow for ReinstateKeyRing.
struct PauseKeyRingWorkflowDispatch2 {
    context: Option<common::RequestContext>,
    key_ring_id: String,
    details_notify_contact_phone: String,
    details_notify_contact_email: String,
}
impl From<keymanagement::PauseKeyRingWorkflowRequest> for PauseKeyRingWorkflowDispatch2 {
    fn from(req: keymanagement::PauseKeyRingWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            key_ring_id: req.key_ring.clone().unwrap_or_default().key_ring_id.clone(),
            details_notify_contact_phone: req
                .details
                .clone()
                .unwrap_or_default()
                .notify
                .clone()
                .unwrap_or_default()
                .phone
                .clone(),
            details_notify_contact_email: req
                .details
                .clone()
                .unwrap_or_default()
                .notify
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
        }
    }
}
impl From<PauseKeyRingWorkflowDispatch2> for keymanagement::ReinstateKeyRingRequest {
    fn from(value: PauseKeyRingWorkflowDispatch2) -> Self {
        keymanagement::ReinstateKeyRingRequest {
            context: value.context,
            details: Some(keymanagement::reinstate_key_ring_request::Details {
                notify_contact: Some(common::ContactInfo {
                    email: value.details_notify_contact_email,
                    phone: value.details_notify_contact_phone,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_ring_id: value.key_ring_id,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRingWorkflowService for KeyRingWorkflowController {
    /// Starts a key ring.
    async fn start_key_ring_workflow(
        &self,
        request: tonic::Request<keymanagement::StartKeyRingWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::StartKeyRingWorkflowResponse>, tonic::Status> {
        self.run_start_key_ring_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances a key ring.
    async fn advance_key_ring_workflow(
        &self,
        request: tonic::Request<keymanagement::AdvanceKeyRingWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::AdvanceKeyRingWorkflowResponse>, tonic::Status> {
        self.run_advance_key_ring_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a key ring.
    async fn cancel_key_ring_workflow(
        &self,
        request: tonic::Request<keymanagement::CancelKeyRingWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::CancelKeyRingWorkflowResponse>, tonic::Status> {
        self.run_cancel_key_ring_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a key ring.
    async fn retry_key_ring_workflow(
        &self,
        request: tonic::Request<keymanagement::RetryKeyRingWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::RetryKeyRingWorkflowResponse>, tonic::Status> {
        self.run_retry_key_ring_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses a key ring.
    async fn pause_key_ring_workflow(
        &self,
        request: tonic::Request<keymanagement::PauseKeyRingWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::PauseKeyRingWorkflowResponse>, tonic::Status> {
        self.run_pause_key_ring_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes a key ring.
    async fn resume_key_ring_workflow(
        &self,
        request: tonic::Request<keymanagement::ResumeKeyRingWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::ResumeKeyRingWorkflowResponse>, tonic::Status> {
        self.run_resume_key_ring_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key ring.
    async fn inspect_key_ring_workflow(
        &self,
        request: tonic::Request<keymanagement::InspectKeyRingWorkflowRequest>,
    ) -> Result<tonic::Response<keymanagement::InspectKeyRingWorkflowResponse>, tonic::Status> {
        self.run_inspect_key_ring_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists key rings a page at a time.
    async fn list_key_ring_workflows(
        &self,
        request: tonic::Request<keymanagement::ListKeyRingWorkflowsRequest>,
    ) -> Result<tonic::Response<keymanagement::ListKeyRingWorkflowsResponse>, tonic::Status> {
        self.run_list_key_ring_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
