use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_history_service_client::AccessPolicyHistoryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_validation_service_client::UserValidationServiceClient;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_validation_service_server::KeyRotationValidationService;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::sponsorships::v1::sponsor_calculation_service_client::SponsorCalculationServiceClient;
use acme_proto::acme::tours::v1 as tours;
use acme_proto::acme::tours::v1::tour_service_client::TourServiceClient;

/// Checks key rotations against the rules before anything acts on them.
#[derive(Clone)]
pub struct KeyRotationValidationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    lint_key_rotations_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            featureflags::PreviewFeatureFlagRequest,
            featureflags::PreviewFeatureFlagResponse,
        >,
    >,
    lint_key_rotations_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<keymanagement::QuoteKeyRingRequest, keymanagement::QuoteKeyRingResponse>,
    >,
    lint_key_rotations_3_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::NormalizeRateLimitRequest,
            ratelimiting::NormalizeRateLimitResponse,
        >,
    >,
    normalize_key_rotation_0_port: acme_platform::ports::Callback<
        keymanagement::ValidateCryptoKeyRequest,
        keymanagement::ValidateCryptoKeyResponse,
    >,
}
impl KeyRotationValidationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
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
        let normalize_key_rotation_0_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::PreviewFeatureFlagRequest,
            featureflags::PreviewFeatureFlagResponse,
        > = {
            let client = FeatureFlagValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::PreviewFeatureFlagRequest| -> acme_platform::ports::RpcFuture<featureflags::PreviewFeatureFlagResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_feature_flag(request).await?.into_inner())
})
})
        };
        let lint_key_rotations_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::QuoteKeyRingRequest,
            keymanagement::QuoteKeyRingResponse,
        > = {
            let client = KeyRingCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::QuoteKeyRingRequest| -> acme_platform::ports::RpcFuture<keymanagement::QuoteKeyRingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_key_ring(request).await?.into_inner())
})
})
        };
        let lint_key_rotations_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::NormalizeRateLimitRequest,
            ratelimiting::NormalizeRateLimitResponse,
        > = {
            let client = RateLimitValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::NormalizeRateLimitRequest| -> acme_platform::ports::RpcFuture<ratelimiting::NormalizeRateLimitResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_rate_limit(request).await?.into_inner())
})
})
        };
        let lint_key_rotations_3_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            lint_key_rotations_0_port,
            lint_key_rotations_1_port,
            lint_key_rotations_3_port,
            normalize_key_rotation_0_port,
        }
    }
    /// Run ValidateKeyRotation after validating the tenant and durably recording the command.
    async fn run_validate_key_rotation(
        &self,
        req: keymanagement::ValidateKeyRotationRequest,
    ) -> Result<keymanagement::ValidateKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CheckRateLimitEligibilityRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    etag: req
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
                    display_name: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CheckRateLimitEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_rate_limit_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::NormalizeKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_at: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            lines: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone(),
                            recipient: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .recipient
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: reply_0
                                .rate_limit
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = KeyRingValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::NormalizeKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ValidateKeyRotationResponse::default();
        let contribution: String = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
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
            response.key_rotation.get_or_insert_with(Default::default).participants[0]
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
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
            response.key_rotation.get_or_insert_with(Default::default).participants[0]
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
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
    /// Run PreviewKeyRotation after validating the tenant and durably recording the command.
    async fn run_preview_key_rotation(
        &self,
        req: keymanagement::PreviewKeyRotationRequest,
    ) -> Result<keymanagement::PreviewKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let command_0 = PreviewKeyRotationDispatch0::from(req.clone());
        let outgoing_0: sponsorships::EstimateSponsorRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SponsorCalculationServiceClient::new(channel);
            let input: tonic::Request<sponsorships::EstimateSponsorRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_sponsor(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::PreviewKeyRotationResponse::default();
        let contribution: String = reply_0
            .sponsor
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .phone
            .clone();
        if contribution != <String>::default() {
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
            response.key_rotation.get_or_insert_with(Default::default).participants[0]
                .contact
                .get_or_insert_with(Default::default)
                .phone = contribution;
        }
        let contribution: String = reply_0
            .sponsor
            .clone()
            .unwrap_or_default()
            .relationship_manager
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
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.key_rotation.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CheckKeyRotationEligibility after validating the tenant and durably recording the command.
    async fn run_check_key_rotation_eligibility(
        &self,
        req: keymanagement::CheckKeyRotationEligibilityRequest,
    ) -> Result<keymanagement::CheckKeyRotationEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckKeyRotationEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckKeyRotationEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let jobs_0 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = configuration::PreviewEnvironmentRequest {
                        context: req.context.clone(),
                        environment: Some(configuration::Environment {
                            owner: Some(identity::UserRef {
                                email: item.contact.clone().unwrap_or_default().email.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = EnvironmentValidationServiceClient::new(channel);
                        let input: tonic::Request<configuration::PreviewEnvironmentRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.preview_environment(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = keymanagement::GetKeyRotationRatesRequest {
            context: req.context.clone(),
            key_rotation_id: req.key_rotation_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = KeyRotationCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::GetKeyRotationRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_key_rotation_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = keymanagement::CheckKeyRotationEligibilityResponse::default();
        let contribution: String = reply_1
            .key_rotation
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
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
        let contribution: std::collections::BTreeMap<String, String> =
            req.key_rotation.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run NormalizeKeyRotation after validating the tenant and durably recording the command.
    async fn run_normalize_key_rotation(
        &self,
        req: keymanagement::NormalizeKeyRotationRequest,
    ) -> Result<keymanagement::NormalizeKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ValidateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    email: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            details: Some(keymanagement::validate_crypto_key_request::Details {
                contact: Some(common::ContactInfo {
                    preferred_channel: req
                        .details
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
        let port = acme_platform::ports::opaque(self.normalize_key_rotation_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let jobs_1 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_0 = reply_0.clone();
                async move {
                    let outgoing_1 = authorization::RevertAccessPolicyRequest {
                        access_policy: Some(authorization::AccessPolicy {
                            audit: Some(common::AuditInfo {
                                created_at: reply_0
                                    .crypto_key
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .created_at
                                    .clone(),
                                created_by: reply_0
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
                            owner: Some(identity::UserRef {
                                display_name: req
                                    .key_rotation
                                    .clone()
                                    .unwrap_or_default()
                                    .organizer
                                    .clone()
                                    .unwrap_or_default()
                                    .display_name
                                    .clone(),
                                email: item.contact.clone().unwrap_or_default().email.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AccessPolicyHistoryServiceClient::new(channel);
                        let input: tonic::Request<authorization::RevertAccessPolicyRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.revert_access_policy(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::NormalizeKeyRotationResponse::default();
        let contribution: String = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .phone
            .clone();
        if contribution != <String>::default() {
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
            response.key_rotation.get_or_insert_with(Default::default).participants[0]
                .contact
                .get_or_insert_with(Default::default)
                .phone = contribution;
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
    /// Run ExplainKeyRotationDecision after validating the tenant and durably recording the command.
    async fn run_explain_key_rotation_decision(
        &self,
        req: keymanagement::ExplainKeyRotationDecisionRequest,
    ) -> Result<keymanagement::ExplainKeyRotationDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainKeyRotationDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainKeyRotationDecision", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ScoreCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    etag: req
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ScoreCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::QuoteEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    recipient: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    etag: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_at: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::QuoteEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ExplainKeyRotationDecisionResponse::default();
        let contribution: Option<common::ContactInfo> = reply_0.crypto_key.clone().unwrap_or_default().contact.clone();
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
            .environment
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
        let contribution: Vec<String> = (match reply_1.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
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
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.key_rotation.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ScoreKeyRotation after validating the tenant and durably recording the command.
    async fn run_score_key_rotation(
        &self,
        req: keymanagement::ScoreKeyRotationRequest,
    ) -> Result<keymanagement::ScoreKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = tours::WatchToursRequest {
            context: req.context.clone(),
            filter: Some(tours::watch_tours_request::Filter {
                created_after: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
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
            since: req
                .key_rotation
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .organizer
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
            let mut client = TourServiceClient::new(channel);
            let input: tonic::Request<tours::WatchToursRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_tours(input).await?;
            let mut frames = response.into_inner();
            let mut last = tours::WatchToursResponse::default();
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
        let outgoing_1 = identity::GetUserRulesRequest {
            context: req.context.clone(),
            user_id: req
                .key_rotation
                .clone()
                .unwrap_or_default()
                .organizer
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<identity::GetUserRulesRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = UserValidationServiceClient::new(channel);
                let input: tonic::Request<identity::GetUserRulesRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.get_user_rules(input).await?;
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
        let jobs_2 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_2 = keymanagement::RecalculateKeyRotationsRequest {
                        context: req.context.clone(),
                        key_rotation: Some(keymanagement::KeyRotation {
                            organizer: Some(identity::UserRef {
                                email: item.contact.clone().unwrap_or_default().email.clone(),
                                ..Default::default()
                            }),
                            participants: vec![keymanagement::KeyRotationParticipant {
                                contact: Some(common::ContactInfo {
                                    phone: item.contact.clone().unwrap_or_default().phone.clone(),
                                    preferred_channel: item
                                        .contact
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
                    async move {
                        let mut client = KeyRotationCalculationServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::RecalculateKeyRotationsRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.recalculate_key_rotations(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::ScoreKeyRotationResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_1
            .user
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.key_rotation.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.key_rotation.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run LintKeyRotations after validating the tenant and durably recording the command.
    async fn run_lint_key_rotations(
        &self,
        req: keymanagement::LintKeyRotationsRequest,
    ) -> Result<keymanagement::LintKeyRotationsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintKeyRotations", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintKeyRotations", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::PreviewFeatureFlagRequest {
            context: req.context.clone(),
            details: Some(featureflags::preview_feature_flag_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
        let reply_0 = acme_platform::ports::execute(self.lint_key_rotations_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = keymanagement::QuoteKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            updated_at: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            locality: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .locality
                                .clone(),
                            recipient: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .recipient
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            email: req
                                .details
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.lint_key_rotations_1_port.as_ref(), outgoing_1).await?;
        let jobs_2 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_2 = keymanagement::CompareCryptoKeyOptionsRequest {
                        context: req.context.clone(),
                        crypto_key: Some(keymanagement::CryptoKey {
                            contact: Some(common::ContactInfo {
                                name: item.contact.clone().unwrap_or_default().name.clone(),
                                preferred_channel: item.contact.clone().unwrap_or_default().preferred_channel.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CryptoKeyCalculationServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::CompareCryptoKeyOptionsRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.compare_crypto_key_options(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let outgoing_3 = ratelimiting::NormalizeRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        notify_email: (match reply_0.feature_flag.clone().unwrap_or_default().action.clone() {
                            Some(featureflags::feature_flag::Action::Block(value)) => value,
                            _ => Default::default(),
                        })
                        .notify_email
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_3 =
            self.lint_key_rotations_3_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_3)
            .await?;
        let mut response = keymanagement::LintKeyRotationsResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .owner
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
        let contribution: String = reply_1
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
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
            response.key_rotation.get_or_insert_with(Default::default).participants[0]
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: Vec<String> = reply_1
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
        let contribution: Option<prost_types::Timestamp> =
            reply_3.outcome.clone().unwrap_or_default().effective_at.clone();
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
        let contribution: i32 = req.key_rotation.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.key_rotation.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run GetKeyRotationRules after validating the tenant and durably recording the command.
    async fn run_get_key_rotation_rules(
        &self,
        req: keymanagement::GetKeyRotationRulesRequest,
    ) -> Result<keymanagement::GetKeyRotationRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetKeyRotationRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetKeyRotationRules", "running workflow");
        let mut response = keymanagement::GetKeyRotationRulesResponse::default();
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by PreviewKeyRotation for EstimateSponsor.
struct PreviewKeyRotationDispatch0 {
    context: Option<common::RequestContext>,
    sponsor_primary_address_region: String,
}
impl From<keymanagement::PreviewKeyRotationRequest> for PreviewKeyRotationDispatch0 {
    fn from(req: keymanagement::PreviewKeyRotationRequest) -> Self {
        Self {
            context: req.context.clone(),
            sponsor_primary_address_region: req
                .key_rotation
                .clone()
                .unwrap_or_default()
                .location
                .clone()
                .unwrap_or_default()
                .region
                .clone(),
        }
    }
}
impl From<PreviewKeyRotationDispatch0> for sponsorships::EstimateSponsorRequest {
    fn from(value: PreviewKeyRotationDispatch0) -> Self {
        sponsorships::EstimateSponsorRequest {
            context: value.context,
            sponsor: Some(sponsorships::Sponsor {
                primary_address: Some(common::PostalAddress {
                    region: value.sponsor_primary_address_region,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRotationValidationService for KeyRotationValidationApi {
    /// Validates a key rotation.
    async fn validate_key_rotation(
        &self,
        request: tonic::Request<keymanagement::ValidateKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::ValidateKeyRotationResponse>, tonic::Status> {
        self.run_validate_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a key rotation.
    async fn preview_key_rotation(
        &self,
        request: tonic::Request<keymanagement::PreviewKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::PreviewKeyRotationResponse>, tonic::Status> {
        self.run_preview_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a key rotation.
    async fn check_key_rotation_eligibility(
        &self,
        request: tonic::Request<keymanagement::CheckKeyRotationEligibilityRequest>,
    ) -> Result<tonic::Response<keymanagement::CheckKeyRotationEligibilityResponse>, tonic::Status> {
        self.run_check_key_rotation_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a key rotation.
    async fn normalize_key_rotation(
        &self,
        request: tonic::Request<keymanagement::NormalizeKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::NormalizeKeyRotationResponse>, tonic::Status> {
        self.run_normalize_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a key rotation.
    async fn explain_key_rotation_decision(
        &self,
        request: tonic::Request<keymanagement::ExplainKeyRotationDecisionRequest>,
    ) -> Result<tonic::Response<keymanagement::ExplainKeyRotationDecisionResponse>, tonic::Status> {
        self.run_explain_key_rotation_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a key rotation.
    async fn score_key_rotation(
        &self,
        request: tonic::Request<keymanagement::ScoreKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::ScoreKeyRotationResponse>, tonic::Status> {
        self.run_score_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a key rotation.
    async fn lint_key_rotations(
        &self,
        request: tonic::Request<keymanagement::LintKeyRotationsRequest>,
    ) -> Result<tonic::Response<keymanagement::LintKeyRotationsResponse>, tonic::Status> {
        self.run_lint_key_rotations(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key rotation.
    async fn get_key_rotation_rules(
        &self,
        request: tonic::Request<keymanagement::GetKeyRotationRulesRequest>,
    ) -> Result<tonic::Response<keymanagement::GetKeyRotationRulesResponse>, tonic::Status> {
        self.run_get_key_rotation_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
