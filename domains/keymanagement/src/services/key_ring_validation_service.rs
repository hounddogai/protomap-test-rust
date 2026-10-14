use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_query_service_client::DocumentQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_server::KeyRingValidationService;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_service_client::MetricSeriesServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_validation_service_client::NotificationValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;

/// Checks key rings against the rules before anything acts on them.
#[derive(Clone)]
pub struct KeyRingValidationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    check_key_ring_eligibility_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                keymanagement::CalculateKeyRotationRequest,
                keymanagement::CalculateKeyRotationResponse,
            >,
    >,
    check_key_ring_eligibility_1_port:
        acme_platform::ports::Callback<documents::ResolveDocumentsRequest, documents::ResolveDocumentsResponse>,
    check_key_ring_eligibility_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::SummarizeConfigChangesRequest,
            configuration::SummarizeConfigChangesResponse,
        >,
    >,
    check_key_ring_eligibility_3_port: acme_platform::ports::Callback<
        keymanagement::ExplainCryptoKeyDecisionRequest,
        keymanagement::ExplainCryptoKeyDecisionResponse,
    >,
    normalize_key_ring_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            keymanagement::ExplainKeyRotationCalculationRequest,
            keymanagement::ExplainKeyRotationCalculationResponse,
        >,
    >,
    normalize_key_ring_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<metrics::ListMetricSeriesesRequest, metrics::ListMetricSeriesesResponse>,
    >,
    normalize_key_ring_3_port: acme_platform::ports::Callback<
        keymanagement::EstimateKeyRotationRequest,
        keymanagement::EstimateKeyRotationResponse,
    >,
    preview_key_ring_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            keymanagement::ExplainKeyRotationCalculationRequest,
            keymanagement::ExplainKeyRotationCalculationResponse,
        >,
    >,
    preview_key_ring_1_port:
        acme_platform::ports::Callback<keymanagement::PreviewCryptoKeyRequest, keymanagement::PreviewCryptoKeyResponse>,
}
impl KeyRingValidationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::ExplainKeyRotationCalculationRequest,
            keymanagement::ExplainKeyRotationCalculationResponse,
        > = {
            let client = KeyRotationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ExplainKeyRotationCalculationRequest| -> acme_platform::ports::RpcFuture<keymanagement::ExplainKeyRotationCalculationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_key_rotation_calculation(request).await?.into_inner())
})
})
        };
        let preview_key_ring_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            keymanagement::PreviewCryptoKeyRequest,
            keymanagement::PreviewCryptoKeyResponse,
        > = {
            let client = CryptoKeyValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::PreviewCryptoKeyRequest| -> acme_platform::ports::RpcFuture<keymanagement::PreviewCryptoKeyResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_crypto_key(request).await?.into_inner())
})
})
        };
        let preview_key_ring_1_port = callback;
        let callback: acme_platform::ports::Callback<
            keymanagement::CalculateKeyRotationRequest,
            keymanagement::CalculateKeyRotationResponse,
        > = {
            let client = KeyRotationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::CalculateKeyRotationRequest| -> acme_platform::ports::RpcFuture<keymanagement::CalculateKeyRotationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_key_rotation(request).await?.into_inner())
})
})
        };
        let check_key_ring_eligibility_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            documents::ResolveDocumentsRequest,
            documents::ResolveDocumentsResponse,
        > = {
            let client = DocumentQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: documents::ResolveDocumentsRequest| -> acme_platform::ports::RpcFuture<documents::ResolveDocumentsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.resolve_documents(request).await?.into_inner())
})
})
        };
        let check_key_ring_eligibility_1_port = callback;
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
        let check_key_ring_eligibility_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
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
        let check_key_ring_eligibility_3_port = callback;
        let callback: acme_platform::ports::Callback<
            keymanagement::ExplainKeyRotationCalculationRequest,
            keymanagement::ExplainKeyRotationCalculationResponse,
        > = {
            let client = KeyRotationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::ExplainKeyRotationCalculationRequest| -> acme_platform::ports::RpcFuture<keymanagement::ExplainKeyRotationCalculationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_key_rotation_calculation(request).await?.into_inner())
})
})
        };
        let normalize_key_ring_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            metrics::ListMetricSeriesesRequest,
            metrics::ListMetricSeriesesResponse,
        > = {
            let client = MetricSeriesServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: metrics::ListMetricSeriesesRequest| -> acme_platform::ports::RpcFuture<metrics::ListMetricSeriesesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_metric_serieses(request).await?.into_inner())
})
})
        };
        let normalize_key_ring_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let normalize_key_ring_3_port = callback;
        Self {
            channel,
            repository,
            check_key_ring_eligibility_0_port,
            check_key_ring_eligibility_1_port,
            check_key_ring_eligibility_2_port,
            check_key_ring_eligibility_3_port,
            normalize_key_ring_1_port,
            normalize_key_ring_2_port,
            normalize_key_ring_3_port,
            preview_key_ring_0_port,
            preview_key_ring_1_port,
        }
    }
    /// Run ValidateKeyRing after validating the tenant and durably recording the command.
    async fn run_validate_key_ring(
        &self,
        req: keymanagement::ValidateKeyRingRequest,
    ) -> Result<keymanagement::ValidateKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::RecalculateQuotaBucketsRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
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
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.ratelimiting.v1.RecalculateQuotaBucketsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RecalculateQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::BrowseCertificateAuthoritiesRequest {
            context: req.context.clone(),
            filter: Some(certificates::browse_certificate_authorities_request::Filter {
                labels: (reply_0.quota_bucket.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
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
        let reply_1 = async move {
            let mut client = CertificateAuthorityQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::BrowseCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_certificate_authorities(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = keymanagement::SimulateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    name: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: req
                        .details
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
            details: Some(keymanagement::simulate_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    locality: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SimulateCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_3 = featureflags::NormalizeFlagEvaluationRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                subject: Some(featureflags::FlagRolloutRef {
                    display_name: reply_2
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
        let reply_3 = async move {
            let mut client = FlagEvaluationValidationServiceClient::new(channel);
            let input: tonic::Request<featureflags::NormalizeFlagEvaluationRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_flag_evaluation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ValidateKeyRingResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
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
                .updated_at = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: String = reply_2
            .crypto_key
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
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_2
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
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
    /// Run PreviewKeyRing after validating the tenant and durably recording the command.
    async fn run_preview_key_ring(
        &self,
        req: keymanagement::PreviewKeyRingRequest,
    ) -> Result<keymanagement::PreviewKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ExplainKeyRotationCalculationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
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
                location: Some(common::PostalAddress {
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
            ..Default::default()
        };
        let reply_0 =
            self.preview_key_ring_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = keymanagement::PreviewCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: reply_0
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_by: reply_0
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
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.preview_key_ring_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = keymanagement::PreviewKeyRingResponse::default();
        let contribution: String = reply_0
            .key_rotation
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
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
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_0
            .key_rotation
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .locality
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
                .locality = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
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
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run CheckKeyRingEligibility after validating the tenant and durably recording the command.
    async fn run_check_key_ring_eligibility(
        &self,
        req: keymanagement::CheckKeyRingEligibilityRequest,
    ) -> Result<keymanagement::CheckKeyRingEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckKeyRingEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckKeyRingEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::CalculateKeyRotationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                location: Some(common::PostalAddress {
                    postal_code: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    ..Default::default()
                }),
                organizer: Some(identity::UserRef {
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
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.check_key_ring_eligibility_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = documents::ResolveDocumentsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = (self.check_key_ring_eligibility_1_port)(outgoing_1).await?;
        let outgoing_2 = configuration::SummarizeConfigChangesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(configuration::summarize_config_changes_request::Details {
                filters: reply_0.key_rotation.clone().unwrap_or_default().metadata.clone(),
                period: reply_0.key_rotation.clone().unwrap_or_default().window.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_2 =
            self.check_key_ring_eligibility_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let outgoing_3 = keymanagement::ExplainCryptoKeyDecisionRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    email: req
                        .details
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
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: req
                        .details
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
        let port = acme_platform::ports::opaque(self.check_key_ring_eligibility_3_port.clone());
        let reply_3 = acme_platform::ports::execute(&port, outgoing_3).await?;
        let mut response = keymanagement::CheckKeyRingEligibilityResponse::default();
        let contribution: Option<common::PostalAddress> =
            reply_0.key_rotation.clone().unwrap_or_default().location.clone();
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
            if response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0].children[0].location =
                contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeKeyRing after validating the tenant and durably recording the command.
    async fn run_normalize_key_ring(
        &self,
        req: keymanagement::NormalizeKeyRingRequest,
    ) -> Result<keymanagement::NormalizeKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::CalculateKeyRotationRequest {
            context: req.context.clone(),
            details: Some(keymanagement::calculate_key_rotation_request::Details {
                destination: Some(common::PostalAddress {
                    lines: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    locality: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::CalculateKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::ExplainKeyRotationCalculationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        email: reply_0
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.normalize_key_ring_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = metrics::ListMetricSeriesesRequest {
            context: req.context.clone(),
            filter: Some(metrics::list_metric_serieses_request::Filter {
                created_after: reply_0
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                region: reply_1
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
        let reply_2 = acme_platform::ports::execute(self.normalize_key_ring_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = keymanagement::EstimateKeyRotationRequest {
            context: req.context.clone(),
            details: Some(keymanagement::estimate_key_rotation_request::Details {
                destination: Some(common::PostalAddress {
                    recipient: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .recipient
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_rotation: Some(keymanagement::KeyRotation {
                audit: Some(common::AuditInfo {
                    updated_by: reply_1
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
                organizer: Some(identity::UserRef {
                    user_id: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        email: reply_0
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_3 = (self.normalize_key_ring_3_port)(outgoing_3).await?;
        let mut response = keymanagement::NormalizeKeyRingResponse::default();
        let contribution: String = reply_0
            .key_rotation
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .region
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
                .region = contribution;
        }
        let contribution: Vec<String> = reply_0
            .key_rotation
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
        let contribution: String = reply_1
            .key_rotation
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .recipient
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .recipient = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: i32 = req.key_ring.clone().unwrap_or_default().depth.clone();
        if contribution != <i32>::default() {
            response.key_ring.get_or_insert_with(Default::default).depth = contribution;
        }
        Ok(response)
    }
    /// Run ExplainKeyRingDecision after validating the tenant and durably recording the command.
    async fn run_explain_key_ring_decision(
        &self,
        req: keymanagement::ExplainKeyRingDecisionRequest,
    ) -> Result<keymanagement::ExplainKeyRingDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExplainKeyRingDecision", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainKeyRingDecision", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::EstimateKeyRotationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                organizer: Some(identity::UserRef {
                    display_name: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        name: req
                            .details
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<keymanagement::EstimateKeyRotationRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::EstimateKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = documents::ExportDocumentsRequest {
            context: req.context.clone(),
            filter: Some(documents::export_documents_request::Filter {
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
        let outgoing_2 = ratelimiting::ExplainRateLimitDecisionRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
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
            }),
            ..Default::default()
        };
        let outgoing_2 =
            acme_platform::codec::through_any(&outgoing_2, "acme.ratelimiting.v1.ExplainRateLimitDecisionRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainRateLimitDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_rate_limit_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = keymanagement::LintCryptoKeysRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
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
                contact: Some(common::ContactInfo {
                    email: reply_2
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::LintCryptoKeysRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CryptoKeyValidationServiceClient::new(channel);
                let input: tonic::Request<keymanagement::LintCryptoKeysRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.lint_crypto_keys(input).await?;
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
        let mut response = keymanagement::ExplainKeyRingDecisionResponse::default();
        let contribution: Vec<String> = reply_0
            .key_rotation
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
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .location
                .get_or_insert_with(Default::default)
                .lines
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .key_rotation
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
        let contribution: String = reply_2
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
                .email = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
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
                .updated_at = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
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
    /// Run ScoreKeyRing after validating the tenant and durably recording the command.
    async fn run_score_key_ring(
        &self,
        req: keymanagement::ScoreKeyRingRequest,
    ) -> Result<keymanagement::ScoreKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ScoreCryptoKeyRequest {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            details: Some(keymanagement::score_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    locality: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<keymanagement::ScoreCryptoKeyRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
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
        let command_1 = ScoreKeyRingDispatch1::from(req.clone());
        let outgoing_1: keymanagement::RecalculateCryptoKeysRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::RecalculateCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ScoreKeyRingResponse::default();
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
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0].key_ring_id = contribution;
        }
        let contribution: String = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
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
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run LintKeyRings after validating the tenant and durably recording the command.
    async fn run_lint_key_rings(
        &self,
        req: keymanagement::LintKeyRingsRequest,
    ) -> Result<keymanagement::LintKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = notifications::NormalizeNotificationRequest {
            context: req.context.clone(),
            notification: Some(notifications::Notification {
                assignee: Some(identity::UserRef {
                    display_name: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    user_id: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
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
                requester: Some(common::ContactInfo {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = NotificationValidationServiceClient::new(channel);
            let input: tonic::Request<notifications::NormalizeNotificationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_notification(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = keymanagement::SimulateKeyRotationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                organizer: Some(identity::UserRef {
                    email: reply_0
                        .notification
                        .clone()
                        .unwrap_or_default()
                        .assignee
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        name: reply_0
                            .notification
                            .clone()
                            .unwrap_or_default()
                            .requester
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        phone: reply_0
                            .notification
                            .clone()
                            .unwrap_or_default()
                            .requester
                            .clone()
                            .unwrap_or_default()
                            .phone
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
        let reply_1 = async move {
            let mut client = KeyRotationCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SimulateKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::LintKeyRingsResponse::default();
        let contribution: String = reply_0
            .notification
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run GetKeyRingRules after validating the tenant and durably recording the command.
    async fn run_get_key_ring_rules(
        &self,
        req: keymanagement::GetKeyRingRulesRequest,
    ) -> Result<keymanagement::GetKeyRingRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetKeyRingRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetKeyRingRules", "running workflow");
        let mut response = keymanagement::GetKeyRingRulesResponse::default();
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ScoreKeyRing for RecalculateCryptoKeys.
struct ScoreKeyRingDispatch1 {
    context: Option<common::RequestContext>,
    details_destination_country_code: String,
}
impl From<keymanagement::ScoreKeyRingRequest> for ScoreKeyRingDispatch1 {
    fn from(req: keymanagement::ScoreKeyRingRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_destination_country_code: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .country_code
                .clone(),
        }
    }
}
impl From<ScoreKeyRingDispatch1> for keymanagement::RecalculateCryptoKeysRequest {
    fn from(value: ScoreKeyRingDispatch1) -> Self {
        keymanagement::RecalculateCryptoKeysRequest {
            context: value.context,
            details: Some(keymanagement::recalculate_crypto_keys_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: value.details_destination_country_code,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRingValidationService for KeyRingValidationHandler {
    /// Validates a key ring.
    async fn validate_key_ring(
        &self,
        request: tonic::Request<keymanagement::ValidateKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::ValidateKeyRingResponse>, tonic::Status> {
        self.run_validate_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a key ring.
    async fn preview_key_ring(
        &self,
        request: tonic::Request<keymanagement::PreviewKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::PreviewKeyRingResponse>, tonic::Status> {
        self.run_preview_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a key ring.
    async fn check_key_ring_eligibility(
        &self,
        request: tonic::Request<keymanagement::CheckKeyRingEligibilityRequest>,
    ) -> Result<tonic::Response<keymanagement::CheckKeyRingEligibilityResponse>, tonic::Status> {
        self.run_check_key_ring_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a key ring.
    async fn normalize_key_ring(
        &self,
        request: tonic::Request<keymanagement::NormalizeKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::NormalizeKeyRingResponse>, tonic::Status> {
        self.run_normalize_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a key ring.
    async fn explain_key_ring_decision(
        &self,
        request: tonic::Request<keymanagement::ExplainKeyRingDecisionRequest>,
    ) -> Result<tonic::Response<keymanagement::ExplainKeyRingDecisionResponse>, tonic::Status> {
        self.run_explain_key_ring_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a key ring.
    async fn score_key_ring(
        &self,
        request: tonic::Request<keymanagement::ScoreKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::ScoreKeyRingResponse>, tonic::Status> {
        self.run_score_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a key ring.
    async fn lint_key_rings(
        &self,
        request: tonic::Request<keymanagement::LintKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::LintKeyRingsResponse>, tonic::Status> {
        self.run_lint_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key ring.
    async fn get_key_ring_rules(
        &self,
        request: tonic::Request<keymanagement::GetKeyRingRulesRequest>,
    ) -> Result<tonic::Response<keymanagement::GetKeyRingRulesResponse>, tonic::Status> {
        self.run_get_key_ring_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
