use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_approval_service_client::EnvironmentApprovalServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_query_service_client::DocumentQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_reporting_service_client::KeyRingReportingServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_lifecycle_service_server::KeyRotationLifecycleService;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_service_client::LedgerAccountServiceClient;
use acme_proto::acme::messaging::v1 as messaging;
use acme_proto::acme::messaging::v1::conversation_calculation_service_client::ConversationCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_query_service_client::SearchIndexQueryServiceClient;

/// Moves a key rotation between the states of its lifecycle.
#[derive(Clone)]
pub struct KeyRotationLifecycleHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    activate_key_rotation_0_port: acme_platform::ports::Callback<
        keymanagement::SimulateKeyRotationRequest,
        keymanagement::SimulateKeyRotationResponse,
    >,
    renew_key_rotation_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::UnsubscribeFlagRolloutUpdatesRequest,
            featureflags::UnsubscribeFlagRolloutUpdatesResponse,
        >,
    >,
    renew_key_rotation_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::SubmitEnvironmentRequest,
                configuration::SubmitEnvironmentResponse,
            >,
    >,
    schedule_key_rotation_expiry_0_port:
        acme_platform::ports::Callback<ledger::ListLedgerAccountsRequest, ledger::ListLedgerAccountsResponse>,
    schedule_key_rotation_expiry_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            keymanagement::ExplainCryptoKeyDecisionRequest,
            keymanagement::ExplainCryptoKeyDecisionResponse,
        >,
    >,
    suspend_key_rotation_0_port: acme_platform::ports::Callback<
        keymanagement::ValidateCryptoKeyRequest,
        keymanagement::ValidateCryptoKeyResponse,
    >,
}
impl KeyRotationLifecycleHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::SimulateKeyRotationRequest,
            keymanagement::SimulateKeyRotationResponse,
        > = {
            let client = KeyRotationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::SimulateKeyRotationRequest| -> acme_platform::ports::RpcFuture<keymanagement::SimulateKeyRotationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_key_rotation(request).await?.into_inner())
})
})
        };
        let activate_key_rotation_0_port = callback;
        let callback: acme_platform::ports::Callback<
            keymanagement::ValidateCryptoKeyRequest,
            keymanagement::ValidateCryptoKeyResponse,
        > = {
            let client = CryptoKeyValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ValidateCryptoKeyRequest| -> acme_platform::ports::RpcFuture<keymanagement::ValidateCryptoKeyResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.validate_crypto_key(request).await?.into_inner())
})
})
        };
        let suspend_key_rotation_0_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::UnsubscribeFlagRolloutUpdatesRequest,
            featureflags::UnsubscribeFlagRolloutUpdatesResponse,
        > = {
            let client = FlagRolloutNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::UnsubscribeFlagRolloutUpdatesRequest| -> acme_platform::ports::RpcFuture<featureflags::UnsubscribeFlagRolloutUpdatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.unsubscribe_flag_rollout_updates(request).await?.into_inner())
})
})
        };
        let renew_key_rotation_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::SubmitEnvironmentRequest,
            configuration::SubmitEnvironmentResponse,
        > = {
            let client = EnvironmentApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::SubmitEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::SubmitEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.submit_environment(request).await?.into_inner())
})
})
        };
        let renew_key_rotation_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ledger::ListLedgerAccountsRequest,
            ledger::ListLedgerAccountsResponse,
        > = {
            let client = LedgerAccountServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ledger::ListLedgerAccountsRequest| -> acme_platform::ports::RpcFuture<ledger::ListLedgerAccountsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_ledger_accounts(request).await?.into_inner())
})
})
        };
        let schedule_key_rotation_expiry_0_port = callback;
        let callback: acme_platform::ports::Callback<
            keymanagement::ExplainCryptoKeyDecisionRequest,
            keymanagement::ExplainCryptoKeyDecisionResponse,
        > = {
            let client = CryptoKeyValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ExplainCryptoKeyDecisionRequest| -> acme_platform::ports::RpcFuture<keymanagement::ExplainCryptoKeyDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_crypto_key_decision(request).await?.into_inner())
})
})
        };
        let schedule_key_rotation_expiry_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            activate_key_rotation_0_port,
            renew_key_rotation_0_port,
            renew_key_rotation_1_port,
            schedule_key_rotation_expiry_0_port,
            schedule_key_rotation_expiry_1_port,
            suspend_key_rotation_0_port,
        }
    }
    /// Run ActivateKeyRotation after validating the tenant and durably recording the command.
    async fn run_activate_key_rotation(
        &self,
        req: keymanagement::ActivateKeyRotationRequest,
    ) -> Result<keymanagement::ActivateKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ActivateKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ActivateKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::SimulateKeyRotationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                organizer: Some(identity::UserRef {
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify_contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        preferred_channel: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify_contact
                            .clone()
                            .unwrap_or_default()
                            .preferred_channel
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.activate_key_rotation_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = documents::ExportDocumentsRequest {
            context: req.context.clone(),
            filter: Some(documents::export_documents_request::Filter {
                created_after: reply_0
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
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
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = DocumentQueryServiceClient::new(channel);
            let input: tonic::Request<documents::ExportDocumentsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_documents(input).await?;
            let mut frames = response.into_inner();
            let mut last = documents::ExportDocumentsResponse::default();
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
        let mut response = keymanagement::ActivateKeyRotationResponse::default();
        let contribution: i32 = reply_0.key_rotation.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.key_rotation.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run SuspendKeyRotation after validating the tenant and durably recording the command.
    async fn run_suspend_key_rotation(
        &self,
        req: keymanagement::SuspendKeyRotationRequest,
    ) -> Result<keymanagement::SuspendKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuspendKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuspendKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ValidateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            details: Some(keymanagement::validate_crypto_key_request::Details {
                contact: Some(common::ContactInfo {
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let port = acme_platform::ports::opaque(self.suspend_key_rotation_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = keymanagement::SuspendKeyRotationResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .new_owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run ReinstateKeyRotation after validating the tenant and durably recording the command.
    async fn run_reinstate_key_rotation(
        &self,
        req: keymanagement::ReinstateKeyRotationRequest,
    ) -> Result<keymanagement::ReinstateKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReinstateKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReinstateKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ResumeQuotaBucketWorkflowRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = QuotaBucketWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ResumeQuotaBucketWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_quota_bucket_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = ratelimiting::CancelQuotaBucketWorkflowRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::cancel_quota_bucket_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
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
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: reply_0
                        .quota_bucket
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
            let mut client = QuotaBucketWorkflowServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CancelQuotaBucketWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.cancel_quota_bucket_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = messaging::ExplainConversationCalculationRequest {
            context: req.context.clone(),
            conversation: Some(messaging::Conversation {
                comments: vec![messaging::ConversationComment {
                    created_at: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }],
                requester: Some(common::ContactInfo {
                    phone: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: reply_0
                        .quota_bucket
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConversationCalculationServiceClient::new(channel);
            let input: tonic::Request<messaging::ExplainConversationCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_conversation_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ReinstateKeyRotationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::ContactInfo> =
            reply_1.quota_bucket.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            if response
                .key_rotation
                .get_or_insert_with(Default::default)
                .participants
                .is_empty()
            {
                response
                    .key_rotation
                    .get_or_insert_with(Default::default)
                    .participants
                    .push(Default::default());
            }
            response.key_rotation.get_or_insert_with(Default::default).participants[0].contact = contribution;
        }
        let contribution: String = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TerminateKeyRotation after validating the tenant and durably recording the command.
    async fn run_terminate_key_rotation(
        &self,
        req: keymanagement::TerminateKeyRotationRequest,
    ) -> Result<keymanagement::TerminateKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TerminateKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TerminateKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = search::SuggestSearchIndexesRequest {
            context: req.context.clone(),
            filter: Some(search::suggest_search_indexes_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req.key_rotation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<search::SuggestSearchIndexesRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SearchIndexQueryServiceClient::new(channel);
            let input: tonic::Request<search::SuggestSearchIndexesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_search_indexes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_1 = TerminateKeyRotationDispatch1::from(req.clone());
        let outgoing_1: featureflags::PreviewFeatureFlagRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FeatureFlagValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::PreviewFeatureFlagRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_feature_flag(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::TerminateKeyRotationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: String = req
            .details
            .clone()
            .unwrap_or_default()
            .notify_contact
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run RenewKeyRotation after validating the tenant and durably recording the command.
    async fn run_renew_key_rotation(
        &self,
        req: keymanagement::RenewKeyRotationRequest,
    ) -> Result<keymanagement::RenewKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RenewKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RenewKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = featureflags::UnsubscribeFlagRolloutUpdatesRequest {
            context: req.context.clone(),
            details: Some(featureflags::unsubscribe_flag_rollout_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let reply_0 = acme_platform::ports::execute(self.renew_key_rotation_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = configuration::SubmitEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_at: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: reply_0
                        .flag_rollout
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
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let reply_1 = acme_platform::ports::execute(self.renew_key_rotation_1_port.as_ref(), outgoing_1).await?;
        let mut response = keymanagement::RenewKeyRotationResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .locality
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ExpireKeyRotation after validating the tenant and durably recording the command.
    async fn run_expire_key_rotation(
        &self,
        req: keymanagement::ExpireKeyRotationRequest,
    ) -> Result<keymanagement::ExpireKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExpireKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExpireKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let command_0 = ExpireKeyRotationDispatch0::from(req.clone());
        let outgoing_0: keymanagement::ShareKeyRingReportRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ShareKeyRingReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_key_ring_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::CalculateKeyRotationRequest {
            context: req.context.clone(),
            details: Some(keymanagement::calculate_key_rotation_request::Details {
                destination: Some(common::PostalAddress {
                    lines: reply_0
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_rotation: Some(keymanagement::KeyRotation {
                audit: Some(common::AuditInfo {
                    etag: reply_0
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
                organizer: Some(identity::UserRef {
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        email: reply_0
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.keymanagement.v1.CalculateKeyRotationRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = KeyRotationCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::CalculateKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ExpireKeyRotationResponse::default();
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
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Vec<String> = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .lines
            .clone();
        if contribution != <Vec<String>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .lines
                .extend(contribution);
        }
        let contribution: String = reply_1
            .key_rotation
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run GetKeyRotationLifecycle after validating the tenant and durably recording the command.
    async fn run_get_key_rotation_lifecycle(
        &self,
        req: keymanagement::GetKeyRotationLifecycleRequest,
    ) -> Result<keymanagement::GetKeyRotationLifecycleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetKeyRotationLifecycle", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetKeyRotationLifecycle", "running workflow");
        let outgoing_0 = keymanagement::ValidateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    created_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
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
                funding: Some(keymanagement::crypto_key::Funding::ExternalAccountId(
                    req.key_rotation_id.clone(),
                )),
                ..Default::default()
            }),
            details: Some(keymanagement::validate_crypto_key_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::ValidateCryptoKeyRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CryptoKeyValidationServiceClient::new(channel);
                let input: tonic::Request<keymanagement::ValidateCryptoKeyRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.validate_crypto_key(input).await?;
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
        let mut response = keymanagement::GetKeyRotationLifecycleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleKeyRotationExpiry after validating the tenant and durably recording the command.
    async fn run_schedule_key_rotation_expiry(
        &self,
        req: keymanagement::ScheduleKeyRotationExpiryRequest,
    ) -> Result<keymanagement::ScheduleKeyRotationExpiryResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleKeyRotationExpiry", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleKeyRotationExpiry", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = ledger::ListLedgerAccountsRequest {
            context: req.context.clone(),
            filter: Some(ledger::list_ledger_accounts_request::Filter {
                created_after: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .organizer
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
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
        let reply_0 = (self.schedule_key_rotation_expiry_0_port)(outgoing_0).await?;
        let outgoing_1 = keymanagement::ExplainCryptoKeyDecisionRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    updated_by: req
                        .key_rotation
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
            details: Some(keymanagement::explain_crypto_key_decision_request::Details {
                contact: Some(common::ContactInfo {
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
            }),
            ..Default::default()
        };
        let reply_1 =
            self.schedule_key_rotation_expiry_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = keymanagement::ScheduleKeyRotationExpiryResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.key_rotation.clone().unwrap_or_default().organizer.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.key_rotation.get_or_insert_with(Default::default).organizer = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by TerminateKeyRotation for PreviewFeatureFlag.
struct TerminateKeyRotationDispatch1 {
    context: Option<common::RequestContext>,
    feature_flag_owner_user_id: String,
    details_contact_email: String,
    details_contact_name: Option<common::PersonName>,
}
impl From<keymanagement::TerminateKeyRotationRequest> for TerminateKeyRotationDispatch1 {
    fn from(req: keymanagement::TerminateKeyRotationRequest) -> Self {
        Self {
            context: req.context.clone(),
            feature_flag_owner_user_id: req
                .details
                .clone()
                .unwrap_or_default()
                .new_owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            details_contact_email: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            details_contact_name: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .name
                .clone(),
        }
    }
}
impl From<TerminateKeyRotationDispatch1> for featureflags::PreviewFeatureFlagRequest {
    fn from(value: TerminateKeyRotationDispatch1) -> Self {
        featureflags::PreviewFeatureFlagRequest {
            context: value.context,
            details: Some(featureflags::preview_feature_flag_request::Details {
                contact: Some(common::ContactInfo {
                    email: value.details_contact_email,
                    name: value.details_contact_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                owner: Some(identity::UserRef {
                    user_id: value.feature_flag_owner_user_id,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by ExpireKeyRotation for ShareKeyRingReport.
struct ExpireKeyRotationDispatch0 {
    context: Option<common::RequestContext>,
    details_recipient_preferred_channel: String,
    details_recipient_email: String,
    details_recipient_phone: String,
}
impl From<keymanagement::ExpireKeyRotationRequest> for ExpireKeyRotationDispatch0 {
    fn from(req: keymanagement::ExpireKeyRotationRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_recipient_preferred_channel: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .preferred_channel
                .clone(),
            details_recipient_email: req
                .details
                .clone()
                .unwrap_or_default()
                .new_owner
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            details_recipient_phone: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .phone
                .clone(),
        }
    }
}
impl From<ExpireKeyRotationDispatch0> for keymanagement::ShareKeyRingReportRequest {
    fn from(value: ExpireKeyRotationDispatch0) -> Self {
        keymanagement::ShareKeyRingReportRequest {
            context: value.context,
            details: Some(keymanagement::share_key_ring_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: value.details_recipient_email,
                    phone: value.details_recipient_phone,
                    preferred_channel: value.details_recipient_preferred_channel,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRotationLifecycleService for KeyRotationLifecycleHandler {
    /// Activates a key rotation.
    async fn activate_key_rotation(
        &self,
        request: tonic::Request<keymanagement::ActivateKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::ActivateKeyRotationResponse>, tonic::Status> {
        self.run_activate_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Suspends a key rotation.
    async fn suspend_key_rotation(
        &self,
        request: tonic::Request<keymanagement::SuspendKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::SuspendKeyRotationResponse>, tonic::Status> {
        self.run_suspend_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reinstates a key rotation.
    async fn reinstate_key_rotation(
        &self,
        request: tonic::Request<keymanagement::ReinstateKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::ReinstateKeyRotationResponse>, tonic::Status> {
        self.run_reinstate_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Terminates a key rotation.
    async fn terminate_key_rotation(
        &self,
        request: tonic::Request<keymanagement::TerminateKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::TerminateKeyRotationResponse>, tonic::Status> {
        self.run_terminate_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Renews a key rotation.
    async fn renew_key_rotation(
        &self,
        request: tonic::Request<keymanagement::RenewKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::RenewKeyRotationResponse>, tonic::Status> {
        self.run_renew_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Expires a key rotation.
    async fn expire_key_rotation(
        &self,
        request: tonic::Request<keymanagement::ExpireKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::ExpireKeyRotationResponse>, tonic::Status> {
        self.run_expire_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key rotation.
    async fn get_key_rotation_lifecycle(
        &self,
        request: tonic::Request<keymanagement::GetKeyRotationLifecycleRequest>,
    ) -> Result<tonic::Response<keymanagement::GetKeyRotationLifecycleResponse>, tonic::Status> {
        self.run_get_key_rotation_lifecycle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules a key rotation.
    async fn schedule_key_rotation_expiry(
        &self,
        request: tonic::Request<keymanagement::ScheduleKeyRotationExpiryRequest>,
    ) -> Result<tonic::Response<keymanagement::ScheduleKeyRotationExpiryResponse>, tonic::Status> {
        self.run_schedule_key_rotation_expiry(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
